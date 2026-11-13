//! Slow, ambient motion shared by every frame: the aurora drifts and the
//! source artwork breathes. Values are published from one low-rate thread and
//! bound to elements as live values, so the view never rebuilds for them.

use rune::Live;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub struct Ambient {
    pub warm: Live<[f32; 2]>,
    pub cool: Live<[f32; 2]>,
    pub glow: Live<[f32; 2]>,
    pub zoom: Live<f32>,
}

pub fn ambient() -> &'static Ambient {
    static AMBIENT: OnceLock<Ambient> = OnceLock::new();
    AMBIENT.get_or_init(|| Ambient {
        warm: Live::new([0.0, 0.0]),
        cool: Live::new([0.0, 0.0]),
        glow: Live::new([0.0, 0.0]),
        zoom: Live::new(1.05),
    })
}

/// Starts the publisher. Thirty updates a second are enough: each binding is
/// smoothed by a spring on the render side.
pub fn start() {
    let started = Instant::now();
    let _ = std::thread::Builder::new()
        .name("y2e-ambient".to_string())
        .spawn(move || {
            let a = ambient();
            loop {
                let t = started.elapsed().as_secs_f32();
                a.warm
                    .set([(t * 0.21).sin() * 90.0, (t * 0.17).cos() * 50.0]);
                a.cool
                    .set([(t * 0.15).cos() * 100.0, (t * 0.23).sin() * 60.0]);
                a.glow
                    .set([(t * 0.11).sin() * 120.0, (t * 0.13).cos() * 30.0]);
                a.zoom.set(1.06 + 0.03 * (t * 0.12).sin());
                std::thread::sleep(Duration::from_millis(33));
            }
        });
}
