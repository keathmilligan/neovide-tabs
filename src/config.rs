//! Configuration loading and parsing for neovide-tabs.
//!
//! Loads configuration from `~/.config/neovide-tabs/config.json`.
//! Falls back to defaults if the file is missing or invalid.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default background color (Tokyo Night dark theme)
pub const DEFAULT_BACKGROUND_COLOR: u32 = 0x1a1b26;

/// Default tab icon filename (Neovide icon for profiles)
pub const DEFAULT_ICON: &str = "neovide.png";

/// Application window icon filename
pub const APP_ICON: &str = "neovide-tabs.png";

/// Default profile name
pub const DEFAULT_PROFILE_NAME: &str = "Default";

/// Default hotkey for the generated Default profile
pub const DEFAULT_PROFILE_HOTKEY: &str = "Ctrl+Shift+F1";

/// Raw profile as read from JSON file
#[derive(Debug, Deserialize, Clone)]
struct ProfileFile {
    /// Profile name (required)
    name: String,
    /// Icon filename (optional, defaults to neovide.png)
    icon: Option<String>,
    /// Working directory (optional, defaults to home directory)
    working_directory: Option<String>,
    /// Global hotkey for this profile (optional, e.g., "Ctrl+Shift+F1")
    hotkey: Option<String>,
}

/// Raw hotkey configuration as read from JSON file
#[derive(Debug, Deserialize, Default, Clone)]
struct HotkeyConfigFile {
    /// Tab hotkey mappings: hotkey string -> tab number (1-based)
    tab: Option<HashMap<String, u32>>,
}

/// Raw configuration as read from JSON file
#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    /// Background color as hex string (with or without # prefix)
    background_color: Option<String>,
    /// List of profiles
    profiles: Option<Vec<ProfileFile>>,
    /// Hotkey configuration
    hotkeys: Option<HotkeyConfigFile>,
}

/// A tab profile with resolved paths
#[derive(Debug, Clone)]
pub struct Profile {
    /// Profile name
    pub name: String,
    /// Icon filename (just the filename, not full path)
    pub icon: String,
    /// Working directory (resolved full path)
    pub working_directory: PathBuf,
    /// Global hotkey for this profile (e.g., "Ctrl+Shift+F1")
    pub hotkey: Option<String>,
}

/// Parsed hotkey configuration
#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    /// Tab hotkey mappings: hotkey string -> tab number (1-based)
    pub tab: HashMap<String, u32>,
}

impl Profile {
    /// Create the default profile with default hotkey
    pub fn default_profile() -> Self {
        Self {
            name: DEFAULT_PROFILE_NAME.to_string(),
            icon: DEFAULT_ICON.to_string(),
            working_directory: dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
            hotkey: Some(DEFAULT_PROFILE_HOTKEY.to_string()),
        }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            tab: default_tab_hotkeys(),
        }
    }
}

/// Generate default tab hotkeys: Ctrl+Shift+1-9,0 for tabs 1-10
fn default_tab_hotkeys() -> HashMap<String, u32> {
    let mut map = HashMap::new();
    // 1-9 map to tabs 1-9
    for i in 1..=9 {
        map.insert(format!("Ctrl+Shift+{}", i), i);
    }
    // 0 maps to tab 10
    map.insert("Ctrl+Shift+0".to_string(), 10);
    map
}

/// Parsed application configuration with validated values
#[derive(Debug, Clone)]
pub struct Config {
    /// Background color as RGB value (0x00RRGGBB format)
    pub background_color: u32,
    /// List of profiles (always has at least one - the Default profile)
    pub profiles: Vec<Profile>,
    /// Hotkey configuration
    pub hotkeys: HotkeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            background_color: DEFAULT_BACKGROUND_COLOR,
            profiles: vec![Profile::default_profile()],
            hotkeys: HotkeyConfig::default(),
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

        let profiles = parse_profiles(file.profiles);
        let hotkeys = parse_hotkey_config(file.hotkeys);

        Self {
            background_color,
            profiles,
            hotkeys,
        }
    }

    /// Get the default profile (first profile in the list, which is always "Default")
    pub fn default_profile(&self) -> &Profile {
        // profiles is guaranteed to have at least one element
        &self.profiles[0]
    }

    /// Get a profile by index
    pub fn get_profile(&self, index: usize) -> Option<&Profile> {
        self.profiles.get(index)
    }
}

