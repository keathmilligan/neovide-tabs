# app-config Specification

## Purpose
TBD - created by archiving change add-config-capability. Update Purpose after archive.
## Requirements
### Requirement: Configuration File Loading
The system SHALL load configuration from a JSON file at application startup.

#### Scenario: Config file exists with valid JSON
- **WHEN** the application starts
- **AND** a file exists at `~/.config/neovide-tabs/config.json`
- **AND** the file contains valid JSON
- **THEN** the configuration values SHALL be parsed and applied

#### Scenario: Config file does not exist
- **WHEN** the application starts
- **AND** no file exists at `~/.config/neovide-tabs/config.json`
- **THEN** default configuration values SHALL be used
- **AND** no error SHALL be displayed to the user

#### Scenario: Config file contains invalid JSON
- **WHEN** the application starts
- **AND** the config file exists but contains invalid JSON
- **THEN** default configuration values SHALL be used
- **AND** no error SHALL be displayed to the user

### Requirement: Background Color Configuration
The system SHALL allow users to configure the window background color via the configuration file.

#### Scenario: Valid hex color specified
- **WHEN** the config file contains a `background_color` field
- **AND** the value is a valid 6-character hex color (with or without `#` prefix)
- **THEN** the window background SHALL be set to the specified color

#### Scenario: Default background color
- **WHEN** no `background_color` is specified in the config
- **OR** the config file does not exist
- **THEN** the window background SHALL default to `#1a1b26`

#### Scenario: Invalid background color format
- **WHEN** the config file contains an invalid `background_color` value
- **THEN** the default background color `#1a1b26` SHALL be used

### Requirement: Resize Background Fill
The system SHALL fill the window background with the configured color during resize operations to prevent visual flashing.

#### Scenario: Window resize
- **WHEN** the user resizes the wrapper window
- **THEN** any exposed window area SHALL be filled with the configured background color
- **AND** no white or system-default color flash SHALL be visible

#### Scenario: Window maximise/restore
- **WHEN** the user maximises or restores the wrapper window
- **THEN** any exposed window area SHALL be filled with the configured background color
- **AND** no white or system-default color flash SHALL be visible

