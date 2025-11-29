## ADDED Requirements

### Requirement: Tab Bar Display
The system SHALL display a tab bar within the titlebar area for managing multiple Neovide instances.

#### Scenario: Initial tab display
- **WHEN** the application is launched
- **THEN** a tab bar SHALL be displayed in the titlebar area, to the left of the window control buttons
- **AND** a single tab labeled "Tab 1" SHALL be displayed
- **AND** a new tab (+) button SHALL be displayed to the right of the last tab

#### Scenario: Tab visual styling
- **WHEN** tabs are displayed
- **THEN** each tab SHALL have a subtle 1px outline to distinguish it from the titlebar background
- **AND** the selected tab SHALL have a visually distinct background (matching or lighter than titlebar)
- **AND** unselected tabs SHALL have a slightly darker background than the selected tab
- **AND** the content area SHALL have a subtle outline to distinguish it from the titlebar

### Requirement: Tab Creation
The system SHALL allow users to create new tabs, each hosting a new Neovide instance.

#### Scenario: Create new tab via button
- **WHEN** the user clicks the new tab (+) button
- **THEN** a new Neovide process SHALL be spawned with frameless configuration
- **AND** a new tab SHALL be added to the right of existing tabs
- **AND** the new tab SHALL be labeled with an incrementing number (e.g., "Tab 2", "Tab 3")
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
The system SHALL allow users to close individual tabs via a close button on each tab.

#### Scenario: Close tab with close button
- **WHEN** the user clicks the close (x) button on a tab
- **THEN** the Neovide process associated with that tab SHALL be terminated gracefully
- **AND** the tab SHALL be removed from the tab bar
- **AND** if the closed tab was selected, the next tab (or previous if no next) SHALL become selected
- **AND** the tab bar SHALL be repainted

#### Scenario: Close the last remaining tab
- **WHEN** the user closes the only remaining tab
- **THEN** the Neovide process SHALL be terminated
- **AND** the application window SHALL be closed
- **AND** the application SHALL exit with exit code 0

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

## MODIFIED Requirements

### Requirement: Neovide Process Spawning
The system SHALL spawn Neovide instances with frameless configuration, supporting multiple concurrent instances.

#### Scenario: Successful Neovide launch
- **WHEN** a tab is created (including the initial tab on startup)
- **AND** the client area dimensions are calculated
- **THEN** a Neovide process SHALL be spawned with the following arguments:
  - `--frame none` (removes window decorations)
  - `--size WxH` (where W and H are the content area width and height in pixels)
- **AND** the working directory SHALL be set to the current working directory of the wrapper
- **AND** the process handle SHALL be associated with the corresponding tab

#### Scenario: Neovide spawn failure for new tab
- **WHEN** a new tab's Neovide process fails to spawn
- **THEN** a MessageBox error dialog SHALL be displayed with the spawn error details
- **AND** the tab creation SHALL be cancelled (no new tab added)
- **AND** the previously selected tab SHALL remain selected

### Requirement: Process Lifecycle Management
The system SHALL manage the lifecycle of multiple Neovide processes and ensure proper cleanup.

#### Scenario: Graceful shutdown on wrapper close
- **WHEN** the user closes the wrapper window
- **THEN** all Neovide processes (one per tab) SHALL be terminated gracefully
- **AND** all process handles SHALL be released
- **AND** the wrapper window SHALL be destroyed
- **AND** the application SHALL exit with exit code 0

#### Scenario: Individual Neovide process crash detection
- **WHEN** a Neovide process exits unexpectedly (e.g., crash)
- **THEN** the application SHALL detect the process exit
- **AND** the corresponding tab SHALL be removed from the tab bar
- **AND** if the crashed process was the selected tab, the next available tab SHALL be selected
- **AND** if no tabs remain, a MessageBox error dialog SHALL be displayed
- **AND** the application SHALL exit gracefully
