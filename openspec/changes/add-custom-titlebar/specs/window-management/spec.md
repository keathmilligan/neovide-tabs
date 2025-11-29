## MODIFIED Requirements

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

## ADDED Requirements

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
