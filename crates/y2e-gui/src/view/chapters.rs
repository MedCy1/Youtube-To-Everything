//! The chapters of the video: a list to pick one from, and the marks that
//! show where they begin on the timeline.

use rune::ScrollSpec;
use rune::{col, el, row, text, El, Stagger, Transition};
use y2e_core::format_time_seconds;

use crate::i18n::t;
use crate::state::State;
use crate::theme::Palette;

use super::landing::solid_button;
use super::*;

/// The button next to the timeline summary. Nothing when there are no chapters.
pub(super) fn chapters_button(s: &State, p: &Palette) -> Option<El> {
    if s.chapter_ranges().is_empty() {
        return None;
    }
    Some(
        el().key("open-chapters")
            .h(26.0)
            .px(10.0)
            .radius(13.0)
            .fill(p.chip)
            .hover(hover_tint(p, false))
            .center()
            .press_scale(0.96)
            .role("button")
            .label(t().chapters)
            .focusable(true)
            .focus_ring(2.0, 2.0, p.text)
            .child(text(t().chapters).font(face(13.0, 600)).color(p.text_dim))
            .on_press(|s: &mut State| s.chapters_open = true),
    )
}

/// The whole video as a strip, one segment per chapter, with the one under the
/// pointer lit and the chosen one solid.
fn chapter_strip(s: &State, p: &Palette) -> El {
    let chosen = s
        .enable_snipper
        .then_some((s.start_time_secs, s.end_time_secs));
    let mut strip = row().w_full().h(8.0).gap(2.0).a11y_hidden();
    for (index, (_, from, to)) in s.chapter_ranges().into_iter().enumerate() {
        let fill = if s.chapter_preview == Some((from, to)) {
            p.accent
        } else if chosen == Some((from, to)) {
            p.text
        } else {
            p.chip
        };
        strip = strip.child(
            el().key(format!("segment-{index}"))
                .grow(to.saturating_sub(from).max(1) as f32)
                .h_full()
                .radius(4.0)
                .fill(fill)
                .motion(motion::SNAPPY),
        );
    }
    strip
}

/// The chapter under the pointer in the list, lit on the waveform.
pub(super) fn chapter_preview(s: &State, p: &Palette) -> El {
    let Some((from, to)) = s.chapter_preview else {
        return el();
    };
    let total = s.duration().max(1) as f32;
    let (fa, fb) = (from as f32 / total, (to as f32 / total).min(1.0));
    fill_parent().child(
        row()
            .w_full()
            .h_full()
            .child(
                el().grow((fa * 1000.0).max(0.0))
                    .h_full()
                    .motion(motion::SNAPPY),
            )
            .child(
                el().key("chapter-preview")
                    .grow(((fb - fa) * 1000.0).max(8.0))
                    .h_full()
                    .radius(12.0)
                    .fill(p.text.alpha(0.3))
                    .border(2.0, p.text)
                    .transition(Transition::opacity())
                    .motion(motion::SNAPPY),
            )
            .child(
                el().grow(((1.0 - fb) * 1000.0).max(0.0))
                    .h_full()
                    .motion(motion::SNAPPY),
            ),
    )
}

/// Thin lines over the waveform where each chapter after the first begins.
pub(super) fn chapter_marks(s: &State, p: &Palette) -> El {
    let ranges = s.chapter_ranges();
    let total = s.duration().max(1) as f32;
    let mut marks = row().w_full().h_full();
    let mut at = 0.0_f32;
    for (_, start, _) in ranges.iter().skip(1) {
        let start = *start as f32;
        marks = marks
            .child(el().grow(((start - at) * 1000.0).max(1.0)).h_full())
            .child(el().w(2.0).h_full().radius(1.0).fill(p.text.alpha(0.55)));
        at = start;
    }
    if !ranges.is_empty() {
        marks = marks.child(el().grow(((total - at) * 1000.0).max(1.0)).h_full());
    }
    fill_parent().child(marks)
}

/// The scrim and the list, when it is open.
pub(super) fn chapters_overlay(s: &State, p: &Palette) -> El {
    if !s.chapters_open {
        return el();
    }
    let current = s
        .enable_snipper
        .then_some((s.start_time_secs, s.end_time_secs));
    let mut list = col()
        .key("chapter-list")
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .w_full()
        .max_h(380.0)
        // Room for the focus ring: the viewport would cut it off at the edges.
        .padding(6.0)
        .scroll_margin(6.0)
        .gap(6.0);
    let ranges = s.chapter_ranges();
    let cascade = Stagger::new(0.025);
    let count = ranges.len();
    for (index, (title, from, to)) in ranges.into_iter().enumerate() {
        let (a, b) = (format_time_seconds(from), format_time_seconds(to));
        let on = current == Some((from, to));
        let entrance = |row: El| {
            if s.settings.motion.is_light() {
                row
            } else {
                row.transition(Transition::offset(0.0, 10.0).combined(Transition::opacity()))
                    .enter_delay(0.12 + cascade.delay(index, count))
            }
        };
        list = list.child(entrance(
            row()
                .key(format!("chapter-{index}"))
                .w_full()
                .px(14.0)
                .py(10.0)
                .gap(14.0)
                .center_y()
                .radius(14.0)
                .fill(if on { p.text } else { p.chip })
                .hover(hover_tint(p, on))
                .motion(motion::SNAPPY)
                .press_scale(0.99)
                .role("button")
                .label((t().chapter_described)(&title, &a, &b))
                .focusable(true)
                .focus_ring(2.0, 2.0, p.text)
                .child(
                    text(title)
                        .font(face(15.0, 600))
                        .color(if on { p.ink } else { p.text })
                        .grow(1.0)
                        .truncate_fade(),
                )
                .child(
                    text(format!("{a} – {b}"))
                        .font(figures(13.0, 500))
                        .color(if on { p.ink } else { p.text_faint }),
                )
                .on_hover_change(move |s: &mut State, inside, _| {
                    s.chapter_preview = inside.then_some((from, to));
                })
                .on_press(move |s: &mut State| s.select_chapter(from, to)),
        ));
    }
    let card = col()
        .key("chapters-card")
        .transition(Transition::offset(0.0, super::settings::SLIDE))
        .role("dialog")
        .label(t().chapters)
        .w_full()
        .max_w(560.0)
        .px(28.0)
        .py(24.0)
        .gap(16.0)
        .radius(24.0)
        .glass(28.0, p.glass, super::settings::CARD_OPACITY)
        .border(1.0, p.text.alpha(0.12))
        .modal(true, |s: &mut State| s.close_chapters())
        .child(text(t().chapters).font(display(26.0, -0.6)).color(p.text))
        .child(
            text(t().chapters_hint)
                .font(face(14.0, 400))
                .color(p.text_faint),
        )
        .child(chapter_strip(s, p))
        .child(list)
        .child(
            row().w_full().justify_end().child(
                solid_button(t().done, "close-chapters", false, true, p)
                    .on_press(|s: &mut State| s.close_chapters()),
            ),
        );
    el().key("chapters-layer")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .child(
            el().key("chapters-scrim")
                .transition(Transition::opacity())
                .absolute(0.0, 0.0)
                .w_full()
                .h_full()
                .fill(p.bg.alpha(0.55))
                .a11y_hidden(),
        )
        .child(
            el().absolute(0.0, 0.0)
                .w_full()
                .h_full()
                .padding(24.0)
                .center()
                .child(card),
        )
}
