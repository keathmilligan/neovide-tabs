//! Configuration loading and parsing for neovide-tabs.
//!
//! Loads configuration from `~/.config/neovide-tabs/config.jsonc` (preferred)
//! or `~/.config/neovide-tabs/config.json` (fallback).
//! Both files support JSONC format (JSON with // comments).
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

/// Default configuration file template (JSONC format with comments)
/// Includes an uncommented "Neovim" profile for out-of-box functionality.
const DEFAULT_CONFIG_TEMPLATE: &str = r##"// neovide-tabs configuration file
// Uncomment and modify options below to customize behavior.
// See https://github.com/your-repo/neovide-tabs for documentation.

{
    // Background color for the window (hex format, with or without # prefix)
    // This color is used for the title bar and to fill exposed areas during resize
    // "background_color": "#1a1b26",

    // Hotkey configuration
    // "hotkeys": {
    //     // Tab switching hotkeys: maps key combination to tab number (1-based)
    //     // Default: Ctrl+Shift+1-9 for tabs 1-9, Ctrl+Shift+0 for tab 10
    //     // Set to empty object {} to disable all tab hotkeys
    //     "tab": {
    //         "Ctrl+Shift+1": 1,
    //         "Ctrl+Shift+2": 2,
    //         "Ctrl+Shift+3": 3,
    //         "Ctrl+Shift+4": 4,
    //         "Ctrl+Shift+5": 5,
    //         "Ctrl+Shift+6": 6,
    //         "Ctrl+Shift+7": 7,
    //         "Ctrl+Shift+8": 8,
    //         "Ctrl+Shift+9": 9,
    //         "Ctrl+Shift+0": 10
    //     }
    // },

    // Profile definitions for tabs
    // Each profile can specify a name, icon, working directory, and hotkey
    // The first profile is used for the initial tab when the application starts
    "profiles": [
        {
            // Default profile - used for the initial tab
            "name": "Neovim",
            // Uses default icon (neovide.png) and home directory
            "hotkey": "Ctrl+Shift+F1"
        }
        // Example: Additional profile with all options
        // {
        //     // Profile name (required) - displayed in the tab
        //     "name": "Work",
        //     // Icon file path (optional) - full path to a PNG file
        //     // Defaults to neovide.png in the data directory
        //     "icon": "C:/path/to/icon.png",
        //     // Working directory (optional) - where Neovide starts
        //     // Supports ~ for home directory. Defaults to home directory
        //     "working_directory": "~/projects/work",
        //     // Global hotkey (optional) - opens or activates a tab with this profile
        //     // Format: Modifier+Key (e.g., Ctrl+Shift+F2, Alt+Shift+W)
        //     "hotkey": "Ctrl+Shift+F2",
        //     // Tab title format (optional) - dynamic tab title with token expansion
        //     // Supported tokens:
        //     //   %p - Profile name
        //     //   %w - Working directory (uses ~/xxx for paths under home)
        //     //   %t - Neovide window title (current file/buffer)
        //     // Defaults to "%t" (Neovide window title)
        //     // Examples: "%t", "%p: %w", "%p - %t"
        //     "title": "%t"
        // },
        // {
        //     // Minimal profile example - only name is required
        //     "name": "Personal"
        // }
    ]
}
"##;

