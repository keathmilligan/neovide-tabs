# window-management Specification

## Purpose
TBD - created by archiving change add-windows-scaffold. Update Purpose after archive.
## Requirements
### Requirement: Application Window Creation
The system SHALL create a native Windows application window with a custom title bar on startup.

#### Scenario: Successful window creation with custom title bar
- **WHEN** the application is launched
- **THEN** a window SHALL be created without standard Windows title bar decorations
- **AND** a custom title bar region of 32 pixels height SHALL be rendered at the top of the window
- **AND** the window SHALL have a default size of 1024x768 pixels
- **AND** the window SHALL be centered on the primary monitor
- **AND** the window SHALL retain resize borders for standard resize behavior

#### Scenario: Window creation failure
- **WHEN** the application fails to create a window (e.g., insufficient system resources)
- **THEN** an error message SHALL be logged
- **AND** the application SHALL exit with a non-zero exit code

### Requirement: Neovide Process Discovery
The system SHALL verify that Neovide is available before attempting to launch it.

#### Scenario: Neovide found in PATH
- **WHEN** the application starts
- **AND** the "neovide" executable is found in the system PATH
- **THEN** the application SHALL proceed to spawn the Neovide process

#### Scenario: Neovide not found
- **WHEN** the application starts
- **AND** the "neovide" executable is NOT found in the system PATH
- **THEN** a MessageBox error dialog SHALL be displayed with the message "Neovide not found in PATH. Please install Neovide and ensure it is accessible."
- **AND** the application SHALL exit gracefully with exit code 1

### Requirement: Neovide Process Spawning
The system SHALL spawn Neovide instances with frameless configuration, supporting multiple concurrent instances with configurable working directories.

#### Scenario: Successful Neovide launch
- **WHEN** a tab is created (including the initial tab on startup)
- **AND** the client area dimensions are calculated
- **THEN** a Neovide process SHALL be spawned with the following arguments:
  - `--frame none` (removes window decorations)
  - `--size WxH` (where W and H are the content area width and height in pixels)
- **AND** the working directory SHALL be set to the profile's configured working directory
- **AND** the process handle SHALL be associated with the corresponding tab

#### Scenario: Neovide spawn failure for new tab
- **WHEN** a new tab's Neovide process fails to spawn
- **THEN** a MessageBox error dialog SHALL be displayed with the spawn error details
- **AND** the tab creation SHALL be cancelled (no new tab added)
- **AND** the previously selected tab SHALL remain selected

### Requirement: Window Sizing and Positioning
The system SHALL calculate and apply correct dimensions for the embedded Neovide window.

#### Scenario: Initial window sizing
- **WHEN** the wrapper window is created
- **THEN** the client area dimensions SHALL be calculated using Win32 GetClientRect
- **AND** the Neovide process SHALL be spawned with `--size WxH` matching the client area dimensions

#### Scenario: Window resize handling
- **WHEN** the user resizes the wrapper window
- **THEN** a WM_SIZE event SHALL be processed
- **AND** the client area dimensions SHALL be recalculated
- **AND** the Neovide window SHALL be resized to match the new client area dimensions using Win32 SetWindowPos

#### Scenario: Minimum window size enforcement
- **WHEN** the user attempts to resize the wrapper window below 800x600 pixels
- **THEN** the window SHALL be constrained to a minimum size of 800x600 pixels
- **AND** the Neovide instance SHALL not be resized smaller than the enforced minimum

### Requirement: Process Lifecycle Management
The system SHALL manage the lifecycle of multiple Neovide processes, detect process exits, and ensure proper cleanup with graceful shutdown support.

#### Scenario: Graceful shutdown on wrapper close
- **WHEN** the user closes the wrapper window (via title bar close button or Alt+F4)
- **THEN** the system SHALL send a WM_CLOSE message to each Neovide window
- **AND** if a Neovide instance prompts the user to save unsaved files, the prompt SHALL be displayed
- **AND** if the user cancels any close prompt, that Neovide process SHALL continue running
- **AND** the existing process polling SHALL detect when each Neovide process exits
- **AND** tabs SHALL be removed as their associated processes exit
- **AND** the application SHALL close when the last process exits (handled by existing polling logic)
- **AND** Neovide processes that were launched externally (not by neovide-tabs) SHALL NOT be affected

#### Scenario: Individual Neovide process exit detection
- **WHEN** a Neovide process exits (via user action like `:q`, crash, or external termination)
- **THEN** the application SHALL detect the process exit within 500 milliseconds
- **AND** the corresponding tab SHALL be removed from the tab bar
- **AND** if the exited process was the selected tab, the next available tab SHALL be selected
- **AND** the tab bar SHALL be repainted to reflect the change

