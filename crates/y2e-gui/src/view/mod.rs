use rune::tok::motion;
use rune::{
    col, el, spacer, text, Beat, Color, Cx, El, Sequence, SurfaceFit, TextStyle, Transition,
};
use rune_kit::Kit;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::gfx;
use crate::i18n::t;
use crate::report::explain;
use crate::state::State;
use crate::theme::{palette, Palette};

mod chapters;
mod dock;
mod landing;
mod settings;
mod studio;
mod timeline;

use chapters::{chapter_marks, chapter_preview, chapters_button, chapters_overlay};
use dock::{dock, notice};
use landing::{hero, logo, url_pill};
use settings::{settings_button, settings_overlay};
use studio::{title_block, top_bar};
use timeline::timeline;

/// Set while a card covers the screen. Rune blurs at most four elements in a
/// frame, in drawing order. The studio's three (URL field, settings button, dock)
/// plus the card fill them, so a banner's blur is skipped while a card is open.
static CARD_OPEN: AtomicBool = AtomicBool::new(false);

/// Set in the performance mode, which leaves out the blur behind the glass.
static LIGHT: AtomicBool = AtomicBool::new(false);

trait Frosted {
    /// `backdrop_blur`, skipped while a card is open.
    fn frosted(self, sigma: f32) -> Self;
    /// A glass pane: blurred and tinted, or in the performance mode just more
    /// opaque, so the text stays readable without the blur.
    fn glass(self, sigma: f32, tint: Color, alpha: f32) -> Self;
    /// `glass` for a card itself, which is blurred even while it is the one open.
    fn card_glass(self, sigma: f32, tint: Color, alpha: f32) -> Self;
}

impl Frosted for El {
    fn frosted(self, sigma: f32) -> Self {
        if CARD_OPEN.load(Ordering::Relaxed) || LIGHT.load(Ordering::Relaxed) {
            self
        } else {
            self.backdrop_blur(sigma)
        }
    }

    fn glass(self, sigma: f32, tint: Color, alpha: f32) -> Self {
        if CARD_OPEN.load(Ordering::Relaxed) {
            // Behind a card the pane is hidden by the scrim; its blur would use up
            // one of the four the card needs.
            self.fill(tint.alpha(alpha))
        } else {
            self.card_glass(sigma, tint, alpha)
        }
    }

    fn card_glass(self, sigma: f32, tint: Color, alpha: f32) -> Self {
        if LIGHT.load(Ordering::Relaxed) {
            self.fill(tint.alpha(alpha.max(0.97)))
        } else {
            self.backdrop_blur(sigma).fill(tint.alpha(alpha))
        }
    }
}

/// The overlay a control shows under the pointer: darker on a light fill, lighter
/// on a dark one.
fn hover_tint(p: &Palette, light_fill: bool) -> Color {
    if light_fill {
        p.ink.alpha(0.10)
    } else {
        p.text.alpha(0.12)
    }
}

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
/// A slow push-in over the first seconds, then it holds.
const PUSH_IN: &[Beat] = &[Beat::new(0.0).scale(1.0), Beat::new(40.0).scale(1.08)];

fn scene(s: &State, p: &Palette) -> El {
    let mut stage = fill_parent().fill(p.bg).clip_rounded(0.0);

    match (&s.thumbnail, &s.metadata) {
        (Some(art), Some(_)) => {
            let artwork = fill_parent()
                .key(format!("art-{}", s.metadata.as_ref().map_or("", |m| &m.id)))
                .pixels_fit(art, SurfaceFit::Fill)
                .transition(Transition::opacity());
            // The artwork slowly pushes in, so the glass in front of it has something
            // to float over. A still image is enough when motion is reduced.
            stage = stage.child(if s.settings.motion.is_calm() {
                artwork
            } else {
                artwork.sequence(Sequence::new(PUSH_IN))
            });
        }
        _ => {
            stage = stage
                .child(gfx::aurora(
                    s.accent_hue,
                    !s.settings.motion.is_calm() && (!s.settings.pause_when_unfocused || s.focused),
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

pub fn app_view(s: &State, cx: &mut Cx<State>) -> El {
    static FIRST: AtomicBool = AtomicBool::new(true);
    if FIRST.swap(false, Ordering::Relaxed) {
        crate::crash::note("first view built");
    }
    CARD_OPEN.store(s.settings_open || s.chapters_open, Ordering::Relaxed);
    LIGHT.store(s.settings.motion.is_light(), Ordering::Relaxed);
    // Travel becomes a jump and opacity a short fade, as the system's own setting asks.
    if s.settings.motion.is_calm() {
        cx.reduced_motion(true);
    }
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
                    t().export_finished,
                    Some(path.display().to_string()),
                    &p,
                )
                .child(
                    el().h(30.0)
                        .px(14.0)
                        .radius(15.0)
                        .fill(p.chip)
                        .hover(hover_tint(&p, false))
                        .press_scale(0.96)
                        .role("button")
                        .label(t().open_folder)
                        .focusable(true)
                        .focus_ring(2.0, 2.0, p.text)
                        .center()
                        .child(text(t().open_folder).font(face(13.0, 600)).color(p.text))
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
        // Everything that answers the pointer does so with the same spring.
        .hover_motion_within(motion::SNAPPY, motion::SNAPPY)
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
        .child(chapters_overlay(s, &p))
        .child(settings_overlay(s, &p))
}
