//! The words of the playing track, in a side panel that follows the song.

use egui::{Align, Color32, Frame, Layout, Margin, Rect, Sense, UiBuilder, pos2, vec2};

use crate::app::App;
use crate::i18n::{gettext, pgettext};
use crate::model::{Action, Loadable};
use crate::theme::{self, Icon};

use super::widgets;
use std::sync::Arc;

const LINE_SIZE: f32 = 19.0;
const LINE_GAP: f32 = 10.0;
/// Where the line being sung sits, as a fraction of the visible lyrics from
/// the top: high up, so the lines to come fill most of the view.
const SUNG_LINE_AT: f32 = 0.2;

/// Scrolls so the middle of `line` sits `SUNG_LINE_AT` of the way down the
/// visible lyrics.
fn show_sung_line(ui: &egui::Ui, line: Rect, animation: Option<egui::style::ScrollAnimation>) {
    let above = (ui.clip_rect().height() * SUNG_LINE_AT - line.height() / 2.0).max(0.0);
    let target = Rect::from_min_max(pos2(line.left(), line.top() - above), line.max);
    match animation {
        Some(animation) => ui.scroll_to_rect_animation(target, Some(Align::Min), animation),
        None => ui.scroll_to_rect(target, Some(Align::Min)),
    }
}
/// How long a line takes to light up or fade.
const LIGHT_UP_SECONDS: f32 = 0.22;

fn blend(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from(egui::Rgba::from(from) * (1.0 - t) + egui::Rgba::from(to) * t)
}

/// Recolour the already-shaped glyph mesh. Word changes never reshape text,
/// change wrapping, or break Arabic joins and combining characters.
fn word_galley(
    galley: Arc<egui::Galley>,
    line: &crate::lyrics::Line,
    position: u32,
    quiet: Color32,
    sung: Color32,
    accent: Color32,
    reduced: bool,
) -> Arc<egui::Galley> {
    if line.words.is_empty() || galley.job.text != line.text {
        return galley;
    }
    let word_index = |cluster: u32| {
        let byte = cluster as usize;
        let index = line.words.partition_point(|word| word.byte_end <= byte);
        line.words
            .get(index)
            .filter(|word| word.byte_start <= byte)
            .map(|_| index)
    };
    let mut widths = vec![0.0f32; line.words.len()];
    for row in &galley.rows {
        for glyph in &row.glyphs {
            if let Some(index) = word_index(glyph.cluster) {
                widths[index] += glyph.advance_width.max(0.0);
            }
        }
    }
    let mut used = vec![0.0f32; widths.len()];
    let mut painted = (*galley).clone();
    for placed in &mut painted.rows {
        let row = Arc::make_mut(&mut placed.row);
        for glyph in &row.glyphs {
            let Some(index) = word_index(glyph.cluster) else {
                continue;
            };
            let word = &line.words[index];
            let previous = used[index];
            used[index] += glyph.advance_width.max(0.0);
            if glyph.uv_rect.is_nothing() {
                continue;
            }
            let Some(vertices) = row
                .visuals
                .mesh
                .vertices
                .get_mut(glyph.first_vertex as usize..glyph.first_vertex as usize + 4)
            else {
                continue;
            };
            for vertex in vertices {
                vertex.color = if position >= word.end_ms {
                    sung
                } else if position < word.start_ms {
                    quiet
                } else if reduced {
                    accent
                } else {
                    let progress = (position - word.start_ms) as f32
                        / (word.end_ms - word.start_ms).max(1) as f32;
                    let within = if glyph.rtl {
                        glyph.max_x() - vertex.pos.x
                    } else {
                        vertex.pos.x - glyph.pos.x
                    };
                    let softness = (glyph.font_height * 0.35).max(1.0);
                    let fill = (widths[index] * progress - previous - within) / softness + 0.5;
                    blend(quiet, accent, fill)
                };
            }
        }
    }
    Arc::new(painted)
}

fn import_menu(app: &mut App, ui: &mut egui::Ui) {
    let available = app.lyrics_import_uri.is_none()
        && !app.offline
        && app.now_playing().is_some_and(|now| !now.is_episode);
    ui.add_enabled_ui(available, |ui| {
        if ui
            .button(gettext(app.locale, "Import lyrics file…"))
            .clicked()
        {
            app.actions.push(Action::ImportLyrics);
            ui.close();
        }
        if matches!(&app.lyrics, Loadable::Loaded(Some(lyrics)) if lyrics.imported)
            && ui
                .button(gettext(app.locale, "Use online lyrics"))
                .clicked()
        {
            app.actions.push(Action::UseOnlineLyrics);
            ui.close();
        }
    });
}