/// Parse profiles from config file, ensuring Default profile exists
fn parse_profiles(profiles_opt: Option<Vec<ProfileFile>>) -> Vec<Profile> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    let mut profiles: Vec<Profile> = match profiles_opt {
        Some(profile_files) if !profile_files.is_empty() => profile_files
            .into_iter()
            .map(|pf| {
                let working_directory = pf
                    .working_directory
                    .map(|wd| resolve_path(&wd, &home_dir))
                    .unwrap_or_else(|| home_dir.clone());

                Profile {
                    name: pf.name,
                    icon: pf.icon.unwrap_or_else(|| DEFAULT_ICON.to_string()),
                    working_directory,
                    hotkey: pf.hotkey,
                }
            })
            .collect(),
        _ => Vec::new(),
    };

    // Ensure a "Default" profile exists at the beginning
    let has_default = profiles.iter().any(|p| p.name == DEFAULT_PROFILE_NAME);
    if !has_default {
        profiles.insert(0, Profile::default_profile());
    } else if let Some(pos) = profiles.iter().position(|p| p.name == DEFAULT_PROFILE_NAME)
        && pos != 0
    {
        // Move Default to the front if it exists elsewhere
        let default = profiles.remove(pos);
        profiles.insert(0, default);
    }

    profiles
}

/// Parse hotkey configuration from config file
fn parse_hotkey_config(config_opt: Option<HotkeyConfigFile>) -> HotkeyConfig {
    match config_opt {
        Some(config) => {
            // If hotkeys section exists, use it (even if empty, which disables defaults)
            let tab = config.tab.unwrap_or_default();
            HotkeyConfig { tab }
        }
        // No hotkeys section - use defaults
        None => HotkeyConfig::default(),
    }
}