/// Default title format for profiles (Neovide window title)
pub const DEFAULT_TITLE_FORMAT: &str = "%t";

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
    /// Tab title format string (optional, defaults to "%t")
    /// Supports tokens: %p (profile name), %w (working directory), %t (Neovide window title)
    title: Option<String>,
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
    /// Tab title format string (supports %p, %w, %t tokens)
    pub title: String,
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
            title: DEFAULT_TITLE_FORMAT.to_string(),
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
    /// Load configuration from the config file.
    /// Looks for config.jsonc first, then config.json as fallback.
    /// If no config file exists, generates a default config.jsonc with documented options.
    /// Both .json and .jsonc files support JSONC format (JSON with // comments).
    /// Returns default config if file is missing or invalid.
    pub fn load() -> Self {
        // Ensure config file exists (generates default if missing)
        ensure_config_file();

        let path = match find_config_file() {
            Some(p) => p,
            None => {
                eprintln!("Config: No config file found, using defaults");
                return Self::default();
            }
        };

        eprintln!("Config: Loading from {}", path.display());

        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Config: Failed to read config file: {}", e);
                return Self::default();
            }
        };

        // Strip JSONC comments before parsing
        let json_content = strip_jsonc_comments(&contents);

        let config_file: ConfigFile = match serde_json::from_str(&json_content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Config: Failed to parse JSON: {}", e);
                eprintln!(
                    "Config: JSON content after stripping comments:\n{}",
                    json_content
                );
                return Self::default();
            }
        };

        eprintln!(
            "Config: Parsed successfully - profiles: {:?}",
            config_file.profiles.as_ref().map(|p| p.len())
        );

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

    /// Get the first profile in the list, used for the initial tab.
    /// When profiles are defined in config, returns the first user-defined profile.
    /// When no profiles are defined, returns the internal "Default" profile.
    pub fn default_profile(&self) -> &Profile {
        // profiles is guaranteed to have at least one element
        &self.profiles[0]
    }

    /// Get a profile by index
    pub fn get_profile(&self, index: usize) -> Option<&Profile> {
        self.profiles.get(index)
    }
}

/// Parse profiles from config file.
/// If no profiles are defined (None or empty), falls back to the internal Default profile.
/// If profiles are defined, uses them as-is without inserting a Default profile.
fn parse_profiles(profiles_opt: Option<Vec<ProfileFile>>) -> Vec<Profile> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    eprintln!(
        "Config: parse_profiles called with {:?} profiles",
        profiles_opt.as_ref().map(|p| p.len())
    );

    let profiles: Vec<Profile> = match profiles_opt {
        Some(profile_files) if !profile_files.is_empty() => {
            eprintln!(
                "Config: Processing {} user-defined profiles",
                profile_files.len()
            );
            profile_files
                .into_iter()
                .map(|pf| {
                    eprintln!("Config: Processing profile '{}'", pf.name);
                    let working_directory = pf
                        .working_directory
                        .map(|wd| resolve_working_directory(&wd, &home_dir))
                        .unwrap_or_else(|| home_dir.clone());
                    let icon = resolve_icon_path(pf.icon, &home_dir);
                    let title = pf.title.unwrap_or_else(|| DEFAULT_TITLE_FORMAT.to_string());

                    Profile {
                        name: pf.name,
                        icon,
                        working_directory,
                        hotkey: pf.hotkey,
                        title,
                    }
                })
                .collect()
        }
        // No profiles defined - use internal Default profile as fallback
        _ => {
            eprintln!("Config: No profiles defined, using internal Default profile");
            vec![Profile::default_profile()]
        }
    };

    eprintln!("Config: Final profile count: {}", profiles.len());
    for (i, p) in profiles.iter().enumerate() {
        eprintln!("Config:   [{}] name='{}', hotkey={:?}", i, p.name, p.hotkey);
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

/// Expand ~ to home directory in a path string
fn expand_tilde(path_str: &str, home_dir: &Path) -> PathBuf {
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
        PathBuf::from(path_str)
    }
}

/// Resolve a working directory path string, expanding ~ to home directory.
/// Falls back to home directory if the path doesn't exist.
fn resolve_working_directory(path_str: &str, home_dir: &Path) -> PathBuf {
    let path = expand_tilde(path_str, home_dir);
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

/// Resolve an icon path string, expanding ~ to home directory.
/// Returns the expanded path as a string, or the default icon if not specified.
fn resolve_icon_path(icon_opt: Option<String>, home_dir: &Path) -> String {
    match icon_opt {
        Some(icon_str) if icon_str.starts_with('~') => {
            let expanded = expand_tilde(&icon_str, home_dir);
            expanded.to_string_lossy().to_string()
        }
        Some(icon_str) => icon_str,
        None => DEFAULT_ICON.to_string(),
    }
}

/// Get the path to the config directory: `~/.config/neovide-tabs/`
fn config_dir_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".config").join("neovide-tabs"))
}

