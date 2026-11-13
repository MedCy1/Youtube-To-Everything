//! The waveform cut: drag handles, keyboard nudges and the pointer marks.

use rune::tok::motion;
use rune::{col, el, row, spacer, text, El};
use rune_kit::{Kit, RollingText};
use y2e_core::format_time_seconds;

use crate::state::State;
use crate::theme::Palette;

use super::*;

pub(super) const WAVE_BARS: usize = 72;

/// Waveform-style bars: pure texture, deterministic from the video id.
pub(super) fn bar_height(index: usize, seed: f32) -> f32 {
    let i = index as f32;
    let a = (i * 0.61 + seed).sin();
    let b = (i * 0.17 + seed * 1.7).sin();
    8.0 + 40.0 * (0.25 + 0.75 * (a * b).abs())
}

pub(super) fn timeline(s: &State, p: &Palette, k: &Kit) -> El {
    let total = s.duration().max(1) as f32;
    let (a, b) = if s.enable_snipper {
        (s.start_time_secs as f32, s.end_time_secs as f32)
    } else {
        (0.0, total)
    };
    let (fa, fb) = ((a / total).clamp(0.0, 1.0), (b / total).clamp(0.0, 1.0));
    let seed = s
        .metadata
        .as_ref()
        .map(|m| {
            m.id.bytes()
                .fold(7u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32))
                % 97
        })
        .unwrap_or(3) as f32;

    let mut bars = row().w_full().h(64.0).gap(3.0).center_y();
    for index in 0..WAVE_BARS {
        let at = (index as f32 + 0.5) / WAVE_BARS as f32;
        let inside = at >= fa && at <= fb;
        bars = bars.child(
            el().grow(1.0)
                .h(bar_height(index, seed))
                .radius(2.0)
                .fill(if inside { p.accent_hi } else { p.bar })
                .motion(motion::SNAPPY),
        );
    }

    let selection = el()
        .grow(((fb - fa) * 1000.0).max(8.0))
        .h_full()
        .radius(12.0)
        .border(2.0, p.accent)
        .fill(p.accent.alpha(0.14))
        .motion(motion::SNAPPY)
        .child(grip(p, true))
        .child(grip(p, false));

    let track = el()
        .key("timeline")
        .w_full()
        .h(64.0)
        .child(bars)
        .child(scrub_marks(s, p))
        .child(
            fill_parent().child(
                row()
                    .w_full()
                    .h_full()
                    .child(el().grow((fa * 1000.0).max(0.0)).h_full())
                    .child(selection)
                    .child(el().grow(((1.0 - fb) * 1000.0).max(0.0)).h_full()),
            ),
        )
        .on_layout(|s: &mut State, rect| s.scrub.width = rect[2])
        .on_pointer_move_local(|s: &mut State, event, tx| {
            tx.live_only();
            s.scrub_to(event.x);
        })
        .on_hover_change(|s: &mut State, inside, _| s.scrub.inside = inside)
        .on_drag(|s: &mut State, d, _| {
            let (phase, x, start_x, w) = (d.phase, d.x, d.start_x, d.w);
            s.drag_timeline(phase, x, start_x, w, &mut d.data);
        });

    let summary: El = if s.enable_snipper {
        let length = (s.end_time_secs.saturating_sub(s.start_time_secs)) as f64;
        row()
            .gap(10.0)
            .center_y()
            .child(
                text(format!(
                    "Découpe  {} → {}",
                    format_time_seconds(s.start_time_secs),
                    format_time_seconds(s.end_time_secs)
                ))
                .font(figures(15.0, 600))
                .color(p.text),
            )
            .child(
                RollingText::new("selection-length", format_time_seconds(length as u64))
                    .value(length)
                    .font(figures(15.0, 600))
                    .color(p.accent_hi)
                    .build(k),
            )
            .child(
                el().h(26.0)
                    .px(10.0)
                    .radius(13.0)
                    .fill(p.chip)
                    .center()
                    .press_scale(0.96)
                    .role("button")
                    .label("Sélectionner toute la vidéo")
                    .focusable(true)
                    .focus_ring(2.0, 2.0, p.text)
                    .child(text("Tout").font(face(13.0, 600)).color(p.text_dim))
                    .on_press(|s: &mut State| s.reset_range()),
            )
    } else {
        text("Vidéo entière — glissez sur la forme d’onde pour découper")
            .font(face(15.0, 500))
            .color(p.text_dim)
    };

    col().w_full().gap(10.0).child(track).child(
        row()
            .w_full()
            .center_y()
            .child(text("0:00").font(figures(14.0, 500)).color(p.text_faint))
            .child(spacer())
            .child(summary)
            .child(spacer())
            .child(
                text(if s.duration() > 0 {
                    format_time_seconds(s.duration())
                } else {
                    "—".to_string()
                })
                .font(figures(14.0, 500))
                .color(p.text_faint),
            ),
    )
}

/// Hairline and time label that follow the pointer along the waveform.
pub(super) fn scrub_marks(s: &State, p: &Palette) -> El {
    let shown = if s.scrub.inside && s.duration() > 0 {
        1.0
    } else {
        0.0
    };
    fill_parent()
        .child(
            el().pin(Some(0.0), Some(-6.0), None, Some(-6.0))
                .w(2.0)
                .radius(1.0)
                .fill(p.text)
                .opacity(shown * 0.85)
                .motion(motion::SNAPPY)
                .live_offset(&s.scrub.line),
        )
        .child(
            el().pin(Some(0.0), Some(-46.0), None, None)
                .w(76.0)
                .h(28.0)
                .radius(14.0)
                .fill(p.text)
                .opacity(shown)
                .motion(motion::SNAPPY)
                .center()
                .live_offset(&s.scrub.tip)
                .child(
                    text("0:00:00")
                        .live_text(&s.scrub.label)
                        .font(figures(14.0, 600))
                        .color(p.ink),
                ),
        )
}

/// A cream grab handle on one side of the selection.
pub(super) fn grip(p: &Palette, left: bool) -> El {
    let handle = el().w(6.0).radius(3.0).fill(p.text);
    if left {
        handle.pin(Some(-4.0), Some(16.0), None, Some(16.0))
    } else {
        handle.pin(None, Some(16.0), Some(-4.0), Some(16.0))
    }
}