fn lyric_clock(
    app: &App,
    ui: &egui::Ui,
    lyrics: &crate::lyrics::Lyrics,
    now: &crate::app::NowPlaying,
    active: Option<usize>,
    visible: bool,
) {
    if !now.playing
        || app.window_hidden
        || app.budget == crate::power::Budget::Background
        || !lyrics.synced
    {
        return;
    }
    let focus = ui
        .ctx()
        .input(|input| input.viewport().focused.unwrap_or(true));
    let animated = visible
        && focus
        && app.budget == crate::power::Budget::Normal
        && !crate::motion::reduced(ui.ctx());
    let in_word = active
        .and_then(|index| lyrics.lines.get(index))
        .is_some_and(|line| {
            line.words
                .iter()
                .any(|word| word.start_ms <= now.position_ms && now.position_ms < word.end_ms)
        });
    let boundary = lyrics
        .lines
        .iter()
        .flat_map(|line| {
            line.at_ms.into_iter().chain(line.end_ms).chain(
                line.words
                    .iter()
                    .flat_map(|word| [word.start_ms, word.end_ms]),
            )
        })
        .filter(|at| *at > now.position_ms)
        .min();
    if animated && in_word {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    } else if let Some(next) = boundary {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(u64::from(
                next - now.position_ms,
            )));
    }
}

pub fn side_panel(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let fit = super::yielding_panel(
        ui.ctx(),
        "lyrics-panel",
        theme::SIDE_PANEL_MIN_WIDTH..=640.0,
        app.settings.lyrics_width,
        ui.available_width() - super::topbar::least_width(ui.ctx()),
    );
    let panel = egui::Panel::right("lyrics-panel")
        .resizable(true)
        .default_size(app.settings.lyrics_width)
        .size_range(fit.range.clone())
        .show_separator_line(false)
        .frame(super::lucid::panel(
            Frame::new()
                .fill(palette.panel)
                .inner_margin(Margin::symmetric(12, 12)),
            &palette,
        ));
    let response = panel.show(ui, |ui| {
        let window_controls = super::window_controls_reservation(
            ui.ctx(),
            app.show_queue_panel,
            app.show_lyrics_panel,
            ui.available_width(),
        );
        ui.add_space(window_controls.lyrics_top);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            theme::text(
                ui,
                gettext(app.locale, "Lyrics"),
                theme::bold(18.0),
                palette.text,
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::icon_button(
                    ui,
                    Icon::X,
                    18.0,
                    palette.secondary,
                    palette.text,
                    &gettext(app.locale, "Close"),
                )
                .clicked()
                {
                    app.actions.push(Action::ToggleLyricsPanel);
                }
                if theme::icon_button(
                    ui,
                    Icon::Expand,
                    18.0,
                    palette.secondary,
                    palette.text,
                    &gettext(app.locale, "Full screen lyrics"),
                )
                .clicked()
                {
                    app.actions.push(Action::SetLyricsFullscreen(true));
                }
                let more = theme::icon_button(
                    ui,
                    Icon::Ellipsis,
                    18.0,
                    palette.secondary,
                    palette.text,
                    &gettext(app.locale, "Lyrics options"),
                );
                egui::Popup::menu(&more)
                    .frame(widgets::menu_frame(&palette))
                    .show(|ui| import_menu(app, ui));
                let loaded = matches!(&app.lyrics, Loadable::Loaded(Some(_)));
                if loaded
                    && !app.lyrics_following
                    && theme::pill_button(
                        ui,
                        &palette,
                        &pgettext(app.locale, "lyrics", "Follow"),
                        false,
                    )
                    .clicked()
                {
                    app.lyrics_following = true;
                    app.lyrics_line_shown = None;
                }
            });
        });
        ui.add_space(8.0);
        contents(app, ui);
    });
    let current_width = response.response.rect.width();
    if (app.settings.lyrics_width - current_width).abs() > 1.0
        && super::panel_width_chosen(ui.ctx(), "lyrics-panel", &fit)
    {
        app.settings.lyrics_width = current_width;
        app.actions.push(Action::SettingsChanged);
    }
}