#### Scenario: Last Neovide process exits
- **WHEN** the last remaining Neovide process exits
- **THEN** the application SHALL detect the exit
- **AND** the corresponding tab SHALL be removed
- **AND** the application window SHALL be closed
- **AND** the application SHALL exit with exit code 0

#### Scenario: Process tracking scope
- **WHEN** the application terminates processes on shutdown
- **THEN** only processes spawned by neovide-tabs (tracked via child process handles) SHALL be terminated
- **AND** Neovide instances launched independently by the user SHALL continue running

### Requirement: Error Reporting
The system SHALL provide clear error messages for common failure scenarios.

#### Scenario: Display error for missing Neovide
- **WHEN** Neovide is not found in PATH
- **THEN** a Win32 MessageBox SHALL be displayed with the title "Error: Neovide Not Found"
- **AND** the message body SHALL provide installation instructions

#### Scenario: Display error for spawn failures
- **WHEN** the Neovide process fails to spawn
- **THEN** a Win32 MessageBox SHALL be displayed with the title "Error: Failed to Launch Neovide"
- **AND** the message body SHALL include the system error details

#### Scenario: Display error for unexpected process exit
- **WHEN** the Neovide process exits with a non-zero exit code
- **THEN** a Win32 MessageBox SHALL be displayed with the title "Error: Neovide Exited"
- **AND** the message body SHALL include the exit code

### Requirement: Platform Constraint
The system SHALL be implemented exclusively for Windows.

#### Scenario: Windows-only compilation
- **WHEN** the code is compiled
- **THEN** platform-specific code SHALL use `#[cfg(target_os = "windows")]` attributes
- **AND** the application SHALL only build successfully on Windows targets

#### Scenario: Attempt to run on non-Windows platform
- **WHEN** the application is compiled for or run on a non-Windows platform
- **THEN** compilation SHALL fail with a clear error message indicating Windows-only support

### Requirement: Custom Title Bar Rendering
The system SHALL render a custom title bar with app icon, title text, and window control buttons.

#### Scenario: Title bar content display
- **WHEN** the window is displayed
- **THEN** the title bar region SHALL be filled with the configured background color
- **AND** the application icon SHALL be displayed on the left side of the title bar
- **AND** the window title "neovide-tabs" SHALL be displayed after the icon
- **AND** minimize, maximize, and close buttons SHALL be displayed on the right side of the title bar

#### Scenario: Title bar repainting
- **WHEN** the window receives a paint message
- **THEN** the title bar content SHALL be redrawn with current state (e.g., maximized button icon changes based on window state)

### Requirement: Custom Title Bar Hit Testing
The system SHALL handle mouse interactions with the custom title bar region.

#### Scenario: Title bar drag to move window
- **WHEN** the user clicks and drags on the title bar region (excluding buttons)
- **THEN** the window SHALL move following the mouse cursor
- **AND** Windows snap gestures (drag to edge, shake to minimize) SHALL function normally

#### Scenario: Title bar double-click to maximize/restore
- **WHEN** the user double-clicks on the title bar region (excluding buttons)
- **THEN** the window SHALL toggle between maximized and restored states

#### Scenario: Minimize button click
- **WHEN** the user clicks the minimize button
- **THEN** the window SHALL be minimized to the taskbar

#### Scenario: Maximize button click when restored
- **WHEN** the user clicks the maximize button
- **AND** the window is in restored state
- **THEN** the window SHALL be maximized to fill the screen

#### Scenario: Maximize button click when maximized
- **WHEN** the user clicks the maximize button
- **AND** the window is in maximized state
- **THEN** the window SHALL be restored to its previous size and position

#### Scenario: Close button click
- **WHEN** the user clicks the close button
- **THEN** the window SHALL initiate the close sequence (same as clicking standard close button)

### Requirement: Title Bar Button Visual Feedback
The system SHALL provide visual feedback for title bar button interactions.

#### Scenario: Button hover state
- **WHEN** the mouse cursor hovers over a title bar button
- **THEN** the button background SHALL change to indicate hover state

#### Scenario: Close button hover state
- **WHEN** the mouse cursor hovers over the close button
- **THEN** the button background SHALL change to a red color to indicate destructive action

#### Scenario: Button mouse leave
- **WHEN** the mouse cursor leaves a title bar button
- **THEN** the button background SHALL return to the normal state

### Requirement: Window Client Area Adjustment
The system SHALL correctly calculate the client area accounting for the custom title bar.

