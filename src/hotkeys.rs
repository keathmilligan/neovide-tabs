//! Global hotkey registration and handling for neovide-tabs.
//!
//! Provides functionality to register system-wide hotkeys using Win32 API.

#![cfg(target_os = "windows")]

use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, RegisterHotKey, UnregisterHotKey,
    VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_B, VK_C, VK_D, VK_E, VK_F,
    VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_G,
    VK_H, VK_I, VK_J, VK_K, VK_L, VK_M, VK_N, VK_O, VK_P, VK_Q, VK_R, VK_S, VK_T, VK_U, VK_V, VK_W,
    VK_X, VK_Y, VK_Z,
};

/// Base ID for tab hotkeys (1-10)
#[allow(dead_code)]
pub const TAB_HOTKEY_BASE: i32 = 1;

/// Base ID for profile hotkeys (101+)
pub const PROFILE_HOTKEY_BASE: i32 = 101;

/// Parsed hotkey with modifiers and virtual key code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedHotkey {
    pub modifiers: HOT_KEY_MODIFIERS,
    pub vk: u32,
}

/// Parse a hotkey string like "Ctrl+Shift+F1" into modifiers and virtual key code.
/// Returns None if the format is invalid.
pub fn parse_hotkey_string(s: &str) -> Option<ParsedHotkey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = HOT_KEY_MODIFIERS(0);
    let mut key_part: Option<&str> = None;

    for part in parts {
        let part_upper = part.to_uppercase();
        match part_upper.as_str() {
            "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL,
            "ALT" => modifiers |= MOD_ALT,
            "SHIFT" => modifiers |= MOD_SHIFT,
            "WIN" | "WINDOWS" | "SUPER" => modifiers |= MOD_WIN,
            _ => {
                // This should be the key part
                if key_part.is_some() {
                    // Multiple non-modifier parts - invalid
                    return None;
                }
                key_part = Some(part);
            }
        }
    }

    // Must have at least one modifier and a key
    if modifiers.0 == 0 {
        eprintln!("Warning: Hotkey '{}' has no modifiers, skipping", s);
        return None;
    }

    let key = key_part?;
    let vk = parse_key_name(key)?;

    Some(ParsedHotkey { modifiers, vk })
}

/// Parse a key name to a virtual key code
fn parse_key_name(key: &str) -> Option<u32> {
    let key_upper = key.to_uppercase();

    // Function keys
    if let Some(rest) = key_upper.strip_prefix('F')
        && let Ok(num) = rest.parse::<u32>()
    {
        return match num {
            1 => Some(VK_F1.0 as u32),
            2 => Some(VK_F2.0 as u32),
            3 => Some(VK_F3.0 as u32),
            4 => Some(VK_F4.0 as u32),
            5 => Some(VK_F5.0 as u32),
            6 => Some(VK_F6.0 as u32),
            7 => Some(VK_F7.0 as u32),
            8 => Some(VK_F8.0 as u32),
            9 => Some(VK_F9.0 as u32),
            10 => Some(VK_F10.0 as u32),
            11 => Some(VK_F11.0 as u32),
            12 => Some(VK_F12.0 as u32),
            _ => None,
        };
    }

    // Single character (number or letter)
    if key.len() == 1 {
        let c = key.chars().next()?;
        return match c {
            '0' => Some(VK_0.0 as u32),
            '1' => Some(VK_1.0 as u32),
            '2' => Some(VK_2.0 as u32),
            '3' => Some(VK_3.0 as u32),
            '4' => Some(VK_4.0 as u32),
            '5' => Some(VK_5.0 as u32),
            '6' => Some(VK_6.0 as u32),
            '7' => Some(VK_7.0 as u32),
            '8' => Some(VK_8.0 as u32),
            '9' => Some(VK_9.0 as u32),
            'A' | 'a' => Some(VK_A.0 as u32),
            'B' | 'b' => Some(VK_B.0 as u32),
            'C' | 'c' => Some(VK_C.0 as u32),
            'D' | 'd' => Some(VK_D.0 as u32),
            'E' | 'e' => Some(VK_E.0 as u32),
            'F' | 'f' => Some(VK_F.0 as u32),
            'G' | 'g' => Some(VK_G.0 as u32),
            'H' | 'h' => Some(VK_H.0 as u32),
            'I' | 'i' => Some(VK_I.0 as u32),
            'J' | 'j' => Some(VK_J.0 as u32),
            'K' | 'k' => Some(VK_K.0 as u32),
            'L' | 'l' => Some(VK_L.0 as u32),
            'M' | 'm' => Some(VK_M.0 as u32),
            'N' | 'n' => Some(VK_N.0 as u32),
            'O' | 'o' => Some(VK_O.0 as u32),
            'P' | 'p' => Some(VK_P.0 as u32),
            'Q' | 'q' => Some(VK_Q.0 as u32),
            'R' | 'r' => Some(VK_R.0 as u32),
            'S' | 's' => Some(VK_S.0 as u32),
            'T' | 't' => Some(VK_T.0 as u32),
            'U' | 'u' => Some(VK_U.0 as u32),
            'V' | 'v' => Some(VK_V.0 as u32),
            'W' | 'w' => Some(VK_W.0 as u32),
            'X' | 'x' => Some(VK_X.0 as u32),
            'Y' | 'y' => Some(VK_Y.0 as u32),
            'Z' | 'z' => Some(VK_Z.0 as u32),
            _ => None,
        };
    }

    eprintln!("Warning: Unknown key name '{}', skipping", key);
    None
}