fn contents(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let Some(now) = app.now_playing() else {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Mic,
            &gettext(app.locale, "Nothing playing"),
            &gettext(app.locale, "Play a song to see its lyrics."),
        );
        return;
    };
    let lyrics = match &app.lyrics {
        Loadable::NotLoaded | Loadable::Loading => {
            if !app.network_reachable {
                widgets::empty_state(
                    ui,
                    &palette,
                    Icon::WifiOff,
                    &gettext(app.locale, "You're offline"),
                    &gettext(
                        app.locale,
                        "Spotifast will reconnect automatically when your connection returns.",
                    ),
                );
            } else {
                widgets::loading_row(ui, &palette, app.locale);
            }
            return;
        }
        Loadable::Failed(error) => {
            if !app.network_reachable {
                widgets::empty_state(
                    ui,
                    &palette,
                    Icon::WifiOff,
                    &gettext(app.locale, "You're offline"),
                    &gettext(
                        app.locale,
                        "Spotifast will reconnect automatically when your connection returns.",
                    ),
                );
                return;
            }
            let message = gettext(
                app.locale,
                // Translators: Keep {error} unchanged. It is the original failure detail.
                "Couldn't fetch the lyrics: {error}",
            )
            .replace("{error}", error);
            ui.add_space(8.0);
            theme::text(ui, message, theme::regular(13.0), palette.secondary);
            ui.add_space(8.0);
            if theme::pill_button(ui, &palette, &gettext(app.locale, "Try again"), false).clicked()
            {
                app.request_lyrics();
            }
            return;
        }
        Loadable::Loaded(None) => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Mic,
                &gettext(app.locale, "No lyrics"),
                &gettext(app.locale, "No lyrics found for this track."),
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) if lyrics.instrumental => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Music,
                &gettext(app.locale, "Instrumental"),
                &gettext(app.locale, "No timed lyrics for this track."),
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) => lyrics,
    };

    let active = lyrics.active_line(now.position_ms);
    let follow = app.lyrics_following && app.lyrics_line_shown != Some(active);
    // The line being sung is bold and in the accent colour; every other
    // line is quiet, regular text, the same before and after it has been
    // sung. A line takes 220 ms to light up or fade, as in omarchy-lyrics.
    let quiet = palette.text.gamma_multiply(0.45);
    let reduced = crate::motion::reduced(ui.ctx()) || app.budget != crate::power::Budget::Normal;
    let mut active_visible = false;
    let scroll = crate::autoscroll::show(
        ui,
        egui::ScrollArea::vertical()
            .id_salt("lyrics-scroll")
            .auto_shrink([false, false]),
        egui::Vec2b::new(false, true),
        |ui| {
            // Before the first line there is nothing to highlight, so the
            // panel sits at the top rather than wherever it was left.
            if follow
                && lyrics.synced
                && active.is_none()
                && lyrics
                    .lines
                    .first()
                    .and_then(|line| line.at_ms)
                    .is_some_and(|at| now.position_ms < at)
            {
                let top = ui.cursor().min;
                ui.scroll_to_rect(
                    egui::Rect::from_min_size(top, egui::vec2(1.0, 1.0)),
                    Some(Align::Min),
                );
            }
            ui.add_space(12.0);
            for (index, line) in lyrics.lines.iter().enumerate() {
                let is_active = active == Some(index);
                let lit = ui.ctx().animate_bool_with_time(
                    egui::Id::new("lyric-line").with(index),
                    is_active,
                    crate::motion::seconds(ui.ctx(), LIGHT_UP_SECONDS),
                );
                let color = blend(quiet, palette.accent, lit);
                let font = if lit > 0.5 || !line.words.is_empty() {
                    theme::bold(LINE_SIZE)
                } else {
                    theme::regular(LINE_SIZE)
                };
                // A timed line with no words is the band playing on.
                let text = if line.text.is_empty() && lyrics.synced {
                    "\u{266a}"
                } else {
                    line.text.as_str()
                };
                let sense = if lyrics.synced {
                    Sense::click()
                } else {
                    Sense::hover()
                };
                let response = if is_active && !line.words.is_empty() {
                    let galley = crate::bidi::layout(
                        ui.painter(),
                        text,
                        font,
                        quiet,
                        ui.available_width(),
                        usize::MAX,
                        None,
                    );
                    let galley = if ui
                        .is_rect_visible(Rect::from_min_size(ui.cursor().min, galley.size()))
                    {
                        word_galley(
                            galley,
                            line,
                            now.position_ms,
                            quiet,
                            palette.text,
                            palette.accent,
                            reduced,
                        )
                    } else {
                        galley
                    };
                    ui.add(egui::Label::new(galley).sense(sense))
                } else if crate::bidi::is_rtl(text) {
                    let galley = crate::bidi::layout(
                        ui.painter(),
                        text,
                        font,
                        color,
                        ui.available_width(),
                        usize::MAX,
                        None,
                    );
                    ui.add(egui::Label::new(galley).sense(sense))
                } else {
                    ui.add(
                        egui::Label::new(egui::RichText::new(text).font(font).color(color))
                            .sense(sense),
                    )
                };
                crate::autoscroll::row(ui, &response);
                let rect = response.rect;
                active_visible |= is_active && ui.is_rect_visible(rect);
                if lyrics.synced
                    && response.clicked()
                    && let Some(at_ms) = line.at_ms
                {
                    app.actions.push(Action::Seek(at_ms));
                    app.lyrics_following = true;
                }
                if is_active && follow {
                    show_sung_line(ui, rect, None);
                }
                ui.add_space(LINE_GAP);
            }
            // Words without timing can only be followed by the clock: sit
            // at the part of the text the song is probably at.
            if app.lyrics_following && !lyrics.synced && now.duration_ms > 0 {
                let fraction =
                    (f64::from(now.position_ms) / f64::from(now.duration_ms)).clamp(0.0, 1.0);
                let content = ui.min_rect();
                let y = content.top() + content.height() * fraction as f32;
                ui.scroll_to_rect(
                    egui::Rect::from_min_max(
                        egui::pos2(content.left(), y),
                        egui::pos2(content.right(), y + 1.0),
                    ),
                    Some(Align::Center),
                );
            }
            // Room for the last line to rise to where a sung line sits.
            ui.add_space((ui.clip_rect().height() * (1.0 - SUNG_LINE_AT)).max(60.0));
        },
    );
    crate::autoscroll::lyrics(ui, scroll.id);
    // Scrolling by hand means the reader wants to look elsewhere; the
    // Follow button in the header picks the song back up.
    if ui.rect_contains_pointer(scroll.inner_rect)
        && ui.input(|input| input.smooth_scroll_delta.y != 0.0)
    {
        app.lyrics_following = false;
    }
    app.lyrics_line_shown = Some(active);
    lyric_clock(app, ui, lyrics, &now, active, active_visible);
}