#### Scenario: Client area excludes title bar
- **WHEN** the client area dimensions are calculated
- **THEN** the client area SHALL begin below the 32-pixel title bar region
- **AND** the Neovide process SHALL be positioned to fill the client area below the title bar

#### Scenario: Window resize with custom title bar
- **WHEN** the user resizes the window
- **THEN** the title bar SHALL remain at 32 pixels height
- **AND** the client area SHALL be recalculated excluding the title bar
- **AND** the embedded Neovide window SHALL be resized to match the new client area

### Requirement: Tab Bar Display
The system SHALL display a tab bar within the titlebar area for managing multiple Neovide instances, showing profile icons and names.

#### Scenario: Initial tab display
- **WHEN** the application is launched
- **THEN** a tab bar SHALL be displayed in the titlebar area, to the left of the window control buttons
- **AND** a single tab SHALL be displayed using the Default profile
- **AND** the tab SHALL display the profile's icon (16x16 pixels) followed by the profile's name
- **AND** a new tab (+) button SHALL be displayed to the right of the last tab
- **AND** a profile dropdown button (downward caret) SHALL be displayed to the right of the new tab (+) button

#### Scenario: Tab visual styling
- **WHEN** tabs are displayed
- **THEN** each tab SHALL have a subtle 1px outline to distinguish it from the titlebar background
- **AND** the selected tab SHALL have a visually distinct background (matching or lighter than titlebar)
- **AND** unselected tabs SHALL have a slightly darker background than the selected tab
- **AND** the content area SHALL have a subtle outline to distinguish it from the titlebar
- **AND** each tab SHALL display its profile icon on the left side, appropriately sized for the tab height
- **AND** each tab SHALL display the profile name to the right of the icon

### Requirement: Tab Creation
The system SHALL allow users to create new tabs, each hosting a new Neovide instance, using the Default profile when clicking + or a selected profile from the dropdown.

#### Scenario: Create new tab via button
- **WHEN** the user clicks the new tab (+) button
- **THEN** a new Neovide process SHALL be spawned using the Default profile
- **AND** the working directory SHALL be set to the Default profile's configured directory
- **AND** a new tab SHALL be added to the right of existing tabs
- **AND** the new tab SHALL display the Default profile's icon and name
- **AND** the new tab SHALL become the selected tab
- **AND** the new Neovide instance SHALL be brought to the foreground

#### Scenario: Create new tab via profile dropdown
- **WHEN** the user clicks the profile dropdown button
- **AND** the user selects a profile from the dropdown menu
- **THEN** a new Neovide process SHALL be spawned using the selected profile
- **AND** the working directory SHALL be set to the selected profile's configured directory
- **AND** a new tab SHALL be added to the right of existing tabs
- **AND** the new tab SHALL display the selected profile's icon and name
- **AND** the new tab SHALL become the selected tab
- **AND** the new Neovide instance SHALL be brought to the foreground

### Requirement: Tab Selection
The system SHALL allow users to switch between tabs by clicking on them.

#### Scenario: Select a different tab
- **WHEN** the user clicks on an unselected tab
- **THEN** that tab SHALL become the selected tab
- **AND** the previously selected tab's Neovide window SHALL be hidden
- **AND** the newly selected tab's Neovide window SHALL be shown and brought to the foreground
- **AND** the tab bar SHALL be repainted to reflect the new selection state

#### Scenario: Click already selected tab
- **WHEN** the user clicks on the already selected tab
- **THEN** no change SHALL occur to the tab selection
- **AND** the Neovide window associated with that tab SHALL be brought to the foreground

### Requirement: Tab Closing
The system SHALL allow users to close individual tabs via a close button on each tab, requesting graceful closure from the associated Neovide process.

#### Scenario: Close tab with close button
- **WHEN** the user clicks the close (x) button on a tab
- **THEN** a WM_CLOSE message SHALL be sent to the Neovide window associated with that tab
- **AND** Neovide SHALL be allowed to prompt the user to save unsaved files if needed
- **AND** if the user confirms the close (or there are no unsaved files), the Neovide process SHALL exit
- **AND** the existing process polling SHALL detect the exit and remove the tab
- **AND** if the closed tab was selected, the next tab (or previous if no next) SHALL become selected
- **AND** the tab bar SHALL be repainted

#### Scenario: Close tab cancelled by user
- **WHEN** the user clicks the close (x) button on a tab
- **AND** Neovide prompts the user to save unsaved files
- **AND** the user cancels the close prompt (chooses not to close)
- **THEN** the Neovide process SHALL continue running
- **AND** the tab SHALL remain in the tab bar
- **AND** no changes SHALL be made to the tab selection state

