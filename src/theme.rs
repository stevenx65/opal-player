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

macro_rules! theme {
    (
        $(
            $name:ident => { $($field:ident : ($r:expr, $g:expr, $b:expr)),+ $(,)? }
        )*
    ) => {
        $(
            pub fn $name() -> Self {
                Self {
                    $($field: Color::Rgb($r, $g, $b)),+
                }
            }
        )*

        pub fn from_preset(name: &str) -> Option<Self> {
            match name.to_lowercase().as_str() {
                $(
                    stringify!($name) => Some(Self::$name()),
                )*
                _ => None,
            }
        }

        pub fn preset_names() -> &'static [&'static str] {
            &[$(stringify!($name)),*]
        }
    };
}

impl OpalineTheme {
    theme! {
        opaline => {
            bg:             (0x1a, 0x1b, 0x26),
            surface:        (0x24, 0x28, 0x3b),
            surface_alt:    (0x2f, 0x33, 0x4d),
            primary:        (0xf0, 0xc0, 0xc0),
            secondary:      (0xb4, 0xf9, 0xf8),
            accent:         (0xcb, 0xc0, 0xff),
            text:           (0xc0, 0xca, 0xf5),
            text_dim:       (0x56, 0x5f, 0x89),
            text_bright:    (0xff, 0xff, 0xff),
            success:        (0x9e, 0xce, 0x6a),
            warning:        (0xe0, 0xaf, 0x68),
            error:          (0xf7, 0x76, 0x8e),
            progress:       (0xf0, 0xc0, 0xc0),
            progress_bg:    (0x41, 0x45, 0x68),
            border:         (0x56, 0x5f, 0x89),
            highlight:      (0xf0, 0xc0, 0xc0),
            highlight_bg:   (0x36, 0x3b, 0x54),
        }

        catppuccin => {
            bg:             (0x1e, 0x1e, 0x2e),
            surface:        (0x31, 0x32, 0x44),
            surface_alt:    (0x45, 0x47, 0x5a),
            primary:        (0xf5, 0xc2, 0xe7),
            secondary:      (0x89, 0xdc, 0xeb),
            accent:         (0xb4, 0xbe, 0xfe),
            text:           (0xcd, 0xd6, 0xf4),
            text_dim:       (0x6c, 0x70, 0x86),
            text_bright:    (0xff, 0xff, 0xff),
            success:        (0xa6, 0xe3, 0xa1),
            warning:        (0xf9, 0xe2, 0xaf),
            error:          (0xf3, 0x8b, 0xa8),
            progress:       (0xf5, 0xc2, 0xe7),
            progress_bg:    (0x45, 0x47, 0x5a),
            border:         (0x6c, 0x70, 0x86),
            highlight:      (0xf5, 0xc2, 0xe7),
            highlight_bg:   (0x45, 0x47, 0x5a),
        }

        nord => {
            bg:             (0x2e, 0x34, 0x40),
            surface:        (0x3b, 0x42, 0x52),
            surface_alt:    (0x43, 0x4c, 0x5e),
            primary:        (0x88, 0xc0, 0xd0),
            secondary:      (0x81, 0xa1, 0xc1),
            accent:         (0xb4, 0x8e, 0xad),
            text:           (0xe5, 0xe9, 0xf0),
            text_dim:       (0x4c, 0x56, 0x6a),
            text_bright:    (0xec, 0xef, 0xf4),
            success:        (0xa3, 0xbe, 0x8c),
            warning:        (0xeb, 0xcb, 0x8b),
            error:          (0xbf, 0x61, 0x6a),
            progress:       (0x88, 0xc0, 0xd0),
            progress_bg:    (0x4c, 0x56, 0x6a),
            border:         (0x4c, 0x56, 0x6a),
            highlight:      (0x88, 0xc0, 0xd0),
            highlight_bg:   (0x3b, 0x42, 0x52),
        }

        dracula => {
            bg:             (0x28, 0x2a, 0x36),
            surface:        (0x34, 0x36, 0x46),
            surface_alt:    (0x44, 0x47, 0x5a),
            primary:        (0xbd, 0x93, 0xf9),
            secondary:      (0x8b, 0xe9, 0xfd),
            accent:         (0xff, 0x79, 0xc6),
            text:           (0xf8, 0xf8, 0xf2),
            text_dim:       (0x62, 0x72, 0xa4),
            text_bright:    (0xff, 0xff, 0xff),
            success:        (0x50, 0xfa, 0x7b),
            warning:        (0xf1, 0xfa, 0x8c),
            error:          (0xff, 0x55, 0x55),
            progress:       (0xbd, 0x93, 0xf9),
            progress_bg:    (0x44, 0x47, 0x5a),
            border:         (0x62, 0x72, 0xa4),
            highlight:      (0xbd, 0x93, 0xf9),
            highlight_bg:   (0x34, 0x36, 0x46),
        }

        one_dark => {
            bg:             (0x28, 0x2c, 0x34),
            surface:        (0x33, 0x38, 0x42),
            surface_alt:    (0x3e, 0x44, 0x51),
            primary:        (0xe0, 0x6c, 0x75),
            secondary:      (0x61, 0xaf, 0xef),
            accent:         (0xc6, 0x78, 0xdd),
            text:           (0xab, 0xb2, 0xbf),
            text_dim:       (0x5c, 0x63, 0x6a),
            text_bright:    (0xff, 0xff, 0xff),
            success:        (0x98, 0xc3, 0x79),
            warning:        (0xe5, 0xc0, 0x7b),
            error:          (0xe0, 0x6c, 0x75),
            progress:       (0x61, 0xaf, 0xef),
            progress_bg:    (0x3e, 0x44, 0x51),
            border:         (0x5c, 0x63, 0x6a),
            highlight:      (0x61, 0xaf, 0xef),
            highlight_bg:   (0x2c, 0x32, 0x3c),
        }
    }

    pub fn from_config(
        preset: &str,
        overrides: &std::collections::HashMap<String, String>,
    ) -> Self {
        let mut theme = Self::from_preset(preset).unwrap_or_else(Self::opaline);
        for (key, val) in overrides {
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

impl Default for OpalineTheme {
    fn default() -> Self {
        Self::opaline()
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