pub fn fullscreen(app: &mut App, ui: &mut egui::Ui) {
    egui::CentralPanel::default()
        .frame(Frame::new().fill(theme::Palette::dark().window))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            background(app, ui, rect);
            let top = theme::titlebar_inset(ui.ctx()) + 24.0;
            if app.now_playing().is_some() && rect.width() >= COVER_BESIDE_MIN_WIDTH {
                with_cover(app, ui, rect, top);
                return;
            }
            let width = fullscreen_content_width(rect.width());
            let region = Rect::from_min_max(
                pos2(rect.center().x - width / 2.0, rect.top() + top),
                pos2(rect.center().x + width / 2.0, rect.bottom()),
            );
            let mut content = ui.new_child(UiBuilder::new().max_rect(region));
            fullscreen_header(app, &mut content);
            content.add_space(20.0);
            track_heading(app, &mut content);
            content.add_space(16.0);
            fullscreen_contents(app, &mut content);
        });
}

/// The widest the lyrics get beside the cover, so lines stay easy to read.
const LYRICS_BESIDE_WIDTH: f32 = 640.0;

/// The narrowest window that shows the cover beside the lyrics; narrower
/// ones keep a single column with a small cover in the heading.
const COVER_BESIDE_MIN_WIDTH: f32 = 900.0;