#### Scenario: Close the last remaining tab
- **WHEN** the user closes the only remaining tab via the close button
- **AND** the Neovide process exits (user confirms close or no unsaved files)
- **THEN** the existing process polling SHALL detect the exit
- **AND** the application window SHALL be closed
- **AND** the application SHALL exit with exit code 0

#### Scenario: Close last tab cancelled by user
- **WHEN** the user clicks close on the only remaining tab
- **AND** the user cancels the close prompt in Neovide
- **THEN** the tab and Neovide process SHALL remain active
- **AND** the application SHALL continue running

### Requirement: Tab Reordering
The system SHALL allow users to reorder tabs by dragging them within the tab bar.

#### Scenario: Drag tab to new position
- **WHEN** the user presses and holds the left mouse button on a tab
- **AND** the user drags the mouse horizontally beyond a threshold (5 pixels)
- **THEN** visual feedback SHALL indicate the drag operation is in progress
- **AND** the potential drop position SHALL be indicated

#### Scenario: Drop tab at new position
- **WHEN** the user releases the mouse button after dragging a tab
- **THEN** the tab SHALL be repositioned in the tab order at the drop location
- **AND** the tab bar SHALL be repainted to reflect the new order
- **AND** tab selection state SHALL be preserved (the dragged tab remains selected if it was selected)

#### Scenario: Cancel drag operation
- **WHEN** the user drags a tab outside the tab bar area and releases
- **THEN** the drag operation SHALL be cancelled
- **AND** the tab SHALL remain in its original position

### Requirement: Process Exit Polling
The system SHALL continuously monitor spawned Neovide processes for unexpected exits.

#### Scenario: Periodic process status check
- **WHEN** the application is running with one or more tabs
- **THEN** a timer SHALL poll all Neovide process statuses at a regular interval (250-500ms)
- **AND** the timer SHALL use Win32 `SetTimer` with a dedicated timer ID

#### Scenario: Process exit detected during poll
- **WHEN** a Neovide process is detected as exited during a status poll
- **THEN** the system SHALL handle the exit as specified in the Process Lifecycle Management requirement
- **AND** multiple simultaneous process exits SHALL be handled correctly in a single poll cycle

### Requirement: Graceful Neovide Window Close
The system SHALL request graceful closure of Neovide windows by sending WM_CLOSE messages instead of forcefully terminating processes.

#### Scenario: Send close request to Neovide window
- **WHEN** a graceful close is requested for a tab
- **THEN** the system SHALL send a WM_CLOSE message to the associated Neovide window handle
- **AND** the system SHALL NOT forcefully terminate the process
- **AND** the system SHALL NOT immediately remove the tab
- **AND** the system SHALL rely on the existing process exit polling to detect when Neovide exits and remove the tab

#### Scenario: Neovide window not yet ready
- **WHEN** a close is requested for a tab whose Neovide window has not been discovered yet
- **THEN** the system SHALL forcefully terminate the process using the child process handle
- **AND** the tab SHALL be removed immediately

### Requirement: Profile Dropdown Display
The system SHALL display a dropdown menu for selecting profiles when creating new tabs.

#### Scenario: Dropdown button appearance
- **WHEN** the tab bar is rendered
- **THEN** a dropdown button with a downward caret icon SHALL be displayed immediately to the right of the new tab (+) button
- **AND** the dropdown button SHALL be 16 pixels wide
- **AND** the dropdown button SHALL have the same height as the new tab button

#### Scenario: Dropdown button hover state
- **WHEN** the mouse cursor hovers over the profile dropdown button
- **THEN** the button background SHALL change to indicate hover state

#### Scenario: Dropdown menu display
- **WHEN** the user clicks the profile dropdown button
- **THEN** a dropdown menu SHALL appear below the button
- **AND** the menu SHALL list all configured profiles
- **AND** each menu item SHALL display the profile's icon (16x16 pixels) followed by the profile's name
- **AND** the menu items SHALL be ordered as defined in the configuration (with generated Default first if applicable)

#### Scenario: Dropdown menu selection
- **WHEN** the user clicks on a profile in the dropdown menu
- **THEN** the dropdown menu SHALL close
- **AND** a new tab SHALL be created using the selected profile

#### Scenario: Dropdown menu dismiss
- **WHEN** the dropdown menu is open
- **AND** the user clicks outside the dropdown menu
- **THEN** the dropdown menu SHALL close without creating a new tab

