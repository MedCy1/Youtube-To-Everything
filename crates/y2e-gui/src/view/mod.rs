use rune::{col, el, spacer, text, Color, Cx, El, SurfaceFit, TextStyle, Transition};
use rune_kit::Kit;
use std::f32::consts::PI;

use crate::gfx;
use crate::report::explain;
use crate::state::State;
use crate::theme::{palette, Palette};

mod dock;
mod landing;
mod settings;
mod studio;
mod timeline;

use dock::{dock, notice};
use landing::{hero, logo, url_pill};
use settings::{settings_button, settings_overlay};
use studio::{title_block, top_bar};
use timeline::timeline;

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
                .child(gfx::aurora(
                    s.accent_hue,
                    !s.settings.pause_when_unfocused || s.focused,
                ))
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
                    "Export finished",
                    Some(path.display().to_string()),
                    &p,
                )
                .child(
                    el().h(30.0)
                        .px(14.0)
                        .radius(15.0)
                        .fill(p.chip)
                        .press_scale(0.96)
                        .role("button")
                        .label("Open folder")
                        .focusable(true)
                        .focus_ring(2.0, 2.0, p.text)
                        .center()
                        .child(text("Open folder").font(face(13.0, 600)).color(p.text))
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
        // A link copied elsewhere is offered as soon as the window is back in front.
        .on_window_focus(|s: &mut State, focused, tx| {
            s.focused = focused;
            if focused && !s.showing_source() && !s.is_loading_metadata {
                tx.read_clipboard(|s: &mut State, content, _| {
                    s.offer_clipboard_link(content.text());
                });
            }
        })
        .child(scene(s, &p))
        .child(fill_parent().child(layout.w_full().h_full()))
        .child(settings_button(&p))
        .child(settings_overlay(s, &p))
}
