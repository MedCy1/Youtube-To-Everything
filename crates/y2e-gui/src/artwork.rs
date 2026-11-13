//! Downloads and decodes the source artwork off the interface thread.

use rune::Pixels;

use crate::tint;

pub struct Artwork {
    pub pixels: Pixels,
    pub hue: Option<f32>,
    /// How light the lower part of the picture is, from 0 to 1.
    pub lightness: f32,
}

/// `None` when the picture cannot be fetched or decoded: the interface then
/// goes on without artwork.
pub async fn fetch(url: &str) -> Option<Artwork> {
    let output = tokio::process::Command::new("curl")
        .args(["--fail", "--location", "--silent", "--max-time", "10", url])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = output.stdout;
    tokio::task::spawn_blocking(move || {
        let rgba = image::load_from_memory(&bytes).ok()?.into_rgba8();
        let hue = tint::dominant_hue(&rgba, rgba.width(), rgba.height());
        let lightness = tint::lower_lightness(&rgba, rgba.width(), rgba.height());
        let pixels = Pixels::new();
        pixels.set(rgba.width(), rgba.height(), rgba.into_raw());
        Some(Artwork {
            pixels,
            hue,
            lightness,
        })
    })
    .await
    .ok()
    .flatten()
}
