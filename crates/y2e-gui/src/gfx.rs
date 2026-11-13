//! GPU effects drawn by `rune-fx` into effect slots of the view: the landing
//! aurora (a shader mesh gradient) and the glow behind the export button.
//! Slots carry their parameters in `effect_data`; nothing here touches state.

use rune::color::oklch;
use rune::compose::{Compositor, Frame};
use rune::render::{EffectPhase, EffectSite, KIND_EFFECT};
use rune::El;
use rune_fx::*;

use crate::state::State;

const AURORA: u32 = 1;
const GLOW: u32 = 2;

/// An effect slot. `hue` is the interface's hue, so effects follow the artwork.
pub fn aurora(hue: f32) -> El {
    rune::effect(AURORA).effect_data([hue, 0.0, 0.0, 1.0]).pin(
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
    )
}

/// `moving` lets the aura flow; otherwise it rests in a fixed pose and costs no frames.
pub fn glow(hue: f32, intensity: f32, radius: f32, moving: bool) -> El {
    rune::effect(GLOW)
        .effect_data([hue, intensity, radius, f32::from(moving)])
        .effect_bleed(radius * 2.0)
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .radius(28.0)
}

fn srgb(l: f32, c: f32, h: f32) -> u32 {
    let hex = oklch(l, c, h).hex();
    u32::from_str_radix(&hex[1..7], 16).unwrap_or(0)
}

fn layer(kind: u32, d: [f32; 4], shape: Shape, t: f32) -> Option<FxLayer> {
    let hue = d[0];
    let fx = match kind {
        AURORA => Fx::fill(MeshGradientParams {
            colors: Stops::hex(&[
                srgb(0.2, 0.06, hue + 270.0),
                srgb(0.72, 0.19, hue),
                srgb(0.83, 0.13, hue + 28.0),
                srgb(0.4, 0.15, hue + 270.0),
                srgb(0.3, 0.1, hue),
            ]),
            speed: 0.18,
            drift: 0.16,
            ..MeshGradientParams::dusk()
        }),
        GLOW => Fx::glow(GlowParams {
            style: GlowStyle::Aura,
            // The aura flows between three colours: keep them neighbours on
            // the hue wheel so it stays one family with the interface.
            color: rgb(srgb(0.72, 0.19, hue)),
            core: rgb(srgb(0.82, 0.14, hue + 12.0)),
            third: rgb(srgb(0.8, 0.15, hue + 32.0)),
            breathe: if d[3] > 0.5 { 0.2 } else { 0.0 },
            intensity: d[1],
            radius: d[2],
            aura_speed: if d[3] > 0.5 { 0.35 } else { 0.0 },
            ..Default::default()
        }),
        _ => return None,
    };
    Some(fx.on(shape).at(t).reduced(d[3] < 0.5))
}

#[derive(Default)]
pub struct StudioGfx {
    fx: Option<FxRenderer>,
    moving: bool,
}

impl Compositor<State> for StudioGfx {
    fn animating(&self) -> bool {
        self.moving
    }

    fn render(&mut self, f: &mut Frame<'_, State>) {
        let (device, queue, size) = (f.device, f.queue, f.size);
        let t = f.rt.t as f32;
        let marks = &f.rt.marks;
        let mut at: Vec<u32> = Vec::new();
        let mut layers: Vec<FxLayer> = Vec::new();
        let mut placements: Vec<FxSlot> = Vec::new();

        for (n, i) in f.rt.instances.iter().enumerate() {
            if i.params[1] != KIND_EFFECT {
                continue;
            }
            let kind = marks.get(n).copied().unwrap_or(0) & 0xff;
            let k = i.params[3];
            let [x, y, w, h] = i.rect.map(|v| v / k);
            let r = i.radii.iter().fold(0.0f32, |a, v| a.max(v.abs())) / k;
            let Some(l) = layer(kind, i.uv, Shape::rounded(x, y, w, h, r), t) else {
                continue;
            };
            at.push(n as u32);
            layers.push(l);
            placements.push(FxSlot::default().scale(k).clip(i.clip, i.fade));
        }
        self.moving = layers.iter().any(|l| l.is_animating());

        let fx = self
            .fx
            .get_or_insert_with(|| FxRenderer::new(device, f.format, FxOptions { dither: true }));
        fx.set_srgb_blend(f.srgb_blend());
        fx.add_format(device, wgpu::TextureFormat::Rgba16Float);
        fx.prepare_with_slots(
            device,
            queue,
            &FxFrame {
                viewport: [size.0, size.1],
                scale: 1.0,
                layers: &layers,
                backdrop: None,
                sources: &[],
            },
            &placements,
        );

        let clear = f.rt.clear_color();
        f.ui.prepare(device, queue, &mut f.rt.text.atlas, &f.rt.instances, size);
        f.ui.draw_with_backdrop(
            f.encoder,
            f.target,
            clear,
            &mut |phase, s: &EffectSite<'_>| {
                let Some(k) = at.iter().position(|n| *n == s.index) else {
                    return;
                };
                match phase {
                    EffectPhase::Backdrop(enc) => {
                        if let Some(bd) = s.backdrop {
                            fx.blur_backdrop(device, queue, enc, bd, k);
                        }
                    }
                    EffectPhase::Draw(pass) => fx.draw_layer_in(pass, s.format, s.origin, k),
                }
            },
        );
    }
}