/// Full screen with the cover large: beside the lyrics when there are
/// words to follow, and alone in the middle when there are none, a calm
/// view of what is playing.
fn with_cover(app: &mut App, ui: &mut egui::Ui, rect: Rect, top: f32) {
    let outer = Rect::from_min_max(
        pos2(rect.left() + 48.0, rect.top() + top),
        pos2(rect.right() - 48.0, rect.bottom() - 40.0),
    );
    let mut header = ui.new_child(UiBuilder::new().max_rect(outer));
    fullscreen_header(app, &mut header);
    let below = Rect::from_min_max(
        pos2(outer.left(), header.min_rect().bottom() + 24.0),
        outer.max,
    );
    // The cover moves aside only for words to read. While they load it
    // stays in the middle, so a song that turns out to have none never
    // moves at all.
    let words = matches!(&app.lyrics, Loadable::Loaded(Some(lyrics)) if !lyrics.instrumental);
    // The queue docks right while its flag is on, in both arrangements.
    // Without it the room below is untouched, preserving the stock layout.
    let gap = 64.0;
    let queue_width = if app.show_queue_panel {
        // Shared with the side panel's persisted width; the column's drag
        // strip below writes the same field.
        app.settings.queue_width.clamp(
            super::queue::FULLSCREEN_COLUMN_MIN,
            super::queue::FULLSCREEN_COLUMN_MAX,
        )
    } else {
        0.0
    };
    let queue_gap = if app.show_queue_panel { gap } else { 0.0 };
    let lyrics_end = below.right() - queue_width - queue_gap;
    if app.show_queue_panel {
        let queue_rect = Rect::from_min_max(
            pos2(below.right() - queue_width, below.top()),
            pos2(below.right(), below.bottom()),
        );
        let mut queue_ui = ui.new_child(UiBuilder::new().max_rect(queue_rect));
        super::queue::fullscreen_column(app, &mut queue_ui);
    }
    if words {
        if app.show_queue_panel {
            // The cover docks left; the lyrics take the middle that remains.
            let room = lyrics_end - below.left();
            let side = (room * 0.42).min(below.height() - 90.0).clamp(160.0, 520.0);
            let column = Rect::from_min_size(
                pos2(below.left(), below.center().y - (side + 90.0) / 2.0),
                vec2(side, side + 90.0),
            );
            big_cover(app, ui, column, Align::Min);
            let lyrics_avail = lyrics_end - column.right() - gap;
            let lyrics_width = lyrics_avail.clamp(80.0, LYRICS_BESIDE_WIDTH);
            // Centre the block between the cover and the queue when there is
            // room to spare; it hugs the cover on narrow windows.
            let lyrics_x = column.right() + gap + (lyrics_avail - lyrics_width).max(0.0) / 2.0;
            let lyrics = Rect::from_min_max(
                pos2(lyrics_x, below.top()),
                pos2(lyrics_x + lyrics_width, below.bottom()),
            );
            let mut content = ui.new_child(UiBuilder::new().max_rect(lyrics));
            fullscreen_contents(app, &mut content);
        } else {
            // The cover and the lyrics are one group, centred in the window.
            let side = (below.width() * 0.38)
                .min(below.height() - 90.0)
                .clamp(200.0, 520.0);
            let lyrics_width = (below.width() - side - gap).min(LYRICS_BESIDE_WIDTH);
            let left = below.center().x - (side + gap + lyrics_width) / 2.0;
            let column = Rect::from_min_size(
                pos2(left, below.center().y - (side + 90.0) / 2.0),
                vec2(side, side + 90.0),
            );
            big_cover(app, ui, column, Align::Min);
            let lyrics = Rect::from_min_max(
                pos2(column.right() + gap, below.top()),
                pos2(column.right() + gap + lyrics_width, below.bottom()),
            );
            let mut content = ui.new_child(UiBuilder::new().max_rect(lyrics));
            fullscreen_contents(app, &mut content);
        }
    } else {
        let span = Rect::from_min_max(
            pos2(below.left(), below.top()),
            pos2(lyrics_end, below.bottom()),
        );
        // Without the queue the span is the whole room, matching the stock
        // centred cover exactly.
        let min_side = if app.show_queue_panel { 160.0 } else { 200.0 };
        let side = (span.height() - 140.0)
            .min(span.width() * 0.5)
            .clamp(min_side, 560.0);
        let column = Rect::from_center_size(span.center(), vec2(side, side + 90.0));
        big_cover(app, ui, column, Align::Center);
        // Why there are no words, quietly, under the song, or that they
        // are still being fetched.
        let (heading, detail) = match &app.lyrics {
            Loadable::Loaded(Some(_)) => (
                gettext(app.locale, "Instrumental"),
                gettext(app.locale, "No timed lyrics for this track."),
            ),
            Loadable::Loaded(None) => (
                gettext(app.locale, "No lyrics"),
                gettext(app.locale, "No lyrics found for this track."),
            ),
            Loadable::Failed(error) => (
                gettext(
                    app.locale,
                    // Translators: Keep {error} unchanged. It is the original failure detail.
                    "Couldn't fetch the lyrics: {error}",
                )
                .replace("{error}", error)
                .into(),
                Default::default(),
            ),
            Loadable::NotLoaded | Loadable::Loading => {
                (gettext(app.locale, "Loading…"), Default::default())
            }
        };
        let heading = ui.painter().text(
            pos2(column.center().x, column.bottom() + 8.0),
            egui::Align2::CENTER_TOP,
            heading,
            theme::semibold(13.0),
            Color32::from_gray(200),
        );
        if matches!(app.lyrics, Loadable::Failed(_)) {
            let retry = Rect::from_center_size(
                pos2(column.center().x, heading.bottom() + 24.0),
                vec2(column.width(), 32.0),
            );
            let mut retry_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(retry)
                    .layout(Layout::top_down(Align::Center)),
            );
            let label = gettext(app.locale, "Try again");
            if theme::pill_button(&mut retry_ui, &theme::Palette::dark(), &label, false).clicked() {
                app.actions.push(Action::RetryLyrics);
            }
            return;
        }
        ui.painter().text(
            pos2(column.center().x, heading.bottom() + 4.0),
            egui::Align2::CENTER_TOP,
            detail,
            theme::regular(13.0),
            Color32::from_gray(170),
        );
    }
}

