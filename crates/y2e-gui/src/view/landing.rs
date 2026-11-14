//! The landing view: logo, URL field, hero and the card for an unreadable link.

use rune::tok::motion;
use rune::{col, el, row, text, text_input, El, LiveRegion, LoopSpec, Transition};

use crate::i18n::t;
use crate::report::{explain, install_problem, missing_tools, updates, Report};
use crate::state::State;
use crate::theme::Palette;

use super::*;
use y2e_core::tools::Stage;

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
        .hover(hover_tint(p, !busy))
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
    // Without the tools there is nothing to read: the field is shown, but dimmed and inert.
    let locked = !s.tools_ready();
    let input = text_input(&s.url_input, face(if big { 17.0 } else { 15.0 }, 500))
        .key("url_input")
        .grow(1.0)
        // Without this the field is as wide as what is typed in it, and pushes the
        // button out of the bar.
        .min_w(0.0)
        .single_line()
        .label(t().link_label)
        .disabled(locked)
        .placeholder(t().link_placeholder, p.text_faint)
        .on_edit(|s: &mut State| &mut s.url_input)
        .on_submit(|s: &mut State, _| {
            if !s.is_loading_metadata {
                s.trigger_inspect();
            }
        });

    let action = solid_button(
        if s.is_loading_metadata {
            t().loading
        } else {
            t().load
        },
        "inspect",
        big,
        s.is_loading_metadata || locked,
        p,
    )
    .disabled(locked)
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
        .glass(18.0, p.glass, 0.5)
        .border(
            1.0,
            if invalid {
                p.danger.alpha(0.8)
            } else {
                p.glass_line
            },
        )
        .motion(motion::SNAPPY)
        .opacity(if locked { 0.45 } else { 1.0 })
        // A click anywhere on the pill focuses the field.
        .on_press_tx(|s: &mut State, tx| {
            if s.tools_ready() {
                tx.focus("url_input", false);
            }
        })
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
    notice_card("hero-error", "!", p.danger, report, p)
}

/// A rounded card with a coloured badge, a headline and an optional second line.
fn notice_card(key: &str, badge: &str, tint: Color, report: &Report, p: &Palette) -> El {
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
        .key(key.to_string())
        .live_region(LiveRegion::Polite)
        .transition(Transition::offset(0.0, 16.0).combined(Transition::opacity()))
        .w_full()
        .max_w(680.0)
        .px(20.0)
        .py(14.0)
        .gap(16.0)
        .center_y()
        .radius(20.0)
        .glass(18.0, p.glass, 0.6)
        .border(1.0, tint.alpha(0.5))
        .child(
            el().size(36.0, 36.0)
                .radius(18.0)
                .fill(tint.alpha(0.18))
                .center()
                .child(text(badge.to_string()).font(face(19.0, 700)).color(tint)),
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
                .child(text(t().hero_first).font(display(74.0, -2.6)).color(p.text))
                .child(
                    text(t().hero_second)
                        .font(display(74.0, -2.6))
                        .color(p.text_dim),
                ),
        )
        .child(url_pill(s, p, true));

    if s.is_loading_metadata {
        stack = stack.child(loading_row("hero-loading", t().fetching_details, None, p));
    } else if s.installing {
        let label = match &s.install_stage {
            Some(Stage::Downloading {
                tool,
                number,
                total,
            }) => (t().step_downloading)(tool, *number, *total),
            Some(Stage::Verifying(tool)) => (t().step_verifying)(tool),
            Some(Stage::Installing(tool)) => (t().step_installing)(tool),
            None => t().preparing.to_string(),
        };
        stack = stack.child(loading_row("hero-install", &label, s.install_fraction, p));
    } else if !s.missing_tools.is_empty() {
        let can_install = s
            .missing_tools
            .iter()
            .all(|t| y2e_core::tools::installable(t));
        let failed = s.install_error.is_some();
        let report = match &s.install_error {
            Some(problem) => Report {
                headline: t().install_failed,
                cause: Some(install_problem(problem)),
            },
            None => missing_tools(&s.missing_tools, can_install),
        };
        stack = stack.child(error_card(&report, p));
        if can_install {
            stack = stack.child(
                solid_button(
                    if failed {
                        t().try_again
                    } else {
                        t().install_auto
                    },
                    "install-tools",
                    true,
                    false,
                    p,
                )
                .default_button()
                .on_press(|s: &mut State| s.install_tools()),
            );
        }
    } else if let Some(error) = &s.inspect_error {
        stack = stack.child(error_card(&explain(error), p));
    } else if !s.updates.is_empty() && !s.updates_dismissed {
        stack = stack.child(notice_card(
            "hero-update",
            "↑",
            p.accent,
            &updates(&s.updates),
            p,
        ));
        // Only what the app installed can be updated from here.
        let can_update = s.updates.iter().any(|u| u.managed);
        let mut actions = row().gap(12.0);
        if can_update {
            actions = actions.child(
                solid_button(t().update, "update-tools", true, false, p)
                    .default_button()
                    .on_press(|s: &mut State| s.update_tools()),
            );
        }
        let dismiss = solid_button(
            if can_update { t().later } else { t().got_it },
            "dismiss-update",
            true,
            can_update,
            p,
        )
        .on_press(|s: &mut State| s.updates_dismissed = true);
        // Alone, it answers Return.
        stack = stack.child(actions.child(if can_update {
            dismiss
        } else {
            dismiss.default_button()
        }));
    } else if let Some(update) = s.app_update.as_ref().filter(|_| !s.app_update_dismissed) {
        let report = Report {
            headline: t().app_update_headline,
            cause: Some((t().app_update_cause)(
                &update.version,
                env!("CARGO_PKG_VERSION"),
            )),
        };
        stack = stack.child(notice_card("hero-app-update", "↑", p.accent, &report, p));
        stack = stack.child(
            row()
                .gap(12.0)
                .child(
                    solid_button(t().see_whats_new, "open-app-update", true, false, p)
                        .default_button()
                        .on_press(|s: &mut State| s.open_app_update()),
                )
                .child(
                    solid_button(t().later, "dismiss-app-update", true, true, p)
                        .on_press(|s: &mut State| s.app_update_dismissed = true),
                ),
        );
    } else {
        stack = stack.child(if s.clipboard_link {
            text(t().clipboard_link)
                .font(face(15.0, 600))
                .color(p.accent_hi)
        } else {
            text(t().tagline).font(face(15.0, 500)).color(p.text_faint)
        });
    }
    stack
}

/// An indeterminate bar with a caption, for work whose length is not known.
/// `fraction`: how far along the work is, from 0 to 1. Without it a segment
/// sweeps along the track until the work is done.
fn loading_row(key: &str, label: &str, fraction: Option<f32>, p: &Palette) -> El {
    row()
        .key(key.to_string())
        .transition(Transition::opacity())
        .gap(14.0)
        .center_y()
        .child(
            row()
                .w(150.0)
                .h(4.0)
                .radius(2.0)
                .clip_rounded(2.0)
                .fill(p.chip)
                .child(match fraction {
                    Some(f) => el()
                        .w(150.0 * f.clamp(0.0, 1.0))
                        .h_full()
                        .radius(2.0)
                        .fill(p.accent),
                    None => el()
                        .w(54.0)
                        .h_full()
                        .radius(2.0)
                        .fill(p.accent)
                        .looping(LoopSpec::slide(1.25, [-1.0, 0.0], [3.0, 0.0])),
                }),
        )
        .child(
            text(label.to_string())
                .font(face(15.0, 500))
                .color(p.text_dim),
        )
}
