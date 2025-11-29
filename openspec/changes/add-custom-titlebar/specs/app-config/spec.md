## ADDED Requirements

### Requirement: Title Bar Color Configuration
The system SHALL use the configured background color for the custom title bar.

#### Scenario: Title bar uses background color
- **WHEN** the window is displayed
- **AND** a `background_color` is configured
- **THEN** the custom title bar background SHALL be filled with the configured background color

#### Scenario: Title bar uses default color
- **WHEN** the window is displayed
- **AND** no `background_color` is configured
- **THEN** the custom title bar background SHALL be filled with the default color `#1a1b26`

### Requirement: Title Text Color
The system SHALL render title bar text in a contrasting color.

#### Scenario: Title text visibility
- **WHEN** the title bar is rendered
- **THEN** the window title text SHALL be rendered in white (#FFFFFF) for visibility against dark backgrounds