/// The playing song's cover filling the top of `column`, with its title and
/// artists beneath, aligned to its left edge or centred.
fn big_cover(app: &App, ui: &mut egui::Ui, column: Rect, align: Align) {
    let Some(now) = app.now_playing() else {
        return;
    };
    let side = column.width();
    let cover = Rect::from_min_size(column.min, vec2(side, side));
    let radius = 10.0;
    ui.painter().add(
        egui::epaint::Shadow {
            offset: [0, 18],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(140),
        }
        .as_shape(cover, radius),
    );
    widgets::playing_cover(
        ui,
        &theme::Palette::dark(),
        app.backend.art(),
        "fullscreen-lyrics",
        now.art_url.as_deref().or(now.art_small.as_deref()),
        cover,
        radius,
    );
    let words = Rect::from_min_max(pos2(column.left(), cover.bottom() + 18.0), column.max);
    let mut text = ui.new_child(
        UiBuilder::new()
            .max_rect(words)
            .layout(Layout::top_down(align)),
    );
    text.spacing_mut().item_spacing.y = 4.0;
    text.add(
        egui::Label::new(
            egui::RichText::new(&now.title)
                .font(theme::semibold(22.0))
                .color(Color32::WHITE),
        )
        .truncate(),
    );
    text.add(
        egui::Label::new(
            egui::RichText::new(&now.subtitle)
                .font(theme::regular(14.0))
                .color(Color32::from_gray(225)),
        )
        .truncate(),
    );
}

fn fullscreen_content_width(viewport_width: f32) -> f32 {
    let available = (viewport_width - 48.0).max(0.0);
    (viewport_width * 0.72).clamp(400.0, 960.0).min(available)
}

fn preferred_backdrop_art(small: Option<String>, large: Option<String>) -> Option<String> {
    small.or(large)
}

fn background(app: &mut App, ui: &mut egui::Ui, rect: Rect) {
    theme::apply_local(ui, &theme::Palette::dark());
    let art = app
        .now_playing()
        .and_then(|now| preferred_backdrop_art(now.art_small, now.art_url));
    let painter = ui.painter().with_clip_rect(rect);
    if let Some(texture) = app
        .lyrics_backdrop
        .texture(ui.ctx(), app.backend.art(), art.as_deref())
    {
        painter.image(
            texture.id(),
            rect,
            cover_uv(rect.size(), texture.size_vec2()),
            Color32::from_gray(180),
        );
    }
    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(120));
    widgets::paint_vertical_gradient(
        ui,
        rect,
        Color32::from_black_alpha(0),
        Color32::from_black_alpha(95),
    );
}

fn cover_uv(view: egui::Vec2, image: egui::Vec2) -> Rect {
    let ratio = (view.x / view.y.max(1.0)) / (image.x / image.y.max(1.0));
    let size = if ratio > 1.0 {
        vec2(1.0, 1.0 / ratio)
    } else {
        vec2(ratio, 1.0)
    };
    Rect::from_center_size(pos2(0.5, 0.5), size)
}

fn fullscreen_header(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::Palette::dark();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(gettext(app.locale, "Lyrics"))
                .font(theme::bold(18.0))
                .color(palette.text),
        )
        .context_menu(|ui| import_menu(app, ui));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::icon_button(
                ui,
                Icon::Shrink,
                18.0,
                palette.text,
                palette.text,
                &gettext(app.locale, "Leave full screen (Esc)"),
            )
            .clicked()
            {
                app.actions.push(Action::SetLyricsFullscreen(false));
            }
            let loaded = matches!(&app.lyrics, Loadable::Loaded(Some(_)));
            if loaded
                && !app.lyrics_following
                && theme::pill_button(
                    ui,
                    &palette,
                    &pgettext(app.locale, "lyrics", "Follow"),
                    false,
                )
                .clicked()
            {
                app.actions.push(Action::FollowLyrics);
            }
        });
    });
}

fn track_heading(app: &App, ui: &mut egui::Ui) {
    if let Some(now) = app.now_playing() {
        ui.horizontal(|ui| {
            let size = 52.0;
            let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
            widgets::playing_cover(
                ui,
                &theme::Palette::dark(),
                app.backend.art(),
                "lyrics-heading",
                app.small_art(&now, size),
                rect,
                4.0,
            );
            ui.vertical(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&now.title)
                            .font(theme::semibold(22.0))
                            .color(Color32::WHITE),
                    )
                    .truncate(),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&now.subtitle)
                            .font(theme::regular(13.0))
                            .color(Color32::from_gray(235)),
                    )
                    .truncate(),
                );
            });
        });
    }
}

