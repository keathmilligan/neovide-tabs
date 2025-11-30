#![cfg(target_os = "windows")]

use anyhow::Result;
use windows::Win32::Foundation::HWND;

use crate::config::Profile;
use crate::process::NeovideProcess;

/// Represents a single tab with its associated Neovide process
pub struct Tab {
    /// Unique identifier for this tab
    #[allow(dead_code)]
    pub id: usize,
    /// The Neovide process associated with this tab
    pub process: NeovideProcess,
    /// Profile name used to create this tab
    pub profile_name: String,
    /// Profile icon filename
    pub profile_icon: String,
    /// Profile working directory (for tooltip display)
    #[allow(dead_code)]
    pub working_directory: std::path::PathBuf,
    /// Profile index in the config (for reference)
    pub profile_index: usize,
}

/// State for tab drag-and-drop reordering
#[derive(Debug, Clone)]
pub struct DragState {
    /// Index of the tab being dragged (updated in real-time as swaps occur)
    pub tab_index: usize,
    /// Initial mouse X position when drag started
    pub start_x: i32,
    /// Current mouse X position
    pub current_x: i32,
    /// Original X position of the tab's left edge when drag started
    pub tab_start_left: i32,
}

impl DragState {
    /// Check if the drag has moved beyond the threshold to be considered active
    pub fn is_active(&self) -> bool {
        (self.current_x - self.start_x).abs() > 5
    }

    /// Get the visual X position for the dragged tab
    pub fn get_visual_x(&self) -> i32 {
        self.tab_start_left + (self.current_x - self.start_x)
    }
}

/// Manages multiple tabs and their associated Neovide processes
pub struct TabManager {
    /// All tabs in display order
    tabs: Vec<Tab>,
    /// Index of the currently selected tab
    selected_index: usize,
    /// Counter for generating unique tab IDs
    next_id: usize,
    /// Current drag state (if dragging)
    pub drag_state: Option<DragState>,
}

impl TabManager {
    /// Create a new TabManager with no tabs
    pub fn new() -> Self {
        TabManager {
            tabs: Vec::new(),
            selected_index: 0,
            next_id: 1,
            drag_state: None,
        }
    }

