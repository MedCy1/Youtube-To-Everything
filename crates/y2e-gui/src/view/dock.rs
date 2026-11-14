//! The glass dock: presets, size hints, the export button and its notices.

use rune::tok::motion;
use rune::{
    col, el, row, spacer, text, Beat, Color, El, LiveRegion, LoopSpec, Sequence, Transition,
};
use rune_kit::{Kit, RollingText};
use y2e_core::{format_time_seconds, DownloadPreset, ExportStage};

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
        .hover(hover_tint(p, on))
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

/// What the running export is doing, with the time left once it is known.
fn stage_name(s: &State) -> &'static str {
    match s.export_stage {
        ExportStage::Preparing => t().preparing,
        ExportStage::Downloading => t().stage_downloading,
        ExportStage::Compressing => t().stage_compressing,
        ExportStage::Converting => t().stage_converting,
    }
}

/// The line under the percentage: the stage, and the time left once known.
fn stage_detail(s: &State) -> String {
    match s.eta_str.trim() {
        "-" | "NA" | "N/A" | "" => stage_name(s).to_string(),
        eta => format!("{} · {eta}", stage_name(s)),
    }
}

/// The thin bar along the top of the button: filled as far as the stage has got,
/// or a sweeping segment while nothing can be measured yet.
fn stage_bar(s: &State, p: &Palette, progress: f32) -> El {
    let bar = el().pin(Some(0.0), None, Some(0.0), Some(0.0)).h(4.0);
    if s.export_stage == ExportStage::Preparing {
        return bar.child(
            el().w(54.0)
                .h_full()
                .radius(2.0)
                .fill(p.accent)
                .looping(LoopSpec::slide(1.25, [-1.0, 0.0], [4.0, 0.0])),
        );
    }
    bar.child(
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
    )
}

/// The percentage and what is being done, or just the word while preparing.
fn stage_label(s: &State, p: &Palette, k: &Kit, progress: f32) -> El {
    let label = col().center().gap(0.0);
    if s.export_stage == ExportStage::Preparing {
        return fill_parent().center().child(
            label.child(
                text(format!("{}…", t().preparing))
                    .font(face(17.0, 600))
                    .color(p.text),
            ),
        );
    }
    fill_parent().center().child(
        label
            .child(
                RollingText::new("export-progress", format!("{progress:.0} %"))
                    .value(progress as f64)
                    .font(figures(19.0, 700))
                    .color(p.text)
                    .build(k),
            )
            .child(
                text(stage_detail(s))
                    .key(format!("stage-{:?}", s.export_stage))
                    .transition(Transition::opacity())
                    .font(figures(12.0, 500))
                    .color(p.text_dim),
            ),
    )
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
        .hover(if s.is_downloading {
            clear(p.chip)
        } else {
            Color::WHITE.alpha(0.16)
        })
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
            .child(stage_bar(s, p, progress))
            .child(stage_label(s, p, k, progress))
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
        .hover(hover_tint(p, false))
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
    // A fixed width, the widest figure there can be: otherwise the row wraps at
    // a different window width depending on how many digits the size has.
    row().w(SIZE_WIDTH).justify_end().child(
        text(format!("{prefix} {}", format_size(estimate)))
            .font(figures(15.0, 600))
            .color(p.text_dim),
    )
}

/// Room for "≈ 999 MB", the longest estimate (and "≈ 9.9 GB", in either language).
const SIZE_WIDTH: f32 = 72.0;

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
            .hover(hover_tint(p, false))
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
        .glass(28.0, p.glass, 0.5)
        .border(1.0, p.glass_line)
        .shadow(18.0, 60.0, Color::BLACK.alpha(0.45))
        // Keyed, so the advice appearing between them does not make the next one
        // look new: it would enter again from below.
        .child(timeline(s, p, k).key("dock-timeline"))
        .children(gif_advice(s, p))
        .child(
            row()
                .key("dock-actions")
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
                            .children((!s.settings.motion.is_light()).then(|| {
                                gfx::glow(
                                    s.accent_hue,
                                    if s.is_downloading { 0.1 } else { 0.22 },
                                    7.0,
                                    !s.settings.motion.is_calm()
                                        && (s.export_hover || s.is_downloading),
                                )
                            }))
                            .child(export_button(s, p, k))
                            .child(completion_ring(s, p)),
                        ),
                ),
        )
}
