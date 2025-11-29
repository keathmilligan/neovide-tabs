## ADDED Requirements

### Requirement: Application Window Creation
The system SHALL create a native Windows application window on startup.

#### Scenario: Successful window creation
- **WHEN** the application is launched
- **THEN** a window titled "neovide-tabs" SHALL be created with standard decorations (title bar, close button, resize borders)
- **AND** the window SHALL have a default size of 1024x768 pixels
- **AND** the window SHALL be centered on the primary monitor

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
The system SHALL spawn a single Neovide instance with frameless configuration.

#### Scenario: Successful Neovide launch
- **WHEN** the wrapper window is created
- **AND** the client area dimensions are calculated
- **THEN** a Neovide process SHALL be spawned with the following arguments:
  - `--frame none` (removes window decorations)
  - `--size WxH` (where W and H are the client area width and height in pixels)
- **AND** the working directory SHALL be set to the current working directory of the wrapper
- **AND** the process handle SHALL be retained for lifecycle management

#### Scenario: Neovide spawn failure
- **WHEN** the Neovide process fails to spawn (e.g., invalid path, permission error)
- **THEN** a MessageBox error dialog SHALL be displayed with the spawn error details
- **AND** the wrapper window SHALL remain open (allowing the user to close it)

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
The system SHALL manage the Neovide process lifecycle and ensure proper cleanup.

#### Scenario: Graceful shutdown on wrapper close
- **WHEN** the user closes the wrapper window
- **THEN** the Neovide process SHALL be terminated gracefully
- **AND** the process handle SHALL be released
- **AND** the wrapper window SHALL be destroyed
- **AND** the application SHALL exit with exit code 0

#### Scenario: Neovide process crash detection
- **WHEN** the Neovide process exits unexpectedly (e.g., crash)
- **THEN** the application SHALL detect the process exit
- **AND** a MessageBox error dialog SHALL be displayed with the message "Neovide process has exited unexpectedly."
- **AND** the wrapper window SHALL remain open (allowing the user to close it)

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