fn fullscreen_contents(app: &mut App, ui: &mut egui::Ui) {
    let palette = theme::Palette::dark();
    let Some(now) = app.now_playing() else {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Mic,
            &gettext(app.locale, "Nothing playing"),
            &gettext(app.locale, "Play a song to see its lyrics."),
        );
        return;
    };
    let lyrics = match &app.lyrics {
        Loadable::NotLoaded | Loadable::Loading => {
            widgets::loading_row(ui, &palette, app.locale);
            return;
        }
        Loadable::Failed(error) => {
            let message = gettext(
                app.locale,
                // Translators: Keep {error} unchanged. It is the original failure detail.
                "Couldn't fetch the lyrics: {error}",
            )
            .replace("{error}", error);
            ui.add_space(8.0);
            theme::text(ui, message, theme::regular(13.0), palette.text);
            ui.add_space(8.0);
            if theme::pill_button(ui, &palette, &gettext(app.locale, "Try again"), false).clicked()
            {
                app.actions.push(Action::RetryLyrics);
            }
            return;
        }
        Loadable::Loaded(None) => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Mic,
                &gettext(app.locale, "No lyrics"),
                &gettext(app.locale, "No lyrics found for this track."),
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) if lyrics.instrumental => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Music,
                &gettext(app.locale, "Instrumental"),
                &gettext(app.locale, "No timed lyrics for this track."),
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) => lyrics,
    };

    let active = lyrics.active_line(now.position_ms);
    // A Follow click resets the remembered line after drawing. Other frames
    // record the shown line before any line-click action restores following.
    if !app
        .actions
        .iter()
        .any(|action| matches!(action, Action::FollowLyrics))
    {
        app.actions.push(Action::LyricsLineShown(active));
    }
    let viewport = ui.available_rect_before_wrap();
    let manual_scroll = ui.rect_contains_pointer(viewport)
        && ui.input(|input| {
            input.smooth_scroll_delta.y != 0.0
                || (input.pointer.primary_down() && input.pointer.delta().y != 0.0)
        });
    let following = app.lyrics_following && !manual_scroll;
    let follow = following && app.lyrics_line_shown != Some(active);
    let animation = crate::motion::scroll(ui.ctx(), 0.45);
    let size = (ui.available_width() * 0.046).clamp(28.0, 42.0);
    // The line being sung brightens; all lines keep the same font metrics
    // so highlighting cannot rewrap the words during a transition.
    // A line takes 300 ms to light up or fade.
    let quiet = palette.text.gamma_multiply(0.68);
    let reduced = crate::motion::reduced(ui.ctx()) || app.budget != crate::power::Budget::Normal;
    let mut active_visible = false;
    ui.spacing_mut().scroll.fade.strength = 0.0;
    egui::ScrollArea::vertical()
        .id_salt(("fullscreen-lyrics-scroll", &now.uri))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Before the first line there is nothing to highlight, so the
            // panel sits at the top rather than wherever it was left.
            if follow
                && lyrics.synced
                && active.is_none()
                && lyrics
                    .lines
                    .first()
                    .and_then(|line| line.at_ms)
                    .is_some_and(|at| now.position_ms < at)
            {
                let top = ui.cursor().min;
                ui.scroll_to_rect_animation(
                    egui::Rect::from_min_size(top, egui::vec2(1.0, 1.0)),
                    Some(Align::Min),
                    animation,
                );
            }
            // Room for the first line to sit where a sung line sits, and for
            // the last to rise to it.
            let (padding, below) = if lyrics.synced {
                (
                    (viewport.height() * SUNG_LINE_AT - size).max(12.0),
                    viewport.height() * (1.0 - SUNG_LINE_AT),
                )
            } else {
                (12.0, 60.0)
            };
            ui.add_space(padding);
            for (index, line) in lyrics.lines.iter().enumerate() {
                let is_active = active == Some(index);
                let lit = ui.ctx().animate_bool_with_time(
                    egui::Id::new("lyric-line").with(("fullscreen", &now.uri, index)),
                    is_active,
                    crate::motion::seconds(ui.ctx(), 0.3),
                );
                let color = if lyrics.synced {
                    blend(quiet, palette.text, lit)
                } else {
                    palette.text
                };
                let font = theme::bold(size);
                // A timed line with no words is the band playing on.
                let text = if line.text.is_empty() && lyrics.synced {
                    "\u{266a}"
                } else {
                    line.text.as_str()
                };
                let sense = if lyrics.synced {
                    Sense::click()
                } else {
                    Sense::hover()
                };
                let mut galley = crate::bidi::layout(
                    ui.painter(),
                    text,
                    font,
                    color,
                    ui.available_width(),
                    usize::MAX,
                    None,
                );
                if is_active
                    && !line.words.is_empty()
                    && ui.is_rect_visible(Rect::from_min_size(ui.cursor().min, galley.size()))
                {
                    galley = word_galley(
                        galley,
                        line,
                        now.position_ms,
                        quiet,
                        palette.text,
                        app.palette.accent,
                        reduced,
                    );
                }
                let center = ui.cursor().top() + galley.size().y * 0.5;
                let edge = ((center - viewport.top()).min(viewport.bottom() - center)
                    / (size * 2.0))
                    .clamp(0.0, 1.0);
                let response = ui
                    .scope(|ui| {
                        ui.multiply_opacity(edge * edge * (3.0 - 2.0 * edge));
                        ui.add(egui::Label::new(galley).sense(sense))
                    })
                    .inner;
                let rect = response.rect;
                active_visible |= is_active && ui.is_rect_visible(rect);
                if lyrics.synced
                    && response.clicked()
                    && let Some(at_ms) = line.at_ms
                {
                    app.actions.push(Action::Seek(at_ms));
                    app.actions.push(Action::FollowLyrics);
                }
                if is_active && follow {
                    show_sung_line(ui, rect, Some(animation));
                }
                ui.add_space(27.0);
            }
            // Words without timing can only be followed by the clock: sit
            // at the part of the text the song is probably at.
            if following && !lyrics.synced && now.duration_ms > 0 {
                let fraction =
                    (f64::from(now.position_ms) / f64::from(now.duration_ms)).clamp(0.0, 1.0);
                let content = ui.min_rect();
                let y = content.top() + content.height() * fraction as f32;
                ui.scroll_to_rect_animation(
                    egui::Rect::from_min_max(
                        egui::pos2(content.left(), y),
                        egui::pos2(content.right(), y + 1.0),
                    ),
                    Some(Align::Center),
                    animation,
                );
            }
            ui.add_space(below.max(60.0));
        });
    // Scrolling by hand means the reader wants to look elsewhere; the
    // Follow button in the header picks the song back up.
    if manual_scroll && app.lyrics_following {
        app.actions.push(Action::PauseLyricsFollow);
    }
    lyric_clock(app, ui, lyrics, &now, active, active_visible);
}