/// Get the path to the preferred config file: `~/.config/neovide-tabs/config.jsonc`
fn config_file_path_jsonc() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".config")
            .join("neovide-tabs")
            .join("config.jsonc"),
    )
}

/// Get the path to the fallback config file: `~/.config/neovide-tabs/config.json`
fn config_file_path_json() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(
        home.join(".config")
            .join("neovide-tabs")
            .join("config.json"),
    )
}

/// Find the config file to load. Prefers .jsonc, falls back to .json.
/// Returns None if neither exists.
fn find_config_file() -> Option<PathBuf> {
    // Check for .jsonc first (preferred)
    if let Some(jsonc_path) = config_file_path_jsonc()
        && jsonc_path.exists()
    {
        return Some(jsonc_path);
    }

    // Fall back to .json
    if let Some(json_path) = config_file_path_json()
        && json_path.exists()
    {
        return Some(json_path);
    }

    None
}

/// Ensure the config directory exists, creating it if necessary.
/// Returns true if the directory exists (or was created), false on error.
fn ensure_config_dir() -> bool {
    match config_dir_path() {
        Some(dir) => {
            if dir.exists() {
                true
            } else {
                fs::create_dir_all(&dir).is_ok()
            }
        }
        None => false,
    }
}

/// Ensure the config file exists, generating a default one if it doesn't.
/// This is called before loading config to provide users with a documented template.
/// Generates config.jsonc (preferred format).
fn ensure_config_file() {
    // Check if either .jsonc or .json exists
    if find_config_file().is_some() {
        return;
    }

    // Generate new config as .jsonc
    let path = match config_file_path_jsonc() {
        Some(p) => p,
        None => return,
    };

    // Ensure the config directory exists
    if !ensure_config_dir() {
        return;
    }

    // Write the default config template
    if let Err(e) = fs::write(&path, DEFAULT_CONFIG_TEMPLATE) {
        eprintln!(
            "Warning: Failed to create default config file at {}: {}",
            path.display(),
            e
        );
    }
}

/// Strip JSONC comments from content, returning valid JSON.
/// Supports // line comments. Comments inside strings are preserved.
fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if !in_string && c == '/' && chars.peek() == Some(&'/') {
            // Skip the rest of the line (// comment)
            chars.next(); // consume second /
            for ch in chars.by_ref() {
                if ch == '\n' {
                    result.push('\n');
                    break;
                }
            }
            continue;
        }

        result.push(c);
    }

    result
}

/// Generate the default config file content (JSONC format).
/// This is exposed for testing purposes.
#[cfg(test)]
pub fn generate_default_config() -> &'static str {
    DEFAULT_CONFIG_TEMPLATE
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

/// Context for title expansion
pub struct TitleContext<'a> {
    /// Profile name
    pub profile_name: &'a str,
    /// Working directory path
    pub working_directory: &'a Path,
    /// Neovide window title (empty if not available)
    pub window_title: &'a str,
}

/// Expand a title format string using the provided context.
/// Supports the following tokens:
/// - `%p` - Profile name
/// - `%w` - Working directory (with ~ substitution for home directory)
/// - `%t` - Neovide window title
///
/// After expansion, strips leading/trailing whitespace, tabs, and dashes.
pub fn expand_title(format: &str, context: &TitleContext) -> String {
    let home_dir = dirs::home_dir();

    let mut result = String::with_capacity(format.len() * 2);
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    'p' => {
                        chars.next();
                        result.push_str(context.profile_name);
                    }
                    'w' => {
                        chars.next();
                        let wd_display = format_working_directory(
                            context.working_directory,
                            home_dir.as_deref(),
                        );
                        result.push_str(&wd_display);
                    }
                    't' => {
                        chars.next();
                        result.push_str(context.window_title);
                    }
                    '%' => {
                        // Escape sequence: %% becomes %
                        chars.next();
                        result.push('%');
                    }
                    _ => {
                        // Unknown token, keep as-is
                        result.push('%');
                    }
                }
            } else {
                // % at end of string
                result.push('%');
            }
        } else {
            result.push(c);
        }
    }

    // Strip leading/trailing whitespace, tabs, and dashes
    sanitize_title(&result)
}