    /// Get the number of tabs
    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if there are no tabs
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get the currently selected tab index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get a reference to a tab by index
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    /// Get a mutable reference to a tab by index
    #[allow(dead_code)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    /// Get the currently selected tab
    pub fn selected_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.selected_index)
    }

    /// Get a mutable reference to the currently selected tab
    #[allow(dead_code)]
    pub fn selected_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.selected_index)
    }

    /// Create a new tab with a spawned Neovide process using a profile
    /// Returns the index of the new tab, or an error if spawning failed
    pub fn create_tab(
        &mut self,
        width: u32,
        height: u32,
        parent_hwnd: HWND,
        profile: &Profile,
        profile_index: usize,
    ) -> Result<usize> {
        let process = NeovideProcess::spawn(
            width,
            height,
            parent_hwnd,
            Some(profile.working_directory.as_path()),
        )?;

        let tab = Tab {
            id: self.next_id,
            process,
            profile_name: profile.name.clone(),
            profile_icon: profile.icon.clone(),
            working_directory: profile.working_directory.clone(),
            profile_index,
        };
        self.next_id += 1;

        self.tabs.push(tab);
        let new_index = self.tabs.len() - 1;
        self.selected_index = new_index;

        Ok(new_index)
    }

    /// Create a new tab with a spawned Neovide process (legacy, uses no working directory)
    /// Returns the index of the new tab, or an error if spawning failed
    #[allow(dead_code)]
    pub fn create_tab_simple(
        &mut self,
        width: u32,
        height: u32,
        parent_hwnd: HWND,
    ) -> Result<usize> {
        let process = NeovideProcess::spawn(width, height, parent_hwnd, None)?;

        let tab = Tab {
            id: self.next_id,
            process,
            profile_name: "Default".to_string(),
            profile_icon: crate::config::DEFAULT_ICON.to_string(),
            working_directory: dirs::home_dir().unwrap_or_default(),
            profile_index: 0,
        };
        self.next_id += 1;

        self.tabs.push(tab);
        let new_index = self.tabs.len() - 1;
        self.selected_index = new_index;

        Ok(new_index)
    }

    /// Select a tab by index
    /// Returns true if the selection changed
    pub fn select_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() && index != self.selected_index {
            self.selected_index = index;
            true
        } else {
            false
        }
    }

    /// Close a tab by index, terminating its Neovide process
    /// Returns true if this was the last tab (caller should close the window)
    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        // Remove and drop the tab (which terminates the process via Drop)
        let mut tab = self.tabs.remove(index);
        let _ = tab.process.terminate();

        if self.tabs.is_empty() {
            return true; // Last tab closed
        }

        // Adjust selected index if needed
        if self.selected_index >= self.tabs.len() {
            self.selected_index = self.tabs.len() - 1;
        } else if self.selected_index > index {
            self.selected_index -= 1;
        }

        false
    }

    /// Move a tab from one position to another
    pub fn move_tab(&mut self, from_index: usize, to_index: usize) {
        if from_index >= self.tabs.len() || to_index >= self.tabs.len() || from_index == to_index {
            return;
        }

        let tab = self.tabs.remove(from_index);
        self.tabs.insert(to_index, tab);

        // Update selected index to follow the moved tab if it was selected
        if self.selected_index == from_index {
            self.selected_index = to_index;
        } else if from_index < self.selected_index && to_index >= self.selected_index {
            self.selected_index -= 1;
        } else if from_index > self.selected_index && to_index <= self.selected_index {
            self.selected_index += 1;
        }
    }

    /// Update the position of all Neovide windows (only moves if needed)
    pub fn update_all_positions(&self, parent_hwnd: HWND, titlebar_height: i32) {
        for tab in &self.tabs {
            tab.process.update_position(parent_hwnd, titlebar_height);
        }
    }

    /// Activate the selected tab: ensure position, show it, hide others, bring to foreground
    /// This is the main method for switching tabs
    pub fn activate_selected(&self, parent_hwnd: HWND, titlebar_height: i32) {
        for (i, tab) in self.tabs.iter().enumerate() {
            if i == self.selected_index {
                // Use the combined activate method which handles position check + show + foreground
                tab.process.activate(parent_hwnd, titlebar_height);
            } else {
                tab.process.hide();
            }
        }
    }

    /// Show the selected tab's Neovide window and hide all others
    /// Note: Prefer activate_selected() when parent_hwnd is available
    #[allow(dead_code)]
    pub fn show_selected_hide_others(&self) {
        for (i, tab) in self.tabs.iter().enumerate() {
            if i == self.selected_index {
                tab.process.show();
                tab.process.bring_to_foreground();
            } else {
                tab.process.hide();
            }
        }
    }

    /// Bring the selected tab's Neovide to foreground (just foreground, no position check)
    #[allow(dead_code)]
    pub fn bring_selected_to_foreground(&self) {
        if let Some(tab) = self.selected_tab() {
            tab.process.bring_to_foreground();
        }
    }

    /// Activate the selected tab with position check, then bring to foreground
    pub fn activate_and_foreground_selected(&self, parent_hwnd: HWND, titlebar_height: i32) {
        if let Some(tab) = self.selected_tab() {
            tab.process.activate(parent_hwnd, titlebar_height);
        }
    }

    /// Check if the selected tab's process is ready
    pub fn is_selected_ready(&self) -> bool {
        self.selected_tab()
            .is_some_and(|tab| tab.process.is_ready())
    }

    /// Terminate all tabs' processes forcefully
    #[allow(dead_code)]
    pub fn terminate_all(&mut self) {
        for tab in &mut self.tabs {
            let _ = tab.process.terminate();
        }
        self.tabs.clear();
    }

    /// Request graceful close for a tab by sending WM_CLOSE to its Neovide window.
    /// If the window is not ready, falls back to forceful termination via close_tab().
    /// Returns true if graceful close was requested (tab remains until process exits),
    /// false if forceful close was used (tab already removed).
    pub fn request_close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        // Try to send WM_CLOSE to the Neovide window
        if self.tabs[index].process.request_close() {
            // Message sent successfully - tab remains until process polling detects exit
            true
        } else {
            // Window not ready - fall back to forceful close
            self.close_tab(index);
            false
        }
    }

    /// Request graceful close for all tabs by sending WM_CLOSE to each Neovide window.
    /// For tabs where window is not ready, forcefully terminates them.
    /// Does not remove tabs - process polling will handle removal as processes exit.
    pub fn request_close_all(&mut self) {
        // Collect indices of tabs that need forceful termination
        let mut force_close_indices = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            if !tab.process.request_close() {
                // Window not ready - mark for forceful termination
                force_close_indices.push(i);
            }
        }

        // Forcefully close tabs without ready windows (in reverse order for safe removal)
        force_close_indices.reverse();
        for index in force_close_indices {
            self.close_tab(index);
        }
    }

    /// Get the label for a tab (profile name)
    pub fn get_tab_label(&self, index: usize) -> String {
        if let Some(tab) = self.tabs.get(index) {
            tab.profile_name.clone()
        } else {
            String::new()
        }
    }

    /// Get the profile index for a tab
    #[allow(dead_code)]
    pub fn get_tab_profile_index(&self, index: usize) -> Option<usize> {
        self.tabs.get(index).map(|tab| tab.profile_index)
    }

    /// Find the first tab with the given profile index.
    /// Returns the tab index if found, None otherwise.
    pub fn find_tab_by_profile_index(&self, profile_index: usize) -> Option<usize> {
        self.tabs
            .iter()
            .position(|tab| tab.profile_index == profile_index)
    }

    /// Get the icon filename for a tab
    pub fn get_tab_icon(&self, index: usize) -> Option<&str> {
        self.tabs.get(index).map(|tab| tab.profile_icon.as_str())
    }

    /// Get the working directory for a tab (for tooltip display)
    #[allow(dead_code)]
    pub fn get_tab_working_directory(&self, index: usize) -> Option<&std::path::Path> {
        self.tabs
            .get(index)
            .map(|tab| tab.working_directory.as_path())
    }

    /// Iterate over all tabs with their indices
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Tab)> {
        self.tabs.iter().enumerate()
    }

    /// Find indices of tabs whose Neovide processes have exited.
    /// Returns indices in reverse order (highest first) to allow safe removal.
    pub fn find_exited_tabs(&self) -> Vec<usize> {
        let mut exited = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            if !tab.process.is_running() {
                exited.push(i);
            }
        }
        // Reverse so we can remove from highest index first without invalidating lower indices
        exited.reverse();
        exited
    }

    /// Remove a tab without terminating its process (for already-exited processes).
    /// Returns true if this was the last tab.
    pub fn remove_exited_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        // Just remove the tab - don't call terminate() since process already exited
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            return true;
        }

        // Adjust selected index if needed
        if self.selected_index >= self.tabs.len() {
            self.selected_index = self.tabs.len() - 1;
        } else if self.selected_index > index {
            self.selected_index -= 1;
        }

        false
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests requiring NeovideProcess cannot be run without Windows,
    // but we can test the basic TabManager logic with mock data

    #[test]
    fn test_tab_manager_new() {
        let manager = TabManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.selected_index(), 0);
    }

    #[test]
    fn test_drag_state_threshold() {
        let drag = DragState {
            tab_index: 0,
            start_x: 100,
            current_x: 100,
            tab_start_left: 8,
        };
        assert!(!drag.is_active());

        let drag = DragState {
            tab_index: 0,
            start_x: 100,
            current_x: 106,
            tab_start_left: 8,
        };
        assert!(drag.is_active());

        let drag = DragState {
            tab_index: 0,
            start_x: 100,
            current_x: 94,
            tab_start_left: 8,
        };
        assert!(drag.is_active());
    }

    #[test]
    fn test_drag_state_visual_x() {
        let drag = DragState {
            tab_index: 0,
            start_x: 100,
            current_x: 150,
            tab_start_left: 8,
        };
        // Visual X should be tab_start_left + (current_x - start_x)
        // = 8 + (150 - 100) = 8 + 50 = 58
        assert_eq!(drag.get_visual_x(), 58);

        let drag = DragState {
            tab_index: 1,
            start_x: 200,
            current_x: 150,
            tab_start_left: 128,
        };
        // Visual X = 128 + (150 - 200) = 128 - 50 = 78
        assert_eq!(drag.get_visual_x(), 78);
    }
}