/// Register a global hotkey. Returns true if successful.
pub fn register_hotkey(hwnd: HWND, id: i32, hotkey: &ParsedHotkey) -> bool {
    unsafe {
        match RegisterHotKey(hwnd, id, hotkey.modifiers, hotkey.vk) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Warning: Failed to register hotkey ID {}: {}", id, e);
                false
            }
        }
    }
}

/// Unregister a global hotkey
pub fn unregister_hotkey(hwnd: HWND, id: i32) {
    unsafe {
        let _ = UnregisterHotKey(hwnd, id);
    }
}

/// Unregister all hotkeys from a list of IDs
pub fn unregister_all_hotkeys(hwnd: HWND, ids: &[i32]) {
    for &id in ids {
        unregister_hotkey(hwnd, id);
    }
}

/// Register tab hotkeys from configuration. Returns list of registered hotkey IDs.
pub fn register_tab_hotkeys(hwnd: HWND, tab_hotkeys: &HashMap<String, u32>) -> Vec<i32> {
    let mut registered = Vec::new();

    for (hotkey_str, &tab_num) in tab_hotkeys {
        if let Some(parsed) = parse_hotkey_string(hotkey_str) {
            // Tab numbers are 1-based, IDs are 1-10
            let id = tab_num as i32;
            if (1..=10).contains(&id) && register_hotkey(hwnd, id, &parsed) {
                registered.push(id);
            }
        } else {
            eprintln!("Warning: Invalid tab hotkey format: '{}'", hotkey_str);
        }
    }

    registered
}

/// Register profile hotkeys. Returns list of registered hotkey IDs.
/// Profile at index i gets hotkey ID = PROFILE_HOTKEY_BASE + i
pub fn register_profile_hotkeys(hwnd: HWND, profiles: &[crate::config::Profile]) -> Vec<i32> {
    let mut registered = Vec::new();

    for (index, profile) in profiles.iter().enumerate() {
        if let Some(ref hotkey_str) = profile.hotkey {
            if let Some(parsed) = parse_hotkey_string(hotkey_str) {
                let id = PROFILE_HOTKEY_BASE + index as i32;
                if register_hotkey(hwnd, id, &parsed) {
                    registered.push(id);
                }
            } else {
                eprintln!(
                    "Warning: Invalid hotkey format '{}' for profile '{}'",
                    hotkey_str, profile.name
                );
            }
        }
    }

    registered
}

/// Check if a hotkey ID is a tab hotkey (1-10)
pub fn is_tab_hotkey(id: i32) -> bool {
    (1..=10).contains(&id)
}

/// Check if a hotkey ID is a profile hotkey (101+)
pub fn is_profile_hotkey(id: i32) -> bool {
    id >= PROFILE_HOTKEY_BASE
}

/// Get the tab index (0-based) from a tab hotkey ID
pub fn tab_index_from_hotkey_id(id: i32) -> Option<usize> {
    if is_tab_hotkey(id) {
        // Hotkey ID 1 = tab index 0, ID 10 = tab index 9
        Some((id - 1) as usize)
    } else {
        None
    }
}

