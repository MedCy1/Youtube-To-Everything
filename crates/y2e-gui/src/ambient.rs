//! Slow, ambient motion shared by every frame: the source artwork breathes.
//! The value is published from one low-rate thread and bound to the artwork as
//! a live value, so the view never rebuilds for it.

use rune::Live;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub struct Ambient {
    pub zoom: Live<f32>,
}

pub fn ambient() -> &'static Ambient {
    static AMBIENT: OnceLock<Ambient> = OnceLock::new();
    AMBIENT.get_or_init(|| Ambient {
        zoom: Live::new(1.05),
    })
}

/// Starts the publisher. Thirty updates a second are enough: the binding is
/// smoothed by a spring on the render side.
pub fn start() {
    let started = Instant::now();
    let _ = std::thread::Builder::new()
        .name("y2e-ambient".to_string())
        .spawn(move || {
            let a = ambient();
            loop {
                let t = started.elapsed().as_secs_f32();
                a.zoom.set(1.06 + 0.03 * (t * 0.12).sin());
                std::thread::sleep(Duration::from_millis(33));
            }
        });
}
