//! The settings window: a glass card over the scrim.

use rune::{col, el, row, text, El, FileDialogOptions, Tx};
use y2e_core::DownloadPreset;

use crate::state::State;
use crate::theme::Palette;

use super::landing::solid_button;
use super::*;

/// The button that opens it, in the top right corner of every screen.
pub(super) fn settings_button(p: &Palette) -> El {
    el().key("open-settings")
        .pin(None, Some(31.0), Some(24.0), None)
        .h(34.0)
        .px(16.0)
        .radius(17.0)
        .backdrop_blur(12.0)
        .fill(p.glass.alpha(0.55))
        .center()
        .press_scale(0.96)
        .role("button")
        .label("Settings")
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .child(text("Settings").font(face(14.0, 600)).color(p.text))
        .on_press(|s: &mut State| s.settings_open = true)
}

/// The scrim and the card, when settings are open.
pub(super) fn settings_overlay(s: &State, p: &Palette) -> El {
    if !s.settings_open {
        return el();
    }
    let folder = s.settings.export_dir();
    let custom = s.settings.output_dir.is_some();

    let choose = el()
        .key("choose-folder")
        .h(34.0)
        .px(16.0)
        .radius(17.0)
        .fill(p.chip)
        .center()
        .press_scale(0.96)
        .role("button")
        .label("Choose the export folder")
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .child(text("Change…").font(face(14.0, 600)).color(p.text))
        .on_press_tx(|_: &mut State, tx: &mut Tx| {
            let options = FileDialogOptions {
                title: "Export folder".into(),
                ..Default::default()
            };
            tx.open_folder(options, |s: &mut State, result, _| s.folder_chosen(result));
        });
    let mut folder_controls = row().gap(8.0).center_y();
    if custom {
        folder_controls = folder_controls.child(
            el().key("reset-folder")
                .h(34.0)
                .px(14.0)
                .radius(17.0)
                .center()
                .press_scale(0.96)
                .role("button")
                .label("Use the Downloads folder")
                .focusable(true)
                .focus_ring(2.0, 2.0, p.text)
                .child(text("Reset").font(face(14.0, 500)).color(p.text_dim))
                .on_press(|s: &mut State| s.set_export_dir(None)),
        );
    }

    let formats = row()
        .gap(8.0)
        .child(format_chip(s, p, DownloadPreset::MaxQuality, "MP4"))
        .child(format_chip(s, p, DownloadPreset::AudioMp3, "MP3"))
        .child(format_chip(s, p, DownloadPreset::Discord25Mb, "Discord"))
        .child(format_chip(s, p, DownloadPreset::GifMeme, "GIF"));

    let mut card = col()
        .key("settings-card")
        .role("dialog")
        .label("Settings")
        .w_full()
        .max_w(560.0)
        .px(28.0)
        .py(24.0)
        .gap(22.0)
        .radius(24.0)
        .backdrop_blur(28.0)
        .fill(p.glass.alpha(0.88))
        .border(1.0, p.text.alpha(0.12))
        .modal(true, |s: &mut State| s.settings_open = false)
        .child(text("Settings").font(display(26.0, -0.6)).color(p.text))
        .child(setting_row(
            "Export folder",
            folder.display().to_string(),
            folder_controls.child(choose),
            p,
        ))
        .child(setting_row(
            "Default format",
            "Selected when y2e starts.".into(),
            formats,
            p,
        ))
        .child(setting_row(
            "Pause the background",
            "Hold the animated background still while y2e is not in front.".into(),
            switch(s.settings.pause_when_unfocused, p),
            p,
        ));
    if let Some(error) = &s.settings_error {
        card = card.child(
            text(error.clone())
                .font(face(13.0, 500))
                .color(p.danger)
                .wrap(),
        );
    }
    card = card.child(
        row().w_full().justify_end().child(
            solid_button("Done", "close-settings", false, false, p)
                .default_button()
                .on_press(|s: &mut State| s.settings_open = false),
        ),
    );

    el().key("settings-layer")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .child(
            el().key("settings-scrim")
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

/// A name and a hint on the left, the control on the right.
fn setting_row(title: &'static str, hint: String, control: impl Into<El>, p: &Palette) -> El {
    row()
        .w_full()
        .gap(20.0)
        .center_y()
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(3.0)
                .child(text(title).font(face(16.0, 600)).color(p.text))
                .child(
                    text(hint)
                        .font(face(13.0, 400))
                        .color(p.text_faint)
                        .wrap()
                        .max_lines(2),
                ),
        )
        .child(control.into())
}

fn format_chip(s: &State, p: &Palette, preset: DownloadPreset, name: &'static str) -> El {
    let on = s.settings.default_preset == preset;
    el().key(format!("default-{name}"))
        .h(34.0)
        .px(14.0)
        .radius(17.0)
        .fill(if on { p.text } else { p.chip })
        .center()
        .press_scale(0.96)
        .role("button")
        .label(format!("{name} as the default format"))
        .checked(on)
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .child(
            text(name)
                .font(face(14.0, if on { 600 } else { 500 }))
                .color(if on { p.ink } else { p.text }),
        )
        .on_press(move |s: &mut State| s.set_default_preset(preset))
}

/// An on and off switch: a knob in a pill.
fn switch(on: bool, p: &Palette) -> El {
    let knob = el()
        .size(20.0, 20.0)
        .radius(10.0)
        .fill(if on { p.ink } else { p.text });
    let track = row()
        .key("pause-switch")
        .w(46.0)
        .h(26.0)
        .px(3.0)
        .radius(13.0)
        .center_y()
        .fill(if on { p.accent } else { p.chip })
        .role("switch")
        .label("Pause the background when y2e is not in front")
        .checked(on)
        .focusable(true)
        .focus_ring(2.0, 2.0, p.text)
        .press_scale(0.96)
        .on_press(|s: &mut State| s.set_pause_when_unfocused(!s.settings.pause_when_unfocused));
    if on {
        track.justify_end().child(knob)
    } else {
        track.child(knob)
    }
}
