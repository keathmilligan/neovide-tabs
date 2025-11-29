## MODIFIED Requirements

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

## ADDED Requirements

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
