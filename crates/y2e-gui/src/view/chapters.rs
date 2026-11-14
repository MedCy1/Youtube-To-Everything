//! The chapters of the video: a list to pick one from, and the marks that
//! show where they begin on the timeline.

use rune::ScrollSpec;
use rune::{col, el, row, text, El};
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
    for (index, (title, from, to)) in s.chapter_ranges().into_iter().enumerate() {
        let (a, b) = (format_time_seconds(from), format_time_seconds(to));
        let on = current == Some((from, to));
        list = list.child(
            row()
                .key(format!("chapter-{index}"))
                .w_full()
                .px(14.0)
                .py(10.0)
                .gap(14.0)
                .center_y()
                .radius(14.0)
                .fill(if on { p.text } else { p.chip })
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
                .on_press(move |s: &mut State| s.select_chapter(from, to)),
        );
    }
    let card = col()
        .key("chapters-card")
        .role("dialog")
        .label(t().chapters)
        .w_full()
        .max_w(560.0)
        .px(28.0)
        .py(24.0)
        .gap(16.0)
        .radius(24.0)
        .backdrop_blur(28.0)
        .fill(p.glass.alpha(super::settings::CARD_OPACITY))
        .border(1.0, p.text.alpha(0.12))
        .modal(true, |s: &mut State| s.chapters_open = false)
        .child(text(t().chapters).font(display(26.0, -0.6)).color(p.text))
        .child(
            text(t().chapters_hint)
                .font(face(14.0, 400))
                .color(p.text_faint),
        )
        .child(list)
        .child(
            row().w_full().justify_end().child(
                solid_button(t().done, "close-chapters", false, true, p)
                    .on_press(|s: &mut State| s.chapters_open = false),
            ),
        );
    el().key("chapters-layer")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .child(
            el().key("chapters-scrim")
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
