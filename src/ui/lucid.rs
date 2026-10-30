//! Lucid-inspired native presentation, isolated from playback and persistence.
//!
//! Colours remain ordinary Spotifast JSON palettes. Build with `lucid` for
//! rounded glass panels and a softened now-playing backdrop. The existing
//! artwork worker owns loading, blurring, caching and memory bounds.

use egui::{Color32, Frame, Id, Rect, TextureHandle, pos2};

use crate::{app::App, theme::Palette};

const ENABLED: bool = cfg!(feature = "lucid");
const BACKDROP_ID: &str = "lucid-backdrop";
const FADE_SECONDS: f64 = 0.45;
const RAPHAEL_CLOCK: &str = "lucid-raphael-clock";
const PRIMARY: &[u8] = include_bytes!("../../contrib/lucid/raphael/primary.svg");
const SECONDARY: &[u8] = include_bytes!("../../contrib/lucid/raphael/secondary.svg");
const GLOW: &[u8] = include_bytes!("../../contrib/lucid/raphael/glow.svg");

/// Preserve the original frame exactly in ordinary upstream builds.
pub(super) fn panel(frame: Frame, palette: &Palette) -> Frame {
    if !ENABLED {
        return frame;
    }
    // A Frame's stroke is part of its layout margin. Paint this one pixel
    // border inward, preserving the upstream content and scroll rectangles.
    let margin = frame.inner_margin;
    frame
        .fill(palette.panel.gamma_multiply(0.76))
        .corner_radius(12)
        .inner_margin(egui::Margin {
            left: margin.left - 1,
            right: margin.right - 1,
            top: margin.top - 1,
            bottom: margin.bottom - 1,
        })
        .stroke(egui::Stroke::new(1.0, palette.outline))
}

#[derive(Clone, Default)]
struct Backdrop {
    source: String,
    current: Option<TextureHandle>,
    previous: Option<TextureHandle>,
    changed_at: f64,
}

/// Signing out must release the previous account's visible artwork.
pub(super) fn end_session(ctx: &egui::Context) {
    ctx.data_mut(|data| data.remove::<Backdrop>(Id::new(BACKDROP_ID)));
    ctx.data_mut(|data| data.remove::<DecorationClock>(Id::new(RAPHAEL_CLOCK)));
}

/// A paused/hidden window must not accrue a jump in decorative animation.
#[derive(Clone, Default)]
struct DecorationClock {
    elapsed: f64,
    last: f64,
    running: bool,
}

impl DecorationClock {
    fn advance(&mut self, now: f64, running: bool) -> f32 {
        if self.running && running {
            self.elapsed += (now - self.last).clamp(0.0, 0.25);
        }
        self.last = now;
        self.running = running;
        (self.elapsed % 900.0) as f32
    }
}

/// The original transparent arrays, behind content but above the panel fill.
/// Three cached textures and three quads; no SVG parsing or rasterization per
/// frame, no layout allocation, hit targets, network requests or new settings.
pub(super) fn raphael(app: &App, ui: &egui::Ui) {
    if !ENABLED || app.settings.custom_theme.as_deref() != Some("TrueVision.json") {
        return;
    }
    let rect = ui.max_rect();
    if !ui.is_rect_visible(rect) {
        return;
    }
    let ctx = ui.ctx();
    let running = app.believed_playing()
        && !app.motion.reduced()
        && app.budget == crate::power::Budget::Normal;
    let now = ui.input(|input| input.time);
    let seconds = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<DecorationClock>(Id::new(RAPHAEL_CLOCK))
            .advance(now, running)
    });
    if running {
        // These very slow rotations need only 12 frames/s. Paused, hidden,
        // reduced-motion and energy-saving views schedule no decorative frames.
        ctx.request_repaint_after(std::time::Duration::from_millis(84));
    }
    let painter = ui.painter().with_clip_rect(rect);
    let scale = (rect.width() / 1280.0).min(rect.height() / 800.0);
    let square = egui::Vec2::splat(1600.0 * scale);
    let tint = Color32::WHITE.gamma_multiply(if app.palette.dark { 1.0 } else { 0.45 });
    decoration(
        &painter,
        "bytes://raphael/glow.svg",
        GLOW,
        [256, 512],
        rect,
        0.0,
        tint,
    );
    decoration(
        &painter,
        "bytes://raphael/secondary.svg",
        SECONDARY,
        [640, 640],
        Rect::from_center_size(rect.lerp_inside(egui::vec2(-0.01, 0.63)), square),
        -seconds * std::f32::consts::TAU / 180.0,
        tint,
    );
    decoration(
        &painter,
        "bytes://raphael/primary.svg",
        PRIMARY,
        [1280, 1280],
        Rect::from_center_size(rect.lerp_inside(egui::vec2(0.72, 0.52)), square),
        seconds * std::f32::consts::TAU / 300.0,
        tint,
    );
}

