//! The landing view: logo, URL field, hero and the card for an unreadable link.

use rune::tok::motion;
use rune::{col, el, row, text, text_input, El, LoopSpec, Transition};

use crate::report::{explain, Report};
use crate::state::State;
use crate::theme::Palette;

use super::*;

pub(super) fn solid_button(label: &str, key: &str, big: bool, busy: bool, p: &Palette) -> El {
    let (fill, ink) = if busy {
        (p.chip, p.text_faint)
    } else {
        (p.text, p.ink)
    };
    el().key(key)
        .h(if big { 44.0 } else { 34.0 })
        .px(if big { 22.0 } else { 16.0 })
        .radius(if big { 22.0 } else { 17.0 })
        .fill(fill)
        .motion(motion::SNAPPY)
        .press_scale(0.96)
        .role("button")
        .label(label)
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .center()
        .child(
            text(label)
                .font(face(if big { 15.0 } else { 14.0 }, 600))
                .color(ink),
        )
}

/// The single URL field, wide in the empty state, a compact pill once a source is loaded.
pub(super) fn url_pill(s: &State, p: &Palette, big: bool) -> El {
    let invalid = s.inspect_error.is_some() && !s.is_loading_metadata;
    let input = text_input(&s.url_input, face(if big { 17.0 } else { 15.0 }, 500))
        .key("url_input")
        .grow(1.0)
        .single_line()
        .label("Lien de la vidéo YouTube")
        .placeholder("Collez un lien YouTube", p.text_faint)
        .on_edit(|s: &mut State| &mut s.url_input)
        .on_submit(|s: &mut State, _| {
            if !s.is_loading_metadata {
                s.trigger_inspect();
            }
        });

    let action = solid_button(
        if s.is_loading_metadata {
            "Lecture…"
        } else {
            "Lire"
        },
        "inspect",
        big,
        s.is_loading_metadata,
        p,
    )
    .on_press(|s: &mut State| {
        if !s.is_loading_metadata {
            s.trigger_inspect();
        }
    });

    let height = if big { 64.0 } else { 48.0 };
    row()
        .key("url-pill")
        .role("group")
        .matched("url-pill")
        .w_full()
        .max_w(if big { 680.0 } else { 560.0 })
        .h(height)
        .pl(if big { 26.0 } else { 20.0 })
        .pr(if big { 10.0 } else { 7.0 })
        .center_y()
        .gap(12.0)
        .radius(height / 2.0)
        .backdrop_blur(18.0)
        .fill(p.glass.alpha(0.5))
        .border(
            1.0,
            if invalid {
                p.danger.alpha(0.8)
            } else {
                p.glass_line
            },
        )
        .motion(motion::SNAPPY)
        // The whole pill is a click target for the field, not just its text.
        .on_press_tx(|_: &mut State, tx| tx.focus("url_input", false))
        .child(input)
        .child(action)
}

pub(super) fn logo(p: &Palette, size: f32) -> El {
    row()
        .gap(10.0)
        .center_y()
        .child(
            el().size(size, size)
                .radius(size * 0.32)
                .fill(p.accent)
                .center()
                .child(text("y").font(face(size * 0.62, 800)).color(p.on_accent)),
        )
        .child(text("y2e").font(display(size * 0.72, -0.4)).color(p.text))
}

/// The landing view's answer to a source that could not be read.
pub(super) fn error_card(report: &Report, p: &Palette) -> El {
    let mut details = col().grow(1.0).min_w(0.0).gap(2.0).child(
        text(report.headline)
            .font(face(17.0, 600))
            .color(p.text)
            .wrap(),
    );
    if let Some(cause) = &report.cause {
        details = details.child(
            text(cause.clone())
                .font(face(13.0, 400))
                .color(p.text_faint)
                .wrap()
                .max_lines(2),
        );
    }
    row()
        .key("hero-error")
        .transition(Transition::offset(0.0, 16.0).combined(Transition::opacity()))
        .w_full()
        .max_w(680.0)
        .px(20.0)
        .py(14.0)
        .gap(16.0)
        .center_y()
        .radius(20.0)
        .backdrop_blur(18.0)
        .fill(p.glass.alpha(0.6))
        .border(1.0, p.danger.alpha(0.5))
        .child(
            el().size(36.0, 36.0)
                .radius(18.0)
                .fill(p.danger.alpha(0.18))
                .center()
                .child(text("!").font(face(19.0, 700)).color(p.danger)),
        )
        .child(details)
}

pub(super) fn hero(s: &State, p: &Palette) -> El {
    let mut stack = col()
        .key("hero")
        .transition(Transition::blur(10.0))
        .w_full()
        .grow(1.0)
        .center()
        .gap(30.0)
        .child(logo(p, 34.0))
        .child(
            col()
                .center()
                .gap(0.0)
                .child(text("Une vidéo.").font(display(74.0, -2.6)).color(p.text))
                .child(
                    text("Tous les formats.")
                        .font(display(74.0, -2.6))
                        .color(p.text_dim),
                ),
        )
        .child(url_pill(s, p, true));

    if s.is_loading_metadata {
        stack = stack.child(
            row()
                .key("hero-loading")
                .transition(Transition::opacity())
                .gap(14.0)
                .center_y()
                .child(
                    // Indeterminate: a segment sweeps along the track until the source arrives.
                    row()
                        .w(150.0)
                        .h(4.0)
                        .radius(2.0)
                        .clip_rounded(2.0)
                        .fill(p.chip)
                        .child(
                            el().w(54.0)
                                .h_full()
                                .radius(2.0)
                                .fill(p.accent)
                                .looping(LoopSpec::slide(1.25, [-1.0, 0.0], [3.0, 0.0])),
                        ),
                )
                .child(
                    text("Lecture de la source")
                        .font(face(15.0, 500))
                        .color(p.text_dim),
                ),
        );
    } else if let Some(error) = &s.inspect_error {
        stack = stack.child(error_card(&explain(error), p));
    } else {
        stack = stack.child(if s.clipboard_link {
            text("Lien détecté dans le presse-papier. Appuyez sur Entrée pour le lire.")
                .font(face(15.0, 600))
                .color(p.accent_hi)
        } else {
            text("MP4 · MP3 · Discord · GIF — et seulement le passage qui vous intéresse.")
                .font(face(15.0, 500))
                .color(p.text_faint)
        });
    }
    stack
}
