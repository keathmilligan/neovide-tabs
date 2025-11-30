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

### Requirement: Profile Hotkey Configuration

The system SHALL allow users to configure a global hotkey for each profile via a `hotkey` field in the profile definition.

#### Scenario: Profile with hotkey field

- **WHEN** the application starts
- **AND** a profile in the config file has a `hotkey` field with a valid hotkey string
- **THEN** the specified hotkey SHALL be registered for that profile
- **AND** pressing the hotkey SHALL open or activate a tab with that profile

#### Scenario: Profile without hotkey field

- **WHEN** the application starts
- **AND** a profile in the config file does not have a `hotkey` field
- **THEN** no hotkey SHALL be registered for that profile

#### Scenario: Generated Default profile hotkey

- **WHEN** the application starts
- **AND** no "Default" profile is defined in the configuration
- **AND** the system generates a Default profile
- **THEN** the generated Default profile SHALL have `Ctrl+Shift+F1` as its default hotkey

#### Scenario: User-defined Default profile without hotkey

- **WHEN** the application starts
- **AND** a user-defined "Default" profile exists without a `hotkey` field
- **THEN** no hotkey SHALL be registered for the Default profile

#### Scenario: Invalid profile hotkey format

- **WHEN** a profile's `hotkey` field contains an invalid hotkey string
- **THEN** that specific hotkey SHALL be skipped
- **AND** a warning SHALL be logged
- **AND** other valid hotkeys SHALL still be registered

### Requirement: Tab Hotkey Configuration

The system SHALL allow users to configure global hotkeys for activating tabs by index.

#### Scenario: Default tab hotkey configuration

- **WHEN** the application starts
- **AND** no `hotkeys.tab` field exists in the config file
- **THEN** the following default tab hotkeys SHALL be registered:
  - `Ctrl+Shift+1` through `Ctrl+Shift+0` for tabs 1-10 (where 0 = tab 10)

#### Scenario: Custom tab hotkey configuration

- **WHEN** the config file contains a `hotkeys.tab` object
- **AND** the object maps key combinations to tab indices (1-based)
- **THEN** the specified hotkeys SHALL be registered for tab activation
- **AND** default tab hotkeys SHALL NOT be registered

#### Scenario: Empty tab hotkey configuration

- **WHEN** the config file contains an empty `hotkeys.tab` object (`{}`)
- **THEN** no tab hotkeys SHALL be registered

#### Scenario: Invalid tab hotkey format

- **WHEN** a tab hotkey string in the config file has an invalid format
- **THEN** that specific hotkey SHALL be skipped
- **AND** a warning SHALL be logged
- **AND** other valid hotkeys SHALL still be registered

### Requirement: Hotkey String Format

The system SHALL parse hotkey strings in a human-readable format combining modifiers and keys.

#### Scenario: Valid hotkey string formats

- **WHEN** parsing hotkey configuration
- **THEN** the following formats SHALL be accepted:
  - `Ctrl+Shift+1` (modifier order: Ctrl, Alt, Shift, Win)
  - `Ctrl+Shift+F1` (function keys F1-F12)
  - `Alt+Shift+A` (alphabetic keys A-Z)
  - `Ctrl+Alt+Shift+0` (multiple modifiers)
- **AND** modifier names SHALL be case-insensitive
- **AND** at least one modifier key SHALL be required

#### Scenario: Key name aliases

- **WHEN** parsing key names in hotkey strings
- **THEN** the following aliases SHALL be accepted:
  - `Ctrl` or `Control` for the Control key
  - `Win` or `Windows` or `Super` for the Windows key
  - `0` through `9` for numeric keys
  - `F1` through `F12` for function keys
  - `A` through `Z` for alphabetic keys

