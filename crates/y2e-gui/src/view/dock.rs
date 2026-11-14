//! The glass dock: presets, size hints, the export button and its notices.

use rune::tok::motion;
use rune::{col, el, row, spacer, text, Beat, Color, El, LiveRegion, Sequence, Transition};
use rune_kit::{Kit, RollingText};
use y2e_core::{format_time_seconds, DownloadPreset};

use crate::gfx;
use crate::i18n::t;
use crate::state::State;
use crate::theme::Palette;

use super::*;

pub(super) fn preset_chip(
    s: &State,
    p: &Palette,
    preset: DownloadPreset,
    tag: &'static str,
    name: &'static str,
) -> El {
    let on = s.preset == preset;
    el().h(48.0)
        .px(18.0)
        .radius(14.0)
        .fill(if on { p.text } else { p.chip })
        .motion(motion::SNAPPY)
        .press_scale(0.97)
        .role("button")
        .label(name)
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .center()
        .child(
            row()
                .gap(10.0)
                .center_y()
                .child(text(tag).font(face(12.0, 700)).color(if on {
                    p.accent_deep
                } else {
                    p.text_faint
                }))
                .child(
                    text(name)
                        .font(face(15.0, if on { 600 } else { 500 }))
                        .color(if on { p.ink } else { p.text }),
                ),
        )
        .on_press(move |s: &mut State| s.preset = preset)
}

/// A ring that leaves the button once, each time an export completes.
pub(super) fn completion_ring(s: &State, p: &Palette) -> El {
    const RING: &[Beat] = &[
        Beat::new(0.0).opacity(0.9).scale(1.0),
        Beat::new(0.7).opacity(0.0).scale(1.28),
    ];
    if s.completed == 0 || s.is_downloading {
        return el();
    }
    el().key(format!("ring-{}", s.completed))
        .pin(Some(-2.0), Some(-2.0), Some(-2.0), Some(-2.0))
        .radius(30.0)
        .border(3.0, p.accent_hi)
        .sequence(Sequence::new(RING))
}

/// The line under the percentage: speed and time left once known, and until then
/// a word saying the export has started (the first seconds report nothing).
fn progress_detail(s: &State, progress: f32) -> String {
    let parts: Vec<&str> = [s.speed_str.trim(), s.eta_str.trim()]
        .into_iter()
        .filter(|value| !matches!(*value, "-" | "NA" | "N/A" | ""))
        .collect();
    if !parts.is_empty() {
        parts.join(" · ")
    } else if progress < 1.0 {
        format!("{}…", t().preparing)
    } else {
        String::new()
    }
}

pub(super) fn export_button(s: &State, p: &Palette, k: &Kit) -> El {
    let progress = s.progress_percent.clamp(0.0, 100.0);
    let button = el()
        .key("launch_export")
        .w(200.0)
        .h(56.0)
        .radius(28.0)
        .clip_rounded(28.0)
        .fill(if s.is_downloading { p.chip } else { p.accent })
        .motion(motion::SNAPPY)
        .press_scale(0.97)
        .role(if s.is_downloading {
            "progressbar"
        } else {
            "button"
        })
        .label(if s.is_downloading {
            t().export_progress
        } else {
            t().export
        })
        .shortcut_hint("Enter")
        .focusable(!s.is_downloading)
        .focus_ring(2.0, 2.0, p.text)
        .on_press(|s: &mut State| {
            if !s.is_downloading {
                s.trigger_download();
            }
        });

    // Return presses the export button from anywhere but the URL field.
    let button = if s.is_downloading {
        button
    } else {
        button.default_button()
    };

    if s.is_downloading {
        button
            .range_value(progress as f64, 0.0, 100.0)
            .child(
                el().pin(Some(0.0), None, Some(0.0), Some(0.0))
                    .h(4.0)
                    .child(
                        row()
                            .w_full()
                            .h_full()
                            .child(
                                el().grow(progress)
                                    .h_full()
                                    .fill(p.accent)
                                    .motion(motion::SNAPPY),
                            )
                            .child(el().grow((100.0 - progress).max(0.0))),
                    ),
            )
            .child(
                fill_parent().center().child(
                    col()
                        .center()
                        .gap(0.0)
                        .child(
                            RollingText::new("export-progress", format!("{progress:.0} %"))
                                .value(progress as f64)
                                .font(figures(19.0, 700))
                                .color(p.text)
                                .build(k),
                        )
                        .child(
                            text(progress_detail(s, progress))
                                .font(figures(12.0, 500))
                                .color(p.text_dim),
                        ),
                ),
            )
    } else {
        button.child(
            fill_parent().center().child(
                row()
                    .gap(12.0)
                    .center_y()
                    .child(
                        text(t().export)
                            .font(display(19.0, -0.3))
                            .color(p.on_accent),
                    )
                    .child(text("↓").font(face(20.0, 700)).color(p.on_accent)),
            ),
        )
    }
}

pub(super) fn notice(
    key: &str,
    color: Color,
    label: &str,
    detail: Option<String>,
    p: &Palette,
) -> El {
    let mut banner = row()
        .key(key)
        .live_region(LiveRegion::Polite)
        .transition(Transition::offset(0.0, 24.0).combined(Transition::opacity()))
        .w_full()
        .px(18.0)
        .py(12.0)
        .gap(12.0)
        .center_y()
        .radius(16.0)
        .frosted(16.0)
        .fill(p.glass)
        .border(1.0, color.alpha(0.55))
        .child(el().size(8.0, 8.0).radius(4.0).fill(color))
        .child(text(label.to_string()).font(face(15.0, 600)).color(color));
    banner = match detail {
        Some(detail) => banner.child(
            text(detail)
                .font(face(14.0, 400))
                .color(p.text_dim)
                .truncate_fade()
                .grow(1.0)
                .min_w(0.0),
        ),
        None => banner.child(spacer()),
    };
    banner
}

