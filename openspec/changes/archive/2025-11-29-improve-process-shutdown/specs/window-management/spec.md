## MODIFIED Requirements

### Requirement: Process Lifecycle Management
The system SHALL manage the lifecycle of multiple Neovide processes, detect process exits, and ensure proper cleanup.

#### Scenario: Graceful shutdown on wrapper close
- **WHEN** the user closes the wrapper window
- **THEN** all Neovide processes spawned by neovide-tabs SHALL be terminated
- **AND** Neovide processes that were launched externally (not by neovide-tabs) SHALL NOT be affected
- **AND** all process handles SHALL be released
- **AND** the wrapper window SHALL be destroyed
- **AND** the application SHALL exit with exit code 0

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
The system SHALL allow users to close individual tabs via a close button on each tab, terminating the associated process.

#### Scenario: Close tab with close button
- **WHEN** the user clicks the close (x) button on a tab
- **THEN** the Neovide process associated with that tab SHALL be terminated immediately
- **AND** the tab SHALL be removed from the tab bar
- **AND** if the closed tab was selected, the next tab (or previous if no next) SHALL become selected
- **AND** the tab bar SHALL be repainted

#### Scenario: Close the last remaining tab
- **WHEN** the user closes the only remaining tab via the close button
- **THEN** the Neovide process SHALL be terminated
- **AND** the application window SHALL be closed
- **AND** the application SHALL exit with exit code 0

## ADDED Requirements

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
