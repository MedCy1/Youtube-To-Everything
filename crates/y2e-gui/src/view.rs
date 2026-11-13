use rune::tok::motion;
use rune::{
    col, el, row, spacer, text, text_input, Beat, Color, Cx, El, LoopSpec, Sequence, SurfaceFit,
    TextStyle, Transition,
};
use rune_kit::{Kit, RollingText};
use std::f32::consts::PI;
use y2e_core::{format_time_seconds, DownloadPreset};

use crate::gfx;
use crate::report::{explain, Report};
use crate::state::State;
use crate::theme::{palette, Palette};

const WAVE_BARS: usize = 72;

fn face(size: f32, weight: u16) -> TextStyle {
    TextStyle::new(size, weight.into())
}

/// Display type: heavy weight, pulled tight.
fn display(size: f32, tracking: f32) -> TextStyle {
    let mut style = face(size, 700);
    style.tracking = tracking;
    style
}

fn figures(size: f32, weight: u16) -> TextStyle {
    let mut style = face(size, weight);
    style.features = "'tnum' 1";
    style
}

fn fill_parent() -> El {
    el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
}

fn clear(color: Color) -> Color {
    color.alpha(0.0)
}

/// The backdrop of the whole window: the source artwork once it has arrived,
/// otherwise a GPU mesh-gradient aurora in the interface's own hue.
fn scene(s: &State, p: &Palette) -> El {
    let mut stage = fill_parent().fill(p.bg).clip_rounded(0.0);

    match (&s.thumbnail, &s.metadata) {
        (Some(art), Some(_)) => {
            stage = stage.child(
                fill_parent()
                    .pixels_fit(art, SurfaceFit::Fill)
                    .transition(Transition::opacity()),
            );
        }
        _ => {
            stage = stage
                .child(gfx::aurora(s.accent_hue))
                .child(fill_parent().radial_gradient(
                    [0.5, 0.46],
                    [560.0, 330.0],
                    [(0.0, p.bg.alpha(0.5)), (1.0, clear(p.bg))],
                ));
        }
    }

    // Light artwork needs a stronger scrim for the white text to stay readable.
    let k = (0.75 + (s.art_lightness - 0.35) * 2.4).clamp(0.75, 1.9);
    let a = |alpha: f32| p.bg.alpha((alpha * k).min(0.92));
    stage.child(fill_parent().linear_gradient(
        PI,
        [
            (0.00, a(0.62)),
            (0.16, a(0.18)),
            (0.30, a(0.0)),
            (0.50, a(0.22)),
            (0.66, a(0.52)),
            (0.80, a(0.48)),
            (1.00, a(0.30)),
        ],
    ))
}

fn solid_button(label: &str, key: &str, big: bool, busy: bool, p: &Palette) -> El {
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
        .center()
        .child(
            text(label)
                .font(face(if big { 15.0 } else { 14.0 }, 600))
                .color(ink),
        )
}

