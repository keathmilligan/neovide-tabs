## ADDED Requirements

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
