//! Configuration loading and parsing for neovide-tabs.
//!
//! Loads configuration from `~/.config/neovide-tabs/config.json`.
//! Falls back to defaults if the file is missing or invalid.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Default background color (Tokyo Night dark theme)
pub const DEFAULT_BACKGROUND_COLOR: u32 = 0x1a1b26;

/// Raw configuration as read from JSON file
#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    /// Background color as hex string (with or without # prefix)
    background_color: Option<String>,
}

/// Parsed application configuration with validated values
#[derive(Debug, Clone)]
pub struct Config {
    /// Background color as RGB value (0x00RRGGBB format)
    pub background_color: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            background_color: DEFAULT_BACKGROUND_COLOR,
        }
    }
}

impl Config {
    /// Load configuration from the default config file path.
    /// Returns default config if file is missing or invalid.
    pub fn load() -> Self {
        let path = match config_file_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let config_file: ConfigFile = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        Self::from_config_file(config_file)
    }

    /// Convert raw config file to validated Config
    fn from_config_file(file: ConfigFile) -> Self {
        let background_color = file
            .background_color
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or(DEFAULT_BACKGROUND_COLOR);

        Self { background_color }
    }
}

/// Get the path to the config file: `~/.config/neovide-tabs/config.json`
fn config_file_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".config")
            .join("neovide-tabs")
            .join("config.json"),
    )
}

/// Parse a hex color string (with or without # prefix) to RGB u32.
/// Returns None if the format is invalid.
///
/// Accepts formats:
/// - "1a1b26" (6 chars, no prefix)
/// - "#1a1b26" (7 chars with # prefix)
fn parse_hex_color(s: &str) -> Option<u32> {
    let hex = s.strip_prefix('#').unwrap_or(s);

    if hex.len() != 6 {
        return None;
    }

    u32::from_str_radix(hex, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_without_prefix() {
        assert_eq!(parse_hex_color("1a1b26"), Some(0x1a1b26));
        assert_eq!(parse_hex_color("ffffff"), Some(0xffffff));
        assert_eq!(parse_hex_color("000000"), Some(0x000000));
        assert_eq!(parse_hex_color("ABCDEF"), Some(0xABCDEF));
    }

    #[test]
    fn test_parse_hex_color_with_prefix() {
        assert_eq!(parse_hex_color("#1a1b26"), Some(0x1a1b26));
        assert_eq!(parse_hex_color("#ffffff"), Some(0xffffff));
        assert_eq!(parse_hex_color("#000000"), Some(0x000000));
        assert_eq!(parse_hex_color("#ABCDEF"), Some(0xABCDEF));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#"), None);
        assert_eq!(parse_hex_color("1a1b2"), None); // Too short
        assert_eq!(parse_hex_color("1a1b267"), None); // Too long
        assert_eq!(parse_hex_color("#1a1b2"), None); // Too short with prefix
        assert_eq!(parse_hex_color("gggggg"), None); // Invalid hex chars
        assert_eq!(parse_hex_color("##1a1b26"), None); // Double prefix
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.background_color, DEFAULT_BACKGROUND_COLOR);
    }

    #[test]
    fn test_config_from_file_with_valid_color() {
        let file = ConfigFile {
            background_color: Some("#ff0000".to_string()),
        };
        let config = Config::from_config_file(file);
        assert_eq!(config.background_color, 0xff0000);
    }

    #[test]
    fn test_config_from_file_with_invalid_color() {
        let file = ConfigFile {
            background_color: Some("invalid".to_string()),
        };
        let config = Config::from_config_file(file);
        assert_eq!(config.background_color, DEFAULT_BACKGROUND_COLOR);
    }

    #[test]
    fn test_config_from_file_with_no_color() {
        let file = ConfigFile {
            background_color: None,
        };
        let config = Config::from_config_file(file);
        assert_eq!(config.background_color, DEFAULT_BACKGROUND_COLOR);
    }

    #[test]
    fn test_config_file_path() {
        let path = config_file_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("config.json"));
        assert!(path.to_string_lossy().contains("neovide-tabs"));
    }
}
