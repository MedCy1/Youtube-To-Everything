use rune::tok::{motion, radius, type_};
use rune::{Cx, El, col, el, row, spacer, text, text_input, Color};
use y2e_core::{format_time_seconds, DownloadPreset};

use crate::state::State;
use crate::theme::PALETTE;

pub fn app_view(s: &State, _: &mut Cx<State>) -> El {
    let p = &PALETTE;

    // --- Header ---
    let badge = el()
        .px(8.0)
        .py(2.0)
        .radius(radius::SM)
        .fill(p.accent)
        .child(
            text("y2e")
                .font(type_::LABEL)
                .color(Color::WHITE),
        );

    let header_title = text("YouTube to Everything")
        .font(type_::TITLE)
        .color(p.text_primary);

    let subtitle = text("Convertisseur ultra-rapide et tactile propulsé par Rune")
        .font(type_::LABEL)
        .color(p.text_muted);

    let header = row()
        .w_full()
        .gap(12.0)
        .center_y()
        .child(badge)
        .child(col().child(header_title).child(subtitle));

    // --- Input Bar ---
    let url_field = text_input(&s.url_input, type_::BODY.lh(24.0))
        .key("url_input")
        .single_line()
        .grow(1.0)
        .color(p.text_primary)
        .placeholder("Colle une URL YouTube (ex: https://youtube.com/watch?v=...)", p.text_muted)
        .on_edit(|s: &mut State| &mut s.url_input);

    let inspect_btn = row()
        .key("inspect_btn")
        .px(14.0)
        .py(8.0)
        .radius(radius::MD)
        .fill(if s.is_loading_metadata { p.control_bg } else { p.card })
        .motion(motion::SNAPPY)
        .center_y()
        .child(
            text(if s.is_loading_metadata { "Analyse..." } else { "Inspecter" })
                .font(type_::LABEL)
                .color(p.text_primary),
        )
        .on_press(|s: &mut State| s.trigger_inspect());

    let input_container = row()
        .w_full()
        .px(14.0)
        .py(8.0)
        .radius(radius::MD)
        .fill(p.surface)
        .gap(12.0)
        .center_y()
        .child(url_field)
        .child(inspect_btn);

    // --- Metadata Card (si chargée) ---
    let meta_card = if let Some(meta) = &s.metadata {
        let title_text = text(meta.title.clone())
            .font(type_::TITLE)
            .color(p.text_primary);

        let uploader = meta.uploader.as_deref().unwrap_or("Auteur inconnu");
        let details = text(format!("Chaîne: {}  •  Durée totale: {}", uploader, meta.formatted_duration()))
            .font(type_::LABEL)
            .color(p.text_secondary);

        col()
            .w_full()
            .px(16.0)
            .py(12.0)
            .radius(radius::MD)
            .fill(p.card)
            .gap(6.0)
            .motion(motion::SNAPPY)
            .child(title_text)
            .child(details)
    } else {
        col()
    };

    // --- Precision Snipper Section ---
    let snipper_toggle = row()
        .gap(10.0)
        .center_y()
        .child(
            el()
                .size(16.0, 16.0)
                .radius(8.0)
                .fill(if s.enable_snipper { p.accent } else { p.control_bg })
                .motion(motion::SNAPPY),
        )
        .child(
            text(if s.enable_snipper { "Snipper Temporel Actif" } else { "Mode Snipper Désactivé (Vidéo complète)" })
                .font(type_::BODY)
                .color(if s.enable_snipper { p.text_primary } else { p.text_secondary }),
        )
        .on_press(|s: &mut State| s.enable_snipper = !s.enable_snipper);

    let snipper_controls = if s.enable_snipper {
        let start_str = format_time_seconds(s.start_time_secs);
        let end_str = format_time_seconds(s.end_time_secs);
        let diff_secs = s.end_time_secs.saturating_sub(s.start_time_secs);

        let btn_adjust = |label: &'static str, delta: i64, is_start: bool| {
            row()
                .px(8.0)
                .py(4.0)
                .radius(radius::SM)
                .fill(p.control_bg)
                .center()
                .child(text(label).font(type_::LABEL).color(p.text_primary))
                .on_press(move |s: &mut State| {
                    if is_start {
                        let new_val = (s.start_time_secs as i64 + delta).max(0) as u64;
                        if new_val < s.end_time_secs {
                            s.start_time_secs = new_val;
                        }
                    } else {
                        let max_d = s.metadata.as_ref().map(|m| m.duration_seconds()).unwrap_or(3600);
                        let new_val = ((s.end_time_secs as i64 + delta).max(s.start_time_secs as i64 + 1) as u64).min(max_d);
                        s.end_time_secs = new_val;
                    }
                })
        };

        row()
            .w_full()
            .px(12.0)
            .py(8.0)
            .radius(radius::SM)
            .fill(p.surface)
            .gap(12.0)
            .center_y()
            .child(text("Début:").font(type_::LABEL).color(p.text_muted))
            .child(btn_adjust("-10s", -10, true))
            .child(text(start_str).font(type_::SECTION).color(p.accent))
            .child(btn_adjust("+10s", 10, true))
            .child(spacer())
            .child(text("Fin:").font(type_::LABEL).color(p.text_muted))
            .child(btn_adjust("-10s", -10, false))
            .child(text(end_str).font(type_::SECTION).color(p.accent))
            .child(btn_adjust("+10s", 10, false))
            .child(spacer())
            .child(text(format!("(Extrait: {}s)", diff_secs)).font(type_::LABEL).color(p.text_secondary))
    } else {
        row()
    };

    let snipper_box = col()
        .w_full()
        .px(16.0)
        .py(12.0)
        .radius(radius::MD)
        .fill(p.card)
        .gap(10.0)
        .child(snipper_toggle)
        .child(snipper_controls);

    // --- Presets Bar ---
    let presets = [
        DownloadPreset::MaxQuality,
        DownloadPreset::Discord25Mb,
        DownloadPreset::AudioMp3,
        DownloadPreset::GifMeme,
    ];

    let mut presets_row = row().w_full().gap(10.0);
    for preset in presets {
        let is_selected = s.preset == preset;
        let card = col()
            .grow(1.0)
            .px(12.0)
            .py(10.0)
            .radius(radius::MD)
            .fill(if is_selected { p.surface } else { p.card })
            .motion(motion::SNAPPY)
            .gap(4.0)
            .child(
                text(preset.label())
                    .font(type_::SECTION)
                    .color(if is_selected { p.accent } else { p.text_primary }),
            )
            .child(
                text(preset.description())
                    .font(type_::LABEL)
                    .color(p.text_muted),
            )
            .on_press(move |s: &mut State| s.preset = preset);
        presets_row = presets_row.child(card);
    }

    // --- Download Button ---
    let download_btn = row()
        .key("download_btn")
        .w_full()
        .h(46.0)
        .radius(radius::MD)
        .fill(if s.is_downloading { p.control_bg } else { p.accent })
        .motion(motion::SNAPPY)
        .center()
        .child(
            text(if s.is_downloading { "⚡ TÉLÉCHARGEMENT EN COURS..." } else { "⚡ LANCER LE TÉLÉCHARGEMENT" })
                .font(type_::SECTION)
                .color(Color::WHITE),
        )
        .on_press(|s: &mut State| {
            if !s.is_downloading {
                s.trigger_download();
            }
        });

    // --- Progress & Status ---
    let filled = s.progress_percent.max(0.1);
    let remaining = (100.0 - s.progress_percent).max(0.0);
    let progress_bar = row()
        .key("progress_track")
        .w_full()
        .h(8.0)
        .radius(4.0)
        .fill(p.surface)
        .child(
            el()
                .key("progress_fill")
                .grow(filled)
                .h_full()
                .radius(4.0)
                .fill(p.accent)
                .motion(motion::SNAPPY),
        )
        .child(
            el()
                .key("progress_empty")
                .grow(remaining)
                .h_full(),
        );

    let status_line = row()
        .w_full()
        .gap(8.0)
        .center_y()
        .child(text(s.status_text.clone()).font(type_::BODY).color(p.text_secondary))
        .child(spacer())
        .child(text(format!("{:.1}%", s.progress_percent)).font(type_::SECTION).color(p.text_primary))
        .child(text(format!("• {}", s.speed_str)).font(type_::LABEL).color(p.text_muted))
        .child(text(format!("• ETA {}", s.eta_str)).font(type_::LABEL).color(p.text_muted));

    let mut footer = col().w_full().gap(8.0).child(status_line).child(progress_bar);

    if let Some(path) = &s.downloaded_file {
        let msg = format!("✓ Fichier disponible : {}", path.display());
        footer = footer.child(text(msg).font(type_::SECTION).color(p.success));
    }

    if let Some(err) = &s.download_error {
        let msg = format!("Erreur téléchargement : {}", err);
        footer = footer.child(text(msg).font(type_::SECTION).color(p.danger));
    }

    let inspect_err_view = if let Some(err) = &s.inspect_error {
        text(format!("Erreur inspection : {}", err)).font(type_::LABEL).color(p.danger)
    } else {
        text("")
    };

    // --- Container Principal ---
    col()
        .w_full()
        .h_full()
        .fill(p.bg)
        .padding(20.0)
        .gap(16.0)
        .child(header)
        .child(input_container)
        .child(inspect_err_view)
        .child(meta_card)
        .child(snipper_box)
        .child(presets_row)
        .child(download_btn)
        .child(spacer())
        .child(footer)
}
