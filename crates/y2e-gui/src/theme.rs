use rune::color::{hex, oklch};
use rune::Color;

/// Warm night palette: the source artwork is the hero, the interface is glass over it.
pub struct Palette {
    pub bg: Color,
    pub ink: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_faint: Color,
    pub accent: Color,
    pub accent_hi: Color,
    pub accent_deep: Color,
    pub on_accent: Color,
    pub glass: Color,
    pub glass_line: Color,
    pub chip: Color,
    pub bar: Color,
    pub success: Color,
    pub danger: Color,
}

fn c(h: &str) -> Color {
    hex(h).unwrap_or(Color::BLACK)
}

/// The neutrals are fixed; every coloured token is derived from one hue so the
/// whole interface follows the artwork while keeping the same lightness and chroma.
pub fn palette(hue: f32) -> Palette {
    let accent = oklch(0.72, 0.19, hue);
    Palette {
        bg: c("#14100E"),
        ink: c("#1A0F0C"),
        text: c("#F7F1EA"),
        text_dim: c("#DCD1C6"),
        text_faint: c("#B9ACA0"),
        accent,
        accent_hi: oklch(0.83, 0.11, hue),
        accent_deep: oklch(0.30, 0.08, hue),
        on_accent: c("#1A0F0C"),
        glass: c("#181110B8"),
        glass_line: c("#FFFFFF24"),
        chip: c("#FFFFFF17"),
        bar: c("#7A6A62"),
        success: c("#9BCD61"),
        danger: c("#FF8A7D"),
    }
}
