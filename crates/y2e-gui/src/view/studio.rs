//! The studio header over the artwork: URL pill and title block.

use rune::{col, el, row, spacer, text, El, Transition};

use crate::i18n::t;
use crate::state::State;
use crate::theme::Palette;

use super::*;

pub(super) fn top_bar(s: &State, p: &Palette) -> El {
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

pub(super) fn title_block(s: &State, p: &Palette) -> El {
    let Some(meta) = &s.metadata else {
        return el();
    };
    let uploader = meta.uploader.as_deref().unwrap_or(t().unknown_author);
    let length = if meta.duration_seconds() > 0 {
        (t().video_loaded)(&meta.formatted_duration())
    } else {
        t().video_loaded_plain.to_string()
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
