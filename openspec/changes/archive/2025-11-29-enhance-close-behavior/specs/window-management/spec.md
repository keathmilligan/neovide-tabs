## MODIFIED Requirements

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

## ADDED Requirements

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