### Requirement: Tab Profile Association
The system SHALL associate each tab with its source profile for display purposes.

#### Scenario: Tab stores profile reference
- **WHEN** a new tab is created with a profile
- **THEN** the tab SHALL store a reference to the profile used to create it
- **AND** the tab SHALL use the profile's icon and name for display

#### Scenario: Tab tooltip display
- **WHEN** the user hovers over a tab
- **THEN** a tooltip SHALL display the full profile name and working directory path

### Requirement: Global Hotkey Registration

The system SHALL register global hotkeys using the Win32 RegisterHotKey API at application startup, only for profiles that exist and have hotkeys configured.

#### Scenario: Successful hotkey registration

- **WHEN** the application window is created
- **AND** the hotkey configuration is loaded
- **THEN** tab hotkeys SHALL be registered with IDs 1-10
- **AND** profile hotkeys SHALL be registered with IDs 101+ (101 + profile_index)
- **AND** profile hotkeys SHALL only be registered for profiles that have a `hotkey` field
- **AND** the hotkeys SHALL be active system-wide regardless of focused application

#### Scenario: Only existing profile hotkeys registered

- **WHEN** the application starts with only the Default profile
- **AND** the Default profile has hotkey `Ctrl+Shift+F1`
- **THEN** only `Ctrl+Shift+F1` SHALL be registered as a profile hotkey
- **AND** `Ctrl+Shift+F2` through `Ctrl+Shift+F12` SHALL NOT be registered

#### Scenario: Hotkey registration conflict

- **WHEN** `RegisterHotKey` fails for a specific hotkey
- **AND** the error indicates the hotkey is already registered by another application
- **THEN** a warning SHALL be logged indicating which hotkey could not be registered
- **AND** the application SHALL continue startup without the conflicting hotkey
- **AND** other hotkeys SHALL still be registered

#### Scenario: Hotkey cleanup on exit

- **WHEN** the application window is destroyed
- **THEN** all registered hotkeys SHALL be unregistered using `UnregisterHotKey`

### Requirement: Tab Activation via Global Hotkey

The system SHALL activate existing tabs when their associated global hotkeys are pressed.

#### Scenario: Activate tab by number hotkey

- **WHEN** a tab activation hotkey is pressed (e.g., `Ctrl+Shift+1`)
- **AND** a tab exists at the corresponding index (1-based: hotkey 1 = tab index 0)
- **THEN** the tab at that index SHALL be selected
- **AND** the wrapper window SHALL be brought to the foreground
- **AND** the tab's Neovide window SHALL be activated

#### Scenario: Tab hotkey for non-existent tab

- **WHEN** a tab activation hotkey is pressed
- **AND** no tab exists at the corresponding index
- **THEN** no action SHALL be taken
- **AND** no error SHALL be displayed

#### Scenario: Tab hotkey for already selected tab

- **WHEN** a tab activation hotkey is pressed
- **AND** the corresponding tab is already selected
- **THEN** the wrapper window SHALL be brought to the foreground
- **AND** the tab's Neovide window SHALL be activated

### Requirement: Profile Activation via Global Hotkey

The system SHALL open or activate profiles when their associated global hotkeys are pressed.

#### Scenario: Activate existing profile tab

- **WHEN** a profile activation hotkey is pressed (e.g., `Ctrl+Shift+F1` for Default)
- **AND** a tab with the corresponding profile already exists
- **THEN** the first tab with that profile SHALL be selected
- **AND** the wrapper window SHALL be brought to the foreground
- **AND** the tab's Neovide window SHALL be activated

#### Scenario: Open new profile tab

- **WHEN** a profile activation hotkey is pressed
- **AND** no tab with the corresponding profile exists
- **THEN** a new tab SHALL be created using that profile
- **AND** the new tab SHALL become the selected tab
- **AND** the wrapper window SHALL be brought to the foreground

### Requirement: Window Focus Handling for Global Hotkeys

The system SHALL properly handle window focus when responding to global hotkeys.

#### Scenario: Bring window to foreground from background

- **WHEN** a global hotkey is received via WM_HOTKEY
- **AND** the wrapper window is not in the foreground
- **THEN** the wrapper window SHALL be restored if minimized
- **AND** the wrapper window SHALL be brought to the foreground using `SetForegroundWindow`
- **AND** the selected tab's Neovide window SHALL be activated

#### Scenario: Wrapper window already in foreground

- **WHEN** a global hotkey is received via WM_HOTKEY
- **AND** the wrapper window is already in the foreground
- **THEN** the tab selection or profile action SHALL be performed
- **AND** no additional focus operations SHALL be required