/// The single URL field, wide in the empty state, a compact pill once a source is loaded.
fn url_pill(s: &State, p: &Palette, big: bool) -> El {
    let invalid = s.inspect_error.is_some() && !s.is_loading_metadata;
    let input = text_input(&s.url_input, face(if big { 17.0 } else { 15.0 }, 500))
        .key("url_input")
        .grow(1.0)
        .single_line()
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

fn logo(p: &Palette, size: f32) -> El {
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
fn error_card(report: &Report, p: &Palette) -> El {
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
        .child(
            col()
                .grow(1.0)
                .gap(2.0)
                .child(text(report.headline).font(face(17.0, 600)).color(p.text))
                .child(
                    text(report.cause.clone())
                        .font(face(13.0, 400))
                        .color(p.text_faint)
                        .truncate_fade(),
                ),
        )
}

fn hero(s: &State, p: &Palette) -> El {
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
        stack = stack.child(
            text("MP4 · MP3 · Discord · GIF — et seulement le passage qui vous intéresse.")
                .font(face(15.0, 500))
                .color(p.text_faint),
        );
    }
    stack
}

fn top_bar(s: &State, p: &Palette) -> El {
    row()
        .key("top-bar")
        .transition(Transition::opacity())
        .w_full()
        .center_y()
        .child(el().w(130.0).child(logo(p, 28.0)))
        .child(spacer())
        .child(url_pill(s, p, false))
        .child(spacer())
        .child(el().w(130.0))
}

fn title_block(s: &State, p: &Palette) -> El {
    let Some(meta) = &s.metadata else {
        return el();
    };
    let uploader = meta.uploader.as_deref().unwrap_or("Auteur inconnu");
    let length = if meta.duration_seconds() > 0 {
        format!("Source chargée · {}", meta.formatted_duration())
    } else {
        "Source chargée".to_string()
    };
    col()
        .key("title")
        .transition(Transition::offset(0.0, 28.0).combined(Transition::opacity()))
        .w_full()
        .gap(10.0)
        .child(
            row()
                .self_start()
                .h(28.0)
                .px(12.0)
                .gap(8.0)
                .center_y()
                .radius(14.0)
                .backdrop_blur(12.0)
                .fill(p.glass.alpha(0.55))
                .child(el().size(8.0, 8.0).radius(4.0).fill(p.accent_hi))
                .child(text(length).font(figures(14.0, 500)).color(p.text)),
        )
        .child(
            text(meta.title.clone())
                .font(display(46.0, -1.4))
                .color(p.text)
                .wrap()
                .max_lines(2),
        )
        .child(text(uploader).font(face(17.0, 500)).color(p.text_dim))
}

/// Waveform-style bars: pure texture, deterministic from the video id.
fn bar_height(index: usize, seed: f32) -> f32 {
    let i = index as f32;
    let a = (i * 0.61 + seed).sin();
    let b = (i * 0.17 + seed * 1.7).sin();
    8.0 + 40.0 * (0.25 + 0.75 * (a * b).abs())
}

fn timeline(s: &State, p: &Palette, k: &Kit) -> El {
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
fn scrub_marks(s: &State, p: &Palette) -> El {
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
fn grip(p: &Palette, left: bool) -> El {
    let handle = el().w(6.0).radius(3.0).fill(p.text);
    if left {
        handle.pin(Some(-4.0), Some(16.0), None, Some(16.0))
    } else {
        handle.pin(None, Some(16.0), Some(-4.0), Some(16.0))
    }
}

fn preset_chip(
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
fn completion_ring(s: &State, p: &Palette) -> El {
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

fn export_button(s: &State, p: &Palette, k: &Kit) -> El {
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
        .on_press(|s: &mut State| {
            if !s.is_downloading {
                s.trigger_download();
            }
        });

    if s.is_downloading {
        button
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
                            text(if s.speed_str != "-" && s.eta_str != "-" {
                                format!("{} · {}", s.speed_str.trim(), s.eta_str.trim())
                            } else {
                                String::new()
                            })
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
                        text("Exporter")
                            .font(display(19.0, -0.3))
                            .color(p.on_accent),
                    )
                    .child(text("↓").font(face(20.0, 700)).color(p.on_accent)),
            ),
        )
    }
}

fn notice(key: &str, color: Color, label: &str, detail: String, p: &Palette) -> El {
    row()
        .key(key)
        .transition(Transition::offset(0.0, 24.0).combined(Transition::opacity()))
        .w_full()
        .px(18.0)
        .py(12.0)
        .gap(12.0)
        .center_y()
        .radius(16.0)
        .backdrop_blur(16.0)
        .fill(p.glass)
        .border(1.0, color.alpha(0.55))
        .child(el().size(8.0, 8.0).radius(4.0).fill(color))
        .child(text(label.to_string()).font(face(15.0, 600)).color(color))
        .child(
            text(detail)
                .font(face(14.0, 400))
                .color(p.text_dim)
                .truncate_fade(),
        )
}

/// Human size: megabytes below a gigabyte, gigabytes above.
fn format_size(mb: f64) -> String {
    if mb >= 1024.0 {
        format!("{:.1} Go", mb / 1024.0).replace('.', ",")
    } else {
        format!("{:.0} Mo", mb.max(1.0))
    }
}

/// What the file will weigh. Never blocks anything: a huge GIF stays possible,
/// the interface only points at the way out.
fn size_hint(s: &State, p: &Palette) -> El {
    let seconds = s.selected_seconds();
    let Some(estimate) = s.preset.estimated_size_mb(seconds).filter(|_| seconds > 0) else {
        return el();
    };
    let prefix = if s.preset == DownloadPreset::Discord25Mb {
        "≤"
    } else {
        "≈"
    };
    text(format!("{prefix} {}", format_size(estimate)))
        .font(figures(15.0, 600))
        .color(p.text_dim)
}

fn gif_advice(s: &State, p: &Palette) -> El {
    let seconds = s.selected_seconds();
    let heavy = s.preset == DownloadPreset::GifMeme
        && s.preset
            .estimated_size_mb(seconds)
            .is_some_and(|mb| mb > 40.0);
    if !heavy {
        return el();
    }
    let chip = |label: &'static str| {
        el().h(30.0)
            .px(14.0)
            .radius(15.0)
            .fill(p.chip)
            .press_scale(0.96)
            .center()
            .child(text(label).font(face(13.0, 600)).color(p.text))
    };
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
            text(format!(
                "Un GIF de {} pèsera lourd. Un extrait court se partage mieux.",
                format_time_seconds(seconds)
            ))
            .font(face(14.0, 500))
            .color(p.text)
            .truncate_fade()
            .grow(1.0),
        )
        .child(chip("Garder 8 s").on_press(|s: &mut State| s.keep_short_clip(8)))
        .child(
            chip("Passer en MP4").on_press(|s: &mut State| s.preset = DownloadPreset::MaxQuality),
        )
}

