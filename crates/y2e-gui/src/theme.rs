use rune::color::hex;
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
    pub ember: Color,
    pub accent_deep: Color,
    pub on_accent: Color,
    pub violet: Color,
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

pub fn palette() -> Palette {
    Palette {
        bg: c("#14100E"),
        ink: c("#1A0F0C"),
        text: c("#F7F1EA"),
        text_dim: c("#DCD1C6"),
        text_faint: c("#B9ACA0"),
        accent: c("#FF6A4D"),
        accent_hi: c("#FF9A84"),
        ember: c("#FFB36B"),
        accent_deep: c("#5A241B"),
        on_accent: c("#1A0F0C"),
        violet: c("#5B3A8C"),
        glass: c("#181110B8"),
        glass_line: c("#FFFFFF24"),
        chip: c("#FFFFFF17"),
        bar: c("#7A6A62"),
        success: c("#9BCD61"),
        danger: c("#FF8A7D"),
    }
}