/// Human size: megabytes below a gigabyte, gigabytes above.
pub(super) fn format_size(mb: f64) -> String {
    let t = t();
    let size = if mb >= 1024.0 {
        format!("{:.1} {}", mb / 1024.0, t.unit_gb)
    } else {
        format!("{:.0} {}", mb.max(1.0), t.unit_mb)
    };
    size.replace('.', &t.decimal.to_string())
}

/// What the file will weigh. Never blocks: a huge GIF stays possible, and the
/// interface only suggests a shorter clip.
fn cancel_chip(p: &Palette) -> El {
    el().h(34.0)
        .px(16.0)
        .radius(17.0)
        .fill(p.chip)
        .center()
        .press_scale(0.96)
        .role("button")
        .label(t().cancel_export)
        .shortcut_hint("Esc")
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .child(text(t().cancel).font(face(14.0, 600)).color(p.text_dim))
        .on_press(|s: &mut State| s.cancel_download())
}

pub(super) fn size_hint(s: &State, p: &Palette) -> El {
    let seconds = s.selected_seconds();
    let Some(estimate) = s.preset.estimated_size_mb(seconds).filter(|_| seconds > 0) else {
        return el();
    };
    let prefix = if s.preset == DownloadPreset::Discord20Mb {
        "≤"
    } else {
        "≈"
    };
    text(format!("{prefix} {}", format_size(estimate)))
        .font(figures(15.0, 600))
        .color(p.text_dim)
}

pub(super) fn gif_advice(s: &State, p: &Palette) -> Option<El> {
    let seconds = s.selected_seconds();
    let heavy = s.preset == DownloadPreset::GifMeme
        && s.preset
            .estimated_size_mb(seconds)
            .is_some_and(|mb| mb > 40.0);
    if !heavy {
        return None;
    }
    let chip = |label: String| {
        el().h(30.0)
            .px(14.0)
            .radius(15.0)
            .fill(p.chip)
            .press_scale(0.96)
            .role("button")
            .label(label.clone())
            .focusable(true)
            .focus_ring(2.0, 2.0, p.text)
            .center()
            .child(text(label).font(face(13.0, 600)).color(p.text))
    };
    Some(
        row()
            .key("gif-advice")
            .transition(Transition::offset(0.0, 12.0).combined(Transition::opacity()))
            .w_full()
            .px(16.0)
            .py(10.0)
            .gap(10.0)
            .center_y()
            .radius(14.0)
            .fill(p.accent_deep.alpha(0.55))
            .child(
                text((t().gif_large)(&format_time_seconds(seconds)))
                    .font(face(14.0, 500))
                    .color(p.text)
                    .truncate_fade()
                    .grow(1.0),
            )
            .child(chip((t().keep_seconds)(8)).on_press(|s: &mut State| s.keep_short_clip(8)))
            .child(
                chip(t().switch_to_mp4.to_string())
                    .on_press(|s: &mut State| s.preset = DownloadPreset::MaxQuality),
            ),
    )
}

pub(super) fn dock(s: &State, p: &Palette, k: &Kit) -> El {
    col()
        .key("dock")
        .transition(Transition::offset(0.0, 72.0).combined(Transition::opacity()))
        .w_full()
        .padding(22.0)
        .gap(20.0)
        .radius(28.0)
        .backdrop_blur(28.0)
        .fill(p.glass.alpha(0.5))
        .border(1.0, p.glass_line)
        .shadow(18.0, 60.0, Color::BLACK.alpha(0.45))
        .child(timeline(s, p, k))
        .children(gif_advice(s, p))
        .child(
            row()
                .w_full()
                .flex_wrap()
                .gap(12.0)
                .center_y()
                .child(
                    row()
                        .flex_wrap()
                        .gap(8.0)
                        .center_y()
                        .child(preset_chip(
                            s,
                            p,
                            DownloadPreset::MaxQuality,
                            "MP4",
                            t().max_quality,
                        ))
                        .child(preset_chip(
                            s,
                            p,
                            DownloadPreset::AudioMp3,
                            "MP3",
                            t().audio,
                        ))
                        .child(preset_chip(
                            s,
                            p,
                            DownloadPreset::Discord20Mb,
                            "< 20 MB",
                            "Discord",
                        ))
                        .child(preset_chip(
                            s,
                            p,
                            DownloadPreset::GifMeme,
                            "GIF",
                            t().loop_gif,
                        )),
                )
                .child(
                    row()
                        .grow(1.0)
                        .gap(14.0)
                        .center_y()
                        .justify_end()
                        .child(if s.is_downloading {
                            cancel_chip(p)
                        } else {
                            size_hint(s, p)
                        })
                        .child(
                            el().on_hover_change(|s: &mut State, inside, _| {
                                s.export_hover = inside
                            })
                            .child(gfx::glow(
                                s.accent_hue,
                                if s.is_downloading { 0.1 } else { 0.22 },
                                7.0,
                                s.export_hover || s.is_downloading,
                            ))
                            .child(export_button(s, p, k))
                            .child(completion_ring(s, p)),
                        ),
                ),
        )
}
