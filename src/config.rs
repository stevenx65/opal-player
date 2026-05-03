use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::theme::OpalineTheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub volume: f32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub repeat_mode: RepeatModeConfig,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default)]
    pub music_dirs: Vec<String>,
    #[serde(default)]
    pub recent_files: Vec<String>,
    #[serde(default)]
    pub last_playlist: Option<String>,
    #[serde(default = "default_theme_overrides")]
    pub theme_overrides: HashMap<String, String>,
    #[serde(default)]
    pub keybindings: HashMap<String, String>,
    #[serde(default)]
    pub scan_on_startup: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepeatModeConfig {
    #[default]
    Off,
    Playlist,
    Track,
}

fn default_theme_overrides() -> HashMap<String, String> {
    HashMap::new()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 0.8,
            muted: false,
            repeat_mode: RepeatModeConfig::default(),
            shuffle: false,
            music_dirs: vec![dirs::audio_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy()
                .to_string()],
            recent_files: Vec::new(),
            last_playlist: None,
            theme_overrides: default_theme_overrides(),
            keybindings: HashMap::new(),
            scan_on_startup: true,
        }
    }
}

impl Config {
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("no config dir")?
            .join("opal-player");
        std::fs::create_dir_all(&dir).ok();
        Ok(dir.join("config.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn theme(&self) -> OpalineTheme {
        OpalineTheme::from_config(&self.theme_overrides)
    }
}