fn decoration(
    painter: &egui::Painter,
    uri: &'static str,
    bytes: &'static [u8],
    size: [u32; 2],
    rect: Rect,
    angle: f32,
    tint: Color32,
) {
    let ctx = painter.ctx();
    // Fixed hints bound GPU memory across resizes/DPI changes (8.4 MiB total).
    let installed = ctx.data_mut(|data| {
        std::mem::replace(
            data.get_temp_mut_or_default::<bool>(Id::new("raphael-loaders")),
            true,
        )
    });
    if !installed {
        egui_extras::install_image_loaders(ctx);
    }
    ctx.include_bytes(uri, bytes);
    match ctx.try_load_texture(
        uri,
        egui::TextureOptions::LINEAR,
        egui::load::SizeHint::Size {
            width: size[0],
            height: size[1],
            maintain_aspect_ratio: false,
        },
    ) {
        Ok(egui::load::TexturePoll::Ready { texture }) => {
            // Reuse the existing artwork policy: retain the GPU texture, not
            // another full decoded copy in RAM. The texture loader caches it.
            crate::images::forget_decoded(ctx, uri);
            let mut mesh = egui::Mesh::with_texture(texture.id);
            mesh.add_rect_with_uv(
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                tint,
            );
            mesh.rotate(egui::emath::Rot2::from_angle(angle), rect.center());
            painter.add(mesh);
        }
        Ok(egui::load::TexturePoll::Pending { .. }) => {}
        Err(error) => {
            let id = Id::new(("raphael-load-error", uri));
            let reported = ctx.data_mut(|data| {
                let reported = data.get_temp_mut_or_default::<bool>(id);
                let previous = *reported;
                *reported = true;
                previous
            });
            if !reported {
                log::warn!("Raphael artwork {uri} could not load: {error}");
            }
        }
    }
}

pub(super) fn background(app: &mut App, ui: &egui::Ui) {
    if !ENABLED {
        return;
    }
    let rect = ui.max_rect();
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 0, app.palette.window);
    painter.rect_filled(rect, 0, app.palette.tint);
    let source = app
        .settings
        .accent_from_art
        .then(|| {
            app.now_playing()
                .and_then(|now| now.art_url.or(now.art_small))
        })
        .flatten();
    let Some(source) = source else {
        ui.ctx()
            .data_mut(|data| data.remove::<Backdrop>(Id::new(BACKDROP_ID)));
        return;
    };
    let art = app.backend.art().clone();
    let texture = app.softened_covers.texture(ui.ctx(), &art, &source);
    let id = Id::new(BACKDROP_ID);
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<Backdrop>(id))
        .unwrap_or_default();
    let now = ui.input(|input| input.time);
    if state.source != source
        && let Some(texture) = texture
    {
        state.previous = state.current.take();
        state.current = Some(texture);
        state.source = source;
        state.changed_at = now;
    }
    let progress = if app.motion.reduced() {
        1.0
    } else {
        ((now - state.changed_at) / FADE_SECONDS).clamp(0.0, 1.0) as f32
    };
    // Draw the old opaque image first. Fading the new image over it keeps
    // the backdrop equally bright throughout a track change.
    if let Some(old) = &state.previous {
        paint_cover(&painter, rect, old, 1.0);
    }
    if let Some(current) = &state.current {
        paint_cover(&painter, rect, current, progress);
    }
    if progress < 1.0 {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    } else {
        state.previous = None;
    }
    // Keep text readable in either appearance, independently of the cover.
    let wash = if app.palette.dark {
        Color32::from_black_alpha(100)
    } else {
        Color32::from_white_alpha(170)
    };
    painter.rect_filled(rect, 0, wash);
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
}

fn paint_cover(painter: &egui::Painter, rect: Rect, texture: &TextureHandle, opacity: f32) {
    painter.image(
        texture.id(),
        rect,
        cover_uv(texture.size_vec2(), rect.size()),
        Color32::WHITE.gamma_multiply(opacity),
    );
}

