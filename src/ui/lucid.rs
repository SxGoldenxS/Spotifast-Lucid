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
        end_session(ui.ctx());
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
