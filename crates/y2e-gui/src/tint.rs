//! Picks the hue the interface takes from a piece of artwork.

const BINS: usize = 36;

/// sRGB bytes to Oklab, the space the interface's colours are authored in.
fn oklab(rgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(|c| {
        let c = c as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// Dominant hue of an RGBA image, in degrees, or `None` for a picture with no
/// colour worth following (greyscale, near black or near white).
pub fn dominant_hue(rgba: &[u8], width: u32, height: u32) -> Option<f32> {
    let pixels = (width as usize) * (height as usize);
    if pixels == 0 || rgba.len() < pixels * 4 {
        return None;
    }
    let stride = (pixels / 4096).max(1);
    let mut weights = [0.0f32; BINS];
    let mut total = 0.0f32;

    for pixel in rgba.as_chunks::<4>().0.iter().step_by(stride) {
        let [l, a, b] = oklab([pixel[0], pixel[1], pixel[2]]);
        let chroma = a.hypot(b);
        if l < 0.25 || chroma < 0.06 {
            continue;
        }
        let hue = b.atan2(a).to_degrees().rem_euclid(360.0);
        let weight = chroma * chroma * l;
        weights[((hue / 360.0 * BINS as f32) as usize).min(BINS - 1)] += weight;
        total += weight;
    }

    if total < 0.5 {
        return None;
    }

    // Smooth over neighbouring bins so one noisy bin cannot win.
    let smoothed = |i: usize| {
        weights[(i + BINS - 1) % BINS] * 0.5 + weights[i] + weights[(i + 1) % BINS] * 0.5
    };
    let best = (0..BINS).max_by(|&a, &b| smoothed(a).total_cmp(&smoothed(b)))?;
    Some((best as f32 + 0.5) * 360.0 / BINS as f32)
}

/// Mean lightness (Oklab, 0 to 1) of the lower part of an RGBA image, where the
/// title and the dock sit. Used to decide how much scrim the text needs.
pub fn lower_lightness(rgba: &[u8], width: u32, height: u32) -> f32 {
    let (width, height) = (width as usize, height as usize);
    if width == 0 || height == 0 || rgba.len() < width * height * 4 {
        return 0.3;
    }
    let first_row = height * 55 / 100;
    let (mut sum, mut count) = (0.0f32, 0u32);
    for y in (first_row..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let i = (y * width + x) * 4;
            sum += oklab([rgba[i], rgba[i + 1], rgba[i + 2]])[0];
            count += 1;
        }
    }
    sum / count.max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(rgb: [u8; 3], side: u32) -> Vec<u8> {
        (0..side * side)
            .flat_map(|_| [rgb[0], rgb[1], rgb[2], 255])
            .collect()
    }

    #[test]
    fn follows_the_dominant_colour() {
        let red = dominant_hue(&flat([230, 40, 30], 64), 64, 64).unwrap();
        let blue = dominant_hue(&flat([30, 60, 230], 64), 64, 64).unwrap();
        assert!((red - 29.0).abs() < 15.0, "red hue was {red}");
        assert!((blue - 265.0).abs() < 15.0, "blue hue was {blue}");
    }

    #[test]
    fn measures_how_light_the_lower_part_is() {
        assert!(lower_lightness(&flat([255, 255, 255], 32), 32, 32) > 0.95);
        assert!(lower_lightness(&flat([0, 0, 0], 32), 32, 32) < 0.05);
    }

    #[test]
    fn ignores_greyscale_and_empty_input() {
        assert_eq!(dominant_hue(&flat([128, 128, 128], 64), 64, 64), None);
        assert_eq!(dominant_hue(&[], 0, 0), None);
    }
}