/// Centre crop, like CSS background-size: cover, without stretching artwork.
fn cover_uv(image: egui::Vec2, target: egui::Vec2) -> Rect {
    if image.x <= 0.0 || image.y <= 0.0 || target.x <= 0.0 || target.y <= 0.0 {
        return Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    }
    let scale = (target.x / image.x).max(target.y / image.y);
    let visible = target / (image * scale);
    Rect::from_center_size(pos2(0.5, 0.5), visible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn decoration_freezes_when_paused_and_does_not_jump_after_a_hidden_window() {
        let mut clock = DecorationClock::default();
        assert_eq!(clock.advance(10.0, true), 0.0);
        assert_eq!(clock.advance(10.125, true), 0.125);
        assert_eq!(clock.advance(10.25, false), 0.125);
        assert_eq!(clock.advance(100.0, false), 0.125);
        assert_eq!(clock.advance(200.0, true), 0.125);
        assert_eq!(clock.advance(200.125, true), 0.25);
        assert_eq!(clock.advance(500.0, true), 0.5, "hidden time is bounded");
    }

    #[test]
    fn embedded_arrays_render_transparently_without_external_assets() {
        for bytes in [PRIMARY, SECONDARY, GLOW] {
            let svg = std::str::from_utf8(bytes).unwrap();
            for forbidden in ["href=", "<script", "<image", "<animate", "<foreignObject"] {
                assert!(
                    !svg.contains(forbidden),
                    "embedded artwork must be self-contained"
                );
            }
            let image = egui_extras::image::load_svg_bytes_with_size(
                bytes,
                egui::load::SizeHint::Width(128),
                &Default::default(),
            )
            .unwrap();
            assert!(image.pixels.iter().any(|pixel| pixel.a() > 0));
            assert!(image.pixels.iter().any(|pixel| pixel.a() == 0));
        }
    }

    #[test]
    fn resizing_reuses_the_decoration_texture_instead_of_allocating_another() {
        let ctx = egui::Context::default();
        for (width, uploads) in [(760.0, 1), (1280.0, 0), (1920.0, 0)] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                decoration(
                    ui.painter(),
                    "bytes://raphael/cache-test.svg",
                    PRIMARY,
                    [128, 128],
                    Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 800.0)),
                    0.0,
                    Color32::WHITE,
                );
            });
            // The font atlas is unrelated to the backdrop's texture cache.
            assert_eq!(
                output
                    .textures_delta
                    .set
                    .values()
                    .flat_map(|deltas| deltas.iter())
                    .filter(|delta| delta.image.size() == [128, 128])
                    .count(),
                uploads
            );
            assert_eq!(
                ctx.loaders()
                    .image
                    .lock()
                    .iter()
                    .map(|loader| loader.byte_size())
                    .sum::<usize>(),
                0,
                "uploaded decoration pixels must not remain duplicated in RAM"
            );
            output.textures_delta.clear();
        }
    }

    #[test]
    fn truevision_uses_the_standard_palette_format() {
        let palette: Palette =
            fastframe_theme::parse_palette(include_str!("../../contrib/lucid/TrueVision.json"))
                .unwrap();
        assert!(palette.dark);
        assert_eq!(palette.accent, Color32::from_rgb(0xd3, 0xae, 0x5b));
        assert_eq!(palette.text, Color32::from_rgb(0xf2, 0xe6, 0xc3));
    }

    #[test]
    fn centre_crop_preserves_artwork_proportions() {
        assert_eq!(
            cover_uv(vec2(100.0, 100.0), vec2(200.0, 100.0)),
            Rect::from_min_max(pos2(0.0, 0.25), pos2(1.0, 0.75))
        );
        assert_eq!(
            cover_uv(vec2(100.0, 100.0), vec2(100.0, 200.0)),
            Rect::from_min_max(pos2(0.25, 0.0), pos2(0.75, 1.0))
        );
    }

    #[test]
    fn no_artwork_survives_the_end_of_a_session() {
        let ctx = egui::Context::default();
        ctx.data_mut(|data| data.insert_temp(Id::new(BACKDROP_ID), Backdrop::default()));
        end_session(&ctx);
        assert!(
            ctx.data(|data| data.get_temp::<Backdrop>(Id::new(BACKDROP_ID)))
                .is_none()
        );
    }

    #[test]
    fn ordinary_builds_keep_the_original_panel_geometry() {
        let original = Frame::new().fill(Palette::dark().panel);
        let result = panel(original, &Palette::dark());
        if !ENABLED {
            assert_eq!(original, result);
        } else {
            assert_eq!(result.total_margin(), original.total_margin());
            assert_eq!(result.corner_radius, egui::CornerRadius::same(12));
        }
    }
}