/// Resolve a path string, expanding ~ to home directory
fn resolve_path(path_str: &str, home_dir: &Path) -> PathBuf {
    if path_str.starts_with('~') {
        let rest = path_str.strip_prefix('~').unwrap_or("");
        let rest = rest
            .strip_prefix('/')
            .or_else(|| rest.strip_prefix('\\'))
            .unwrap_or(rest);
        if rest.is_empty() {
            home_dir.to_path_buf()
        } else {
            home_dir.join(rest)
        }
    } else {
        let path = PathBuf::from(path_str);
        // Validate the directory exists, fall back to home if not
        if path.is_dir() {
            path
        } else {
            eprintln!(
                "Warning: Working directory '{}' does not exist, using home directory",
                path_str
            );
            home_dir.to_path_buf()
        }
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

/// Get the path to the data directory: `~/.local/share/neovide-tabs/`
pub fn data_dir_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".local").join("share").join("neovide-tabs"))
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
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, DEFAULT_PROFILE_NAME);
    }

    #[test]
    fn test_config_from_file_with_valid_color() {
        let file = ConfigFile {
            background_color: Some("#ff0000".to_string()),
            profiles: None,
            hotkeys: None,
        };
        let config = Config::from_config_file(file);
        assert_eq!(config.background_color, 0xff0000);
    }

    #[test]
    fn test_config_from_file_with_invalid_color() {
        let file = ConfigFile {
            background_color: Some("invalid".to_string()),
            profiles: None,
            hotkeys: None,
        };
        let config = Config::from_config_file(file);
        assert_eq!(config.background_color, DEFAULT_BACKGROUND_COLOR);
    }

    #[test]
    fn test_config_from_file_with_no_color() {
        let file = ConfigFile {
            background_color: None,
            profiles: None,
            hotkeys: None,
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

    #[test]
    fn test_default_profile() {
        let profile = Profile::default_profile();
        assert_eq!(profile.name, DEFAULT_PROFILE_NAME);
        assert_eq!(profile.icon, "neovide.png");
        assert_eq!(profile.hotkey, Some(DEFAULT_PROFILE_HOTKEY.to_string()));
    }

    #[test]
    fn test_data_dir_path() {
        let path = data_dir_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("neovide-tabs"));
        assert!(path.to_string_lossy().contains(".local"));
    }

    #[test]
    fn test_parse_profiles_empty() {
        let profiles = parse_profiles(None);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
    }

    #[test]
    fn test_parse_profiles_with_default() {
        let profile_files = vec![ProfileFile {
            name: "Default".to_string(),
            icon: Some("custom.png".to_string()),
            working_directory: Some("~".to_string()),
            hotkey: None,
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
        assert_eq!(profiles[0].icon, "custom.png");
        // User-defined Default without hotkey should have no hotkey
        assert_eq!(profiles[0].hotkey, None);
    }

    #[test]
    fn test_parse_profiles_without_default() {
        let profile_files = vec![ProfileFile {
            name: "Work".to_string(),
            icon: None,
            working_directory: None,
            hotkey: None,
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 2);
        // Default should be first with default hotkey
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
        assert_eq!(profiles[0].hotkey, Some(DEFAULT_PROFILE_HOTKEY.to_string()));
        assert_eq!(profiles[1].name, "Work");
        assert_eq!(profiles[1].hotkey, None);
    }

    #[test]
    fn test_parse_profiles_moves_default_to_front() {
        let profile_files = vec![
            ProfileFile {
                name: "Work".to_string(),
                icon: None,
                working_directory: None,
                hotkey: None,
            },
            ProfileFile {
                name: "Default".to_string(),
                icon: None,
                working_directory: None,
                hotkey: Some("Ctrl+Shift+F2".to_string()),
            },
        ];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 2);
        // Default should be moved to first
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
        // User-defined hotkey should be preserved
        assert_eq!(profiles[0].hotkey, Some("Ctrl+Shift+F2".to_string()));
        assert_eq!(profiles[1].name, "Work");
    }

    #[test]
    fn test_resolve_path_home() {
        let home = PathBuf::from("/home/test");
        let resolved = resolve_path("~", &home);
        assert_eq!(resolved, home);
    }

    #[test]
    fn test_resolve_path_home_subdir() {
        let home = PathBuf::from("/home/test");
        let resolved = resolve_path("~/projects", &home);
        assert_eq!(resolved, home.join("projects"));
    }

    #[test]
    fn test_config_default_profile_method() {
        let config = Config::default();
        let profile = config.default_profile();
        assert_eq!(profile.name, DEFAULT_PROFILE_NAME);
    }

    #[test]
    fn test_config_get_profile() {
        let config = Config::default();
        assert!(config.get_profile(0).is_some());
        assert!(config.get_profile(100).is_none());
    }

    #[test]
    fn test_default_tab_hotkeys() {
        let hotkeys = default_tab_hotkeys();
        assert_eq!(hotkeys.len(), 10);
        assert_eq!(hotkeys.get("Ctrl+Shift+1"), Some(&1));
        assert_eq!(hotkeys.get("Ctrl+Shift+9"), Some(&9));
        assert_eq!(hotkeys.get("Ctrl+Shift+0"), Some(&10));
    }

    #[test]
    fn test_hotkey_config_default() {
        let config = HotkeyConfig::default();
        assert_eq!(config.tab.len(), 10);
        assert_eq!(config.tab.get("Ctrl+Shift+1"), Some(&1));
    }

    #[test]
    fn test_parse_hotkey_config_none() {
        let config = parse_hotkey_config(None);
        // Should get defaults
        assert_eq!(config.tab.len(), 10);
    }

    #[test]
    fn test_parse_hotkey_config_empty() {
        let config = parse_hotkey_config(Some(HotkeyConfigFile {
            tab: Some(HashMap::new()),
        }));
        // Empty tab map disables tab hotkeys
        assert_eq!(config.tab.len(), 0);
    }

    #[test]
    fn test_parse_hotkey_config_custom() {
        let mut tab = HashMap::new();
        tab.insert("Alt+1".to_string(), 1);
        tab.insert("Alt+2".to_string(), 2);
        let config = parse_hotkey_config(Some(HotkeyConfigFile { tab: Some(tab) }));
        assert_eq!(config.tab.len(), 2);
        assert_eq!(config.tab.get("Alt+1"), Some(&1));
        assert_eq!(config.tab.get("Alt+2"), Some(&2));
    }

    #[test]
    fn test_profile_with_hotkey() {
        let profile_files = vec![ProfileFile {
            name: "Work".to_string(),
            icon: None,
            working_directory: None,
            hotkey: Some("Ctrl+Shift+F2".to_string()),
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 2);
        // Generated Default at index 0
        assert_eq!(profiles[0].hotkey, Some(DEFAULT_PROFILE_HOTKEY.to_string()));
        // User profile at index 1
        assert_eq!(profiles[1].hotkey, Some("Ctrl+Shift+F2".to_string()));
    }

    #[test]
    fn test_config_default_has_hotkeys() {
        let config = Config::default();
        // Default config should have default tab hotkeys
        assert_eq!(config.hotkeys.tab.len(), 10);
        // Default profile should have default hotkey
        assert_eq!(
            config.profiles[0].hotkey,
            Some(DEFAULT_PROFILE_HOTKEY.to_string())
        );
    }
}