/// Format a working directory path for display.
/// Replaces home directory prefix with ~ for brevity.
fn format_working_directory(path: &Path, home_dir: Option<&Path>) -> String {
    if let Some(home) = home_dir
        && let Ok(relative) = path.strip_prefix(home)
    {
        if relative.as_os_str().is_empty() {
            return "~".to_string();
        }
        // Use forward slashes for consistency in display
        let relative_str = relative.to_string_lossy();
        return format!("~/{}", relative_str.replace('\\', "/"));
    }
    // Not under home, return full path with forward slashes
    path.to_string_lossy().replace('\\', "/")
}

/// Strip leading and trailing whitespace, tabs, and dash characters from a title.
fn sanitize_title(title: &str) -> String {
    let chars_to_strip: &[char] = &[' ', '\t', '-'];
    title.trim_matches(chars_to_strip).to_string()
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
    fn test_config_file_path_jsonc() {
        let path = config_file_path_jsonc();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("config.jsonc"));
        assert!(path.to_string_lossy().contains("neovide-tabs"));
    }

    #[test]
    fn test_config_file_path_json() {
        let path = config_file_path_json();
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
            title: None,
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, DEFAULT_PROFILE_NAME);
        assert_eq!(profiles[0].icon, "custom.png");
        // User-defined Default without hotkey should have no hotkey
        assert_eq!(profiles[0].hotkey, None);
        // Title should default to %t
        assert_eq!(profiles[0].title, DEFAULT_TITLE_FORMAT);
    }

    #[test]
    fn test_parse_profiles_user_defined() {
        // User-defined profiles are used as-is without inserting Default
        let profile_files = vec![ProfileFile {
            name: "Work".to_string(),
            icon: None,
            working_directory: None,
            hotkey: None,
            title: None,
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 1);
        // First profile is the user-defined one
        assert_eq!(profiles[0].name, "Work");
        assert_eq!(profiles[0].hotkey, None);
        // Title should default to %t
        assert_eq!(profiles[0].title, DEFAULT_TITLE_FORMAT);
    }

    #[test]
    fn test_parse_profiles_preserves_order() {
        // User-defined profiles maintain their order
        let profile_files = vec![
            ProfileFile {
                name: "Work".to_string(),
                icon: None,
                working_directory: None,
                hotkey: None,
                title: None,
            },
            ProfileFile {
                name: "Personal".to_string(),
                icon: None,
                working_directory: None,
                hotkey: Some("Ctrl+Shift+F2".to_string()),
                title: Some("%p: %w".to_string()),
            },
        ];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 2);
        // Order is preserved - first defined profile is first
        assert_eq!(profiles[0].name, "Work");
        assert_eq!(profiles[0].hotkey, None);
        assert_eq!(profiles[0].title, DEFAULT_TITLE_FORMAT);
        assert_eq!(profiles[1].name, "Personal");
        assert_eq!(profiles[1].hotkey, Some("Ctrl+Shift+F2".to_string()));
        assert_eq!(profiles[1].title, "%p: %w");
    }

    #[test]
    fn test_expand_tilde_home() {
        let home = PathBuf::from("/home/test");
        let resolved = expand_tilde("~", &home);
        assert_eq!(resolved, home);
    }

    #[test]
    fn test_expand_tilde_home_subdir() {
        let home = PathBuf::from("/home/test");
        let resolved = expand_tilde("~/projects", &home);
        assert_eq!(resolved, home.join("projects"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let home = PathBuf::from("/home/test");
        let resolved = expand_tilde("/absolute/path", &home);
        assert_eq!(resolved, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_resolve_icon_path_with_tilde() {
        let home = PathBuf::from("/home/test");
        let icon = resolve_icon_path(Some("~/icons/my-icon.png".to_string()), &home);
        // Use PathBuf for comparison to handle platform-specific separators
        let expected = home.join("icons/my-icon.png");
        assert_eq!(icon, expected.to_string_lossy());
    }

    #[test]
    fn test_resolve_icon_path_absolute() {
        let home = PathBuf::from("/home/test");
        let icon = resolve_icon_path(Some("/absolute/path/icon.png".to_string()), &home);
        assert_eq!(icon, "/absolute/path/icon.png");
    }

    #[test]
    fn test_resolve_icon_path_default() {
        let home = PathBuf::from("/home/test");
        let icon = resolve_icon_path(None, &home);
        assert_eq!(icon, DEFAULT_ICON);
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
            title: None,
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles.len(), 1);
        // User profile with hotkey
        assert_eq!(profiles[0].name, "Work");
        assert_eq!(profiles[0].hotkey, Some("Ctrl+Shift+F2".to_string()));
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

    #[test]
    fn test_generate_default_config_not_empty() {
        let content = generate_default_config();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_generate_default_config_contains_background_color() {
        let content = generate_default_config();
        assert!(content.contains("background_color"));
        assert!(content.contains("#1a1b26"));
    }

    #[test]
    fn test_generate_default_config_contains_hotkeys() {
        let content = generate_default_config();
        assert!(content.contains("hotkeys"));
        assert!(content.contains("tab"));
        assert!(content.contains("Ctrl+Shift+1"));
        assert!(content.contains("Ctrl+Shift+0"));
    }

    #[test]
    fn test_generate_default_config_contains_profiles() {
        let content = generate_default_config();
        assert!(content.contains("profiles"));
        assert!(content.contains("name"));
        assert!(content.contains("icon"));
        assert!(content.contains("working_directory"));
        assert!(content.contains("hotkey"));
        // Should contain the uncommented "Neovim" profile
        assert!(content.contains("\"Neovim\""));
        assert!(content.contains("\"Ctrl+Shift+F1\""));
    }

    #[test]
    fn test_generate_default_config_uses_comment_syntax() {
        let content = generate_default_config();
        // Should contain // comments
        assert!(content.contains("//"));
    }

    #[test]
    fn test_generate_default_config_is_valid_jsonc() {
        let content = generate_default_config();
        // Strip comments using the same method as config loading
        let stripped = strip_jsonc_comments(content);

        // The stripped content should be valid JSON with profiles array
        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);
        assert!(
            result.is_ok(),
            "Stripped JSONC should be valid JSON: {}",
            stripped
        );

        // Verify it contains the Neovim profile
        let json = result.unwrap();
        assert!(json["profiles"].is_array());
        assert_eq!(json["profiles"][0]["name"], "Neovim");
    }

    #[test]
    fn test_config_dir_path() {
        let path = config_dir_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("neovide-tabs"));
        assert!(path.to_string_lossy().contains(".config"));
    }

    #[test]
    fn test_strip_jsonc_comments_line_comment() {
        let input = r#"{ // this is a comment
    "key": "value"
}"#;
        let output = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_strip_jsonc_comments_full_line_comment() {
        let input = r#"{
    // this is a full line comment
    "key": "value"
}"#;
        let output = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_strip_jsonc_comments_preserves_strings() {
        let input = r#"{ "key": "value with // not a comment" }"#;
        let output = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["key"], "value with // not a comment");
    }

    #[test]
    fn test_strip_jsonc_comments_escaped_quotes() {
        let input = r#"{ "key": "value with \" escaped // quote" }"#;
        let output = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["key"], r#"value with " escaped // quote"#);
    }

    #[test]
    fn test_strip_jsonc_comments_multiple_comments() {
        let input = r#"{
    // comment 1
    "a": 1, // inline comment
    // comment 2
    "b": 2
}"#;
        let output = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["b"], 2);
    }

    #[test]
    fn test_strip_jsonc_comments_no_comments() {
        let input = r#"{ "key": "value" }"#;
        let output = strip_jsonc_comments(input);
        assert_eq!(input, output);
    }

    #[test]
    fn test_strip_jsonc_comments_empty() {
        let input = "";
        let output = strip_jsonc_comments(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_strip_jsonc_parses_default_template() {
        let content = generate_default_config();
        let stripped = strip_jsonc_comments(content);
        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);
        assert!(
            result.is_ok(),
            "Default template should parse after stripping comments"
        );
    }

    // Title expansion tests

    #[test]
    fn test_expand_title_profile_name() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user/projects"),
            window_title: "file.rs - Neovim",
        };
        let result = expand_title("%p", &context);
        assert_eq!(result, "Work");
    }

    #[test]
    fn test_expand_title_window_title() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user/projects"),
            window_title: "file.rs - Neovim",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "file.rs - Neovim");
    }

    #[test]
    fn test_expand_title_working_directory_under_home() {
        let home = dirs::home_dir().unwrap();
        let projects_path = home.join("projects").join("myapp");
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &projects_path,
            window_title: "file.rs",
        };
        let result = expand_title("%w", &context);
        assert_eq!(result, "~/projects/myapp");
    }

    #[test]
    fn test_expand_title_working_directory_home() {
        let home = dirs::home_dir().unwrap();
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &home,
            window_title: "file.rs",
        };
        let result = expand_title("%w", &context);
        assert_eq!(result, "~");
    }

    #[test]
    fn test_expand_title_combined_tokens() {
        let home = dirs::home_dir().unwrap();
        let projects_path = home.join("projects");
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &projects_path,
            window_title: "file.rs",
        };
        let result = expand_title("%p: %w", &context);
        assert_eq!(result, "Work: ~/projects");
    }

    #[test]
    fn test_expand_title_strip_leading_dash() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "- Neovim",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "Neovim");
    }

    #[test]
    fn test_expand_title_strip_trailing_dash() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "Neovim -",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "Neovim");
    }

    #[test]
    fn test_expand_title_strip_whitespace() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "  Neovim  ",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "Neovim");
    }

    #[test]
    fn test_expand_title_preserve_internal_dash() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "file.rs - Neovim",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "file.rs - Neovim");
    }

    #[test]
    fn test_expand_title_empty_window_title() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "",
        };
        let result = expand_title("%t", &context);
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_title_literal_text() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "file.rs",
        };
        let result = expand_title("Tab: %p", &context);
        assert_eq!(result, "Tab: Work");
    }

    #[test]
    fn test_expand_title_escape_percent() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "file.rs",
        };
        let result = expand_title("100%% complete", &context);
        assert_eq!(result, "100% complete");
    }

    #[test]
    fn test_expand_title_unknown_token() {
        let context = TitleContext {
            profile_name: "Work",
            working_directory: &PathBuf::from("/home/user"),
            window_title: "file.rs",
        };
        // Unknown tokens like %x are kept as-is (just the %)
        let result = expand_title("%x", &context);
        assert_eq!(result, "%x");
    }

    #[test]
    fn test_sanitize_title_all_strip_chars() {
        let result = sanitize_title("---");
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_title_mixed_strip_chars() {
        let result = sanitize_title("- \t");
        assert_eq!(result, "");
    }

    #[test]
    fn test_profile_title_custom() {
        let profile_files = vec![ProfileFile {
            name: "Custom".to_string(),
            icon: None,
            working_directory: None,
            hotkey: None,
            title: Some("%p: %w".to_string()),
        }];
        let profiles = parse_profiles(Some(profile_files));
        assert_eq!(profiles[0].title, "%p: %w");
    }

    #[test]
    fn test_default_profile_has_title() {
        let profile = Profile::default_profile();
        assert_eq!(profile.title, DEFAULT_TITLE_FORMAT);
    }
}
