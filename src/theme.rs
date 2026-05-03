use ratatui::style::Color;

/// Opaline theme — a soft, iridescent pastel-on-dark palette inspired by opal gemstones.
#[derive(Debug, Clone)]
pub struct OpalineTheme {
    pub bg: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub progress: Color,
    pub progress_bg: Color,
    pub border: Color,
    pub highlight: Color,
    pub highlight_bg: Color,
}

impl Default for OpalineTheme {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(0x1a, 0x1b, 0x26),
            surface: Color::Rgb(0x24, 0x28, 0x3b),
            surface_alt: Color::Rgb(0x2f, 0x33, 0x4d),
            primary: Color::Rgb(0xf0, 0xc0, 0xc0),       // soft coral/pink
            secondary: Color::Rgb(0xb4, 0xf9, 0xf8),      // pale teal
            accent: Color::Rgb(0xcb, 0xc0, 0xff),          // soft lavender
            text: Color::Rgb(0xc0, 0xca, 0xf5),
            text_dim: Color::Rgb(0x56, 0x5f, 0x89),
            text_bright: Color::Rgb(0xff, 0xff, 0xff),
            success: Color::Rgb(0x9e, 0xce, 0x6a),
            warning: Color::Rgb(0xe0, 0xaf, 0x68),
            error: Color::Rgb(0xf7, 0x76, 0x8e),
            progress: Color::Rgb(0xf0, 0xc0, 0xc0),
            progress_bg: Color::Rgb(0x41, 0x45, 0x68),
            border: Color::Rgb(0x56, 0x5f, 0x89),
            highlight: Color::Rgb(0xf0, 0xc0, 0xc0),
            highlight_bg: Color::Rgb(0x36, 0x3b, 0x54),
        }
    }
}

impl OpalineTheme {
    pub fn from_config(map: &std::collections::HashMap<String, String>) -> Self {
        let mut theme = Self::default();
        for (key, val) in map {
            if let Some(c) = parse_color(val) {
                match key.as_str() {
                    "bg" => theme.bg = c,
                    "surface" => theme.surface = c,
                    "surface_alt" => theme.surface_alt = c,
                    "primary" => theme.primary = c,
                    "secondary" => theme.secondary = c,
                    "accent" => theme.accent = c,
                    "text" => theme.text = c,
                    "text_dim" => theme.text_dim = c,
                    "text_bright" => theme.text_bright = c,
                    "success" => theme.success = c,
                    "warning" => theme.warning = c,
                    "error" => theme.error = c,
                    "progress" => theme.progress = c,
                    "progress_bg" => theme.progress_bg = c,
                    "border" => theme.border = c,
                    "highlight" => theme.highlight = c,
                    "highlight_bg" => theme.highlight_bg = c,
                    _ => {}
                }
            }
        }
        theme
    }
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}
