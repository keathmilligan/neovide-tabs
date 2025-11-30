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

### Requirement: Profile Configuration
The system SHALL allow users to define tab profiles in the configuration file, where each profile specifies a name, optional icon (as a full path), and optional working directory.

#### Scenario: Config file contains valid profiles array
- **WHEN** the application starts
- **AND** the config file contains a `profiles` array with one or more profile objects
- **AND** each profile object contains at least a `name` field
- **THEN** the profiles SHALL be parsed and made available for tab creation
- **AND** profiles missing an `icon` field SHALL default to `neovide-tabs.png`
- **AND** profiles missing a `working_directory` field SHALL default to the user's home directory

#### Scenario: Config file contains empty profiles array
- **WHEN** the application starts
- **AND** the config file contains an empty `profiles` array
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide-tabs.png", and working directory set to the user's home directory

#### Scenario: Config file has no profiles field
- **WHEN** the application starts
- **AND** the config file exists but has no `profiles` field
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide-tabs.png", and working directory set to the user's home directory

#### Scenario: Profile with invalid working directory
- **WHEN** a profile specifies a `working_directory` that does not exist
- **THEN** the profile SHALL fall back to using the user's home directory
- **AND** no error SHALL be displayed to the user

### Requirement: Default Profile Generation
The system SHALL ensure a profile named "Default" always exists by generating one at runtime if not defined in the configuration.

#### Scenario: No Default profile in configuration
- **WHEN** the application starts
- **AND** no profile with the name "Default" exists in the configuration
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide-tabs.png", and working directory set to the user's home directory
- **AND** the generated Default profile SHALL be inserted at the beginning of the profiles list

#### Scenario: User-defined Default profile exists
- **WHEN** the application starts
- **AND** a profile named "Default" is defined in the configuration
- **THEN** the user-defined Default profile SHALL be used without modification

### Requirement: Profile Icon Loading
The system SHALL load profile icons using the following resolution: the default icon is loaded from the data directory, and user-defined icons are loaded from full paths.

#### Scenario: Default icon loading
- **WHEN** a profile uses the default icon (`neovide-tabs.png`)
- **THEN** the icon SHALL be loaded from `~/.local/share/neovide-tabs/neovide-tabs.png`
- **AND** the icon SHALL be cached for rendering

#### Scenario: User-defined icon with valid path
- **WHEN** a profile specifies a custom icon path
- **AND** the path is a full/absolute path to an existing file
- **THEN** the icon SHALL be loaded from that path and cached for rendering

#### Scenario: User-defined icon with invalid path
- **WHEN** a profile specifies a custom icon path
- **AND** the file does not exist at that path
- **THEN** the default fallback icon (green square) SHALL be used
- **AND** no error SHALL be displayed to the user

#### Scenario: Default icon for profile without icon field
- **WHEN** a profile does not specify an `icon` field
- **THEN** the icon SHALL default to `neovide-tabs.png` (loaded from data directory)

### Requirement: Bundled Default Icon
The system SHALL embed the default tab icon (`neovide-tabs.png`) into the executable at compile time and extract it to the data directory at runtime.

#### Scenario: First application launch
- **WHEN** the application starts for the first time
- **AND** the data directory (`~/.local/share/neovide-tabs/`) does not exist
- **THEN** the system SHALL create the data directory
- **AND** the system SHALL extract the bundled `neovide-tabs.png` icon to that directory

#### Scenario: Subsequent application launches
- **WHEN** the application starts
- **AND** `~/.local/share/neovide-tabs/neovide-tabs.png` already exists
- **THEN** the system SHALL NOT overwrite the existing icon file

#### Scenario: Data directory exists but icon missing
- **WHEN** the application starts
- **AND** the data directory exists but the icon file is missing
- **THEN** the system SHALL extract the bundled icon to the data directory

### Requirement: Application Window Icon
The system SHALL display `neovide-tabs.png` as the application window icon in the taskbar, title bar, and Alt-Tab switcher.

#### Scenario: Window icon display
- **WHEN** the application window is created
- **THEN** the window SHALL display the `neovide-tabs.png` icon in the title bar
- **AND** the window SHALL display the icon in the Windows taskbar
- **AND** the window SHALL display the icon in the Alt-Tab task switcher