fn dock(s: &State, p: &Palette, k: &Kit) -> El {
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
        .child(gif_advice(s, p))
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
                            "Qualité max",
                        ))
                        .child(preset_chip(s, p, DownloadPreset::AudioMp3, "MP3", "Audio"))
                        .child(preset_chip(
                            s,
                            p,
                            DownloadPreset::Discord25Mb,
                            "< 25 Mo",
                            "Discord",
                        ))
                        .child(preset_chip(s, p, DownloadPreset::GifMeme, "GIF", "Boucle")),
                )
                .child(
                    row()
                        .grow(1.0)
                        .gap(14.0)
                        .center_y()
                        .justify_end()
                        .child(size_hint(s, p))
                        .child(
                            el().child(gfx::glow(
                                s.accent_hue,
                                if s.is_downloading { 0.1 } else { 0.22 },
                                7.0,
                            ))
                            .child(export_button(s, p, k))
                            .child(completion_ring(s, p)),
                        ),
                ),
        )
}

pub fn app_view(s: &State, _: &mut Cx<State>) -> El {
    let p = palette(s.accent_hue);
    let k = Kit::neutral(true);

    let mut layout = col().padding(24.0).gap(18.0);

    if !s.showing_source() {
        layout = layout.child(hero(s, &p));
    } else {
        layout = layout
            .child(top_bar(s, &p))
            .child(spacer())
            .child(title_block(s, &p));
        if let Some(path) = &s.downloaded_file {
            layout = layout.child(
                notice(
                    "notice-done",
                    p.success,
                    "Export terminé",
                    path.display().to_string(),
                    &p,
                )
                .child(
                    el().h(30.0)
                        .px(14.0)
                        .radius(15.0)
                        .fill(p.chip)
                        .press_scale(0.96)
                        .center()
                        .child(
                            text("Ouvrir le dossier")
                                .font(face(13.0, 600))
                                .color(p.text),
                        )
                        .on_press(|s: &mut State| s.reveal_download()),
                ),
            );
        }
        if let Some(error) = &s.download_error {
            let report = explain(error);
            layout = layout.child(notice(
                "notice-error",
                p.danger,
                report.headline,
                report.cause,
                &p,
            ));
        }
        layout = layout.child(dock(s, &p, &k));
    }

    el().w_full()
        .h_full()
        .fill(p.bg)
        .child(scene(s, &p))
        .child(fill_parent().child(layout.w_full().h_full()))
}