#[cfg(test)]
mod tests {
    use super::{fullscreen_content_width, preferred_backdrop_art};

    #[test]
    fn timed_word_paint_changes_only_colors_and_handles_backwards_seeks() {
        use super::word_galley;
        use egui::{Color32, FontId};
        let ctx = egui::Context::default();
        let lyrics = crate::lyrics::from_text("[00:01]<00:01>Hello <00:02>world<00:03>").unwrap();
        let line = &lyrics.lines[0];
        let quiet = Color32::GRAY;
        let sung = Color32::WHITE;
        let accent = Color32::GOLD;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let original = crate::bidi::layout(
                ui.painter(),
                &line.text,
                FontId::proportional(28.0),
                quiet,
                95.0,
                usize::MAX,
                None,
            );
            let geometry = |galley: &egui::Galley| {
                galley
                    .rows
                    .iter()
                    .map(|row| {
                        row.visuals
                            .mesh
                            .vertices
                            .iter()
                            .map(|v| (v.pos, v.uv))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            let colors = |galley: &egui::Galley| {
                galley
                    .rows
                    .iter()
                    .flat_map(|row| row.visuals.mesh.vertices.iter().map(|v| v.color))
                    .collect::<Vec<_>>()
            };
            let early = word_galley(original.clone(), line, 1_500, quiet, sung, accent, false);
            let late = word_galley(original.clone(), line, 2_500, quiet, sung, accent, false);
            let rewind = word_galley(original.clone(), line, 1_500, quiet, sung, accent, false);
            assert_eq!(geometry(&original), geometry(&early));
            assert_eq!(geometry(&early), geometry(&late));
            assert_eq!(original.size(), late.size());
            assert_ne!(colors(&early), colors(&late));
            assert_eq!(colors(&early), colors(&rewind));
            let reduced = word_galley(original.clone(), line, 1_500, quiet, sung, accent, true);
            assert!(colors(&reduced).contains(&accent));
        });
        output.textures_delta.clear();
    }

    #[test]
    fn fullscreen_backdrop_prefers_small_art_with_large_art_as_fallback() {
        let small = "small".to_string();
        let large = "large".to_string();
        assert_eq!(
            preferred_backdrop_art(Some(small.clone()), Some(large.clone())),
            Some(small)
        );
        assert_eq!(
            preferred_backdrop_art(None, Some(large.clone())),
            Some(large)
        );
        assert_eq!(preferred_backdrop_art(None, None), None);
    }

    #[test]
    fn fullscreen_content_width_never_inverts_a_narrow_viewport() {
        for viewport_width in [0.0, 24.0, 47.0, 48.0, 64.0, 760.0, 2_000.0] {
            let width = fullscreen_content_width(viewport_width);
            assert!(width >= 0.0);
            assert!(width <= (viewport_width - 48.0).max(0.0));
        }
        assert_eq!(fullscreen_content_width(47.0), 0.0);
        assert_eq!(fullscreen_content_width(2_000.0), 960.0);
    }
}