/// Get the profile index from a profile hotkey ID
pub fn profile_index_from_hotkey_id(id: i32) -> Option<usize> {
    if is_profile_hotkey(id) {
        Some((id - PROFILE_HOTKEY_BASE) as usize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_ctrl_shift_1() {
        let parsed = parse_hotkey_string("Ctrl+Shift+1").unwrap();
        assert_eq!(parsed.modifiers, MOD_CONTROL | MOD_SHIFT);
        assert_eq!(parsed.vk, VK_1.0 as u32);
    }

    #[test]
    fn test_parse_hotkey_ctrl_shift_f1() {
        let parsed = parse_hotkey_string("Ctrl+Shift+F1").unwrap();
        assert_eq!(parsed.modifiers, MOD_CONTROL | MOD_SHIFT);
        assert_eq!(parsed.vk, VK_F1.0 as u32);
    }

    #[test]
    fn test_parse_hotkey_alt_shift_a() {
        let parsed = parse_hotkey_string("Alt+Shift+A").unwrap();
        assert_eq!(parsed.modifiers, MOD_ALT | MOD_SHIFT);
        assert_eq!(parsed.vk, VK_A.0 as u32);
    }

    #[test]
    fn test_parse_hotkey_case_insensitive() {
        let parsed = parse_hotkey_string("ctrl+SHIFT+f1").unwrap();
        assert_eq!(parsed.modifiers, MOD_CONTROL | MOD_SHIFT);
        assert_eq!(parsed.vk, VK_F1.0 as u32);
    }

    #[test]
    fn test_parse_hotkey_control_alias() {
        let parsed = parse_hotkey_string("Control+Shift+1").unwrap();
        assert_eq!(parsed.modifiers, MOD_CONTROL | MOD_SHIFT);
    }

    #[test]
    fn test_parse_hotkey_win_modifier() {
        let parsed = parse_hotkey_string("Win+A").unwrap();
        assert_eq!(parsed.modifiers, MOD_WIN);
        assert_eq!(parsed.vk, VK_A.0 as u32);
    }

    #[test]
    fn test_parse_hotkey_windows_alias() {
        let parsed = parse_hotkey_string("Windows+A").unwrap();
        assert_eq!(parsed.modifiers, MOD_WIN);
    }

    #[test]
    fn test_parse_hotkey_super_alias() {
        let parsed = parse_hotkey_string("Super+A").unwrap();
        assert_eq!(parsed.modifiers, MOD_WIN);
    }

    #[test]
    fn test_parse_hotkey_no_modifier() {
        // No modifier should fail
        assert!(parse_hotkey_string("F1").is_none());
    }

    #[test]
    fn test_parse_hotkey_invalid_key() {
        // Invalid key should fail
        assert!(parse_hotkey_string("Ctrl+InvalidKey").is_none());
    }

    #[test]
    fn test_parse_hotkey_empty() {
        assert!(parse_hotkey_string("").is_none());
    }

    #[test]
    fn test_parse_hotkey_all_function_keys() {
        for i in 1..=12 {
            let s = format!("Ctrl+F{}", i);
            let parsed = parse_hotkey_string(&s);
            assert!(parsed.is_some(), "Failed to parse {}", s);
        }
    }

    #[test]
    fn test_parse_hotkey_all_numbers() {
        for i in 0..=9 {
            let s = format!("Ctrl+{}", i);
            let parsed = parse_hotkey_string(&s);
            assert!(parsed.is_some(), "Failed to parse {}", s);
        }
    }

    #[test]
    fn test_parse_hotkey_all_letters() {
        for c in 'A'..='Z' {
            let s = format!("Ctrl+{}", c);
            let parsed = parse_hotkey_string(&s);
            assert!(parsed.is_some(), "Failed to parse {}", s);
        }
    }

    #[test]
    fn test_is_tab_hotkey() {
        assert!(is_tab_hotkey(1));
        assert!(is_tab_hotkey(10));
        assert!(!is_tab_hotkey(0));
        assert!(!is_tab_hotkey(11));
        assert!(!is_tab_hotkey(101));
    }

    #[test]
    fn test_is_profile_hotkey() {
        assert!(is_profile_hotkey(101));
        assert!(is_profile_hotkey(112));
        assert!(!is_profile_hotkey(1));
        assert!(!is_profile_hotkey(100));
    }

    #[test]
    fn test_tab_index_from_hotkey_id() {
        assert_eq!(tab_index_from_hotkey_id(1), Some(0));
        assert_eq!(tab_index_from_hotkey_id(10), Some(9));
        assert_eq!(tab_index_from_hotkey_id(101), None);
    }

    #[test]
    fn test_profile_index_from_hotkey_id() {
        assert_eq!(profile_index_from_hotkey_id(101), Some(0));
        assert_eq!(profile_index_from_hotkey_id(102), Some(1));
        assert_eq!(profile_index_from_hotkey_id(1), None);
    }
}
