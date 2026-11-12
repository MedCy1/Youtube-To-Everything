use rune::Color;

pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub card: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub success: Color,
    pub danger: Color,
    pub control_bg: Color,
}

pub const PALETTE: Palette = Palette {
    bg: Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 },
    surface: Color { r: 0.12, g: 0.12, b: 0.15, a: 1.0 },
    card: Color { r: 0.16, g: 0.16, b: 0.20, a: 1.0 },
    text_primary: Color { r: 0.95, g: 0.95, b: 0.98, a: 1.0 },
    text_secondary: Color { r: 0.70, g: 0.70, b: 0.78, a: 1.0 },
    text_muted: Color { r: 0.45, g: 0.45, b: 0.52, a: 1.0 },
    accent: Color { r: 0.98, g: 0.22, b: 0.22, a: 1.0 },
    success: Color { r: 0.18, g: 0.80, b: 0.44, a: 1.0 },
    danger: Color { r: 0.92, g: 0.28, b: 0.28, a: 1.0 },
    control_bg: Color { r: 0.20, g: 0.20, b: 0.26, a: 1.0 },
};
