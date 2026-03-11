//! Color themes for SVG output.
//!
//! Each theme defines colors for text elements and ASCII art.
//! ASCII characters are grouped by density (high/medium/low) for coloring.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ThemePalette {
    pub bg: &'static str,
    pub text: &'static str,
    pub key: &'static str,
    pub value: &'static str,
    pub cc: &'static str,
    pub ascii_high: &'static str,
    pub ascii_medium: &'static str,
    pub ascii_low: &'static str,
}

impl ThemePalette {
    /// Maps ASCII art characters to theme colors.
    ///
    /// Character groups (for jp2a-generated art):
    /// - High density: K, X, N, W, 0, O (dark areas)
    /// - Medium: c, o, x, d, k
    /// - Low density: . , ' : ; (light areas)
    pub fn ascii_color_map(&self) -> HashMap<char, String> {
        let mut map = HashMap::new();
        let high = self.ascii_high.to_string();
        let medium = self.ascii_medium.to_string();
        let low = self.ascii_low.to_string();

        for ch in ['K', 'X', 'N', 'W', '0', 'O'] {
            map.insert(ch, high.clone());
        }
        for ch in ['c', 'o', 'x', 'd', 'k'] {
            map.insert(ch, medium.clone());
        }
        for ch in ['.', ',', '\'', ':', ';'] {
            map.insert(ch, low.clone());
        }
        map
    }
}

pub fn get_theme(name: &str) -> &'static ThemePalette {
    match name {
        "matrix" => &MATRIX,
        "gruvbox" => &GRUVBOX,
        "nord" => &NORD,
        "dracula" => &DRACULA,
        "monokai" => &MONOKAI,
        "catppuccin" => &CATPPUCCIN,
        "synthwave" => &SYNTHWAVE,
        "ayu" => &AYU,
        "github" => &GITHUB,
        _ => &GITHUB,
    }
}

const GITHUB: ThemePalette = ThemePalette {
    bg: "#0d1117",
    text: "#c9d1d9",
    key: "#ffa657",
    value: "#a5d6ff",
    cc: "#6e7681",
    ascii_high: "#58a6ff",
    ascii_medium: "#8b949e",
    ascii_low: "#30363d",
};

const MATRIX: ThemePalette = ThemePalette {
    bg: "#0d0d0d",
    text: "#00ff41",
    key: "#39ff14",
    value: "#00cc33",
    cc: "#008f11",
    ascii_high: "#00ff41",
    ascii_medium: "#00cc33",
    ascii_low: "#008f11",
};

const GRUVBOX: ThemePalette = ThemePalette {
    bg: "#282828",
    text: "#ebdbb2",
    key: "#fb4934",
    value: "#8ec07c",
    cc: "#928374",
    ascii_high: "#fb4934",
    ascii_medium: "#8ec07c",
    ascii_low: "#fabd2f",
};

const NORD: ThemePalette = ThemePalette {
    bg: "#2e3440",
    text: "#eceff4",
    key: "#88c0d0",
    value: "#81a1c1",
    cc: "#4c566a",
    ascii_high: "#88c0d0",
    ascii_medium: "#81a1c1",
    ascii_low: "#5e81ac",
};

const DRACULA: ThemePalette = ThemePalette {
    bg: "#282a36",
    text: "#f8f8f2",
    key: "#bd93f9",
    value: "#ff79c6",
    cc: "#6272a4",
    ascii_high: "#bd93f9",
    ascii_medium: "#ff79c6",
    ascii_low: "#f1fa8c",
};

const MONOKAI: ThemePalette = ThemePalette {
    bg: "#272822",
    text: "#f8f8f2",
    key: "#f92672",
    value: "#66d9ef",
    cc: "#75715e",
    ascii_high: "#f92672",
    ascii_medium: "#66d9ef",
    ascii_low: "#e6db74",
};

const CATPPUCCIN: ThemePalette = ThemePalette {
    bg: "#1e1e2e",
    text: "#cdd6f4",
    key: "#f38ba8",
    value: "#a6e3a1",
    cc: "#6c7086",
    ascii_high: "#f38ba8",
    ascii_medium: "#a6e3a1",
    ascii_low: "#f9e2af",
};

const SYNTHWAVE: ThemePalette = ThemePalette {
    bg: "#1a1a2e",
    text: "#eaeaea",
    key: "#fe4a49",
    value: "#2ab7ca",
    cc: "#4a4a6a",
    ascii_high: "#fe4a49",
    ascii_medium: "#2ab7ca",
    ascii_low: "#fed766",
};

const AYU: ThemePalette = ThemePalette {
    bg: "#0f1419",
    text: "#e6e1cf",
    key: "#ff8f40",
    value: "#e6b450",
    cc: "#5c6773",
    ascii_high: "#ff8f40",
    ascii_medium: "#e6b450",
    ascii_low: "#b8cc52",
};
