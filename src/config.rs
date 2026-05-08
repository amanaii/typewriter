use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub test: TestConfig,
    pub theme: ThemeConfig,
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TestConfig {
    pub default_mode: String,
    pub word_count: usize,
    pub duration: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            default_mode: "words".to_string(),
            word_count: 25,
            duration: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
    pub accent: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "dark".to_string(),
            accent: "#e2b714".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub show_live_wpm: bool,
    pub smooth_caret: bool,
    pub stop_on_error: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            show_live_wpm: true,
            smooth_caret: true,
            stop_on_error: false,
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> io::Result<Self> {
        let path = path.map(Path::to_path_buf).unwrap_or_else(config_path);
        if !path.exists() {
            let config = Self::default();
            config.save_to(&path)?;
            return Ok(config);
        }
        let raw = fs::read_to_string(path)?;
        toml::from_str(&raw).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to(&config_path())
    }

    fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, raw)
    }
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("typewriter")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}
