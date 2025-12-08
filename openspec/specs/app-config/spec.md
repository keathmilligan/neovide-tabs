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
The system SHALL generate an internal "Default" profile only when no profiles are defined in the configuration.

#### Scenario: No profiles field in configuration
- **WHEN** the application starts
- **AND** the config file exists but has no `profiles` field
- **THEN** an internal Default profile SHALL be generated with name "Default", icon "neovide.png", and working directory set to the user's home directory
- **AND** the generated Default profile SHALL have `Ctrl+Shift+F1` as its default hotkey

#### Scenario: Empty profiles array in configuration
- **WHEN** the application starts
- **AND** the config file contains an empty `profiles` array
- **THEN** an internal Default profile SHALL be generated with name "Default", icon "neovide.png", and working directory set to the user's home directory

#### Scenario: User-defined profiles exist
- **WHEN** the application starts
- **AND** the config file contains a non-empty `profiles` array
- **THEN** no internal Default profile SHALL be generated
- **AND** the profiles list SHALL contain only the user-defined profiles

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

### Requirement: Default Config File Generation
The system SHALL generate a default configuration file with documented defaults when no config file exists.

#### Scenario: Config file does not exist on startup
- **WHEN** the application starts
- **AND** no file exists at `~/.config/neovide-tabs/config.jsonc` or `~/.config/neovide-tabs/config.json`
- **THEN** the system SHALL create the config directory if it does not exist
- **AND** the system SHALL generate a `config.jsonc` file containing all configuration options
- **AND** all option values SHALL be commented out using `//` comment syntax
- **AND** the file SHALL contain the current default values for each option

#### Scenario: Config file already exists
- **WHEN** the application starts
- **AND** a file already exists at `~/.config/neovide-tabs/config.jsonc` or `~/.config/neovide-tabs/config.json`
- **THEN** the system SHALL NOT overwrite or modify the existing file
- **AND** the system SHALL load configuration from the existing file as normal

#### Scenario: Config directory does not exist
- **WHEN** the application starts
- **AND** the directory `~/.config/neovide-tabs/` does not exist
- **THEN** the system SHALL create the directory with standard permissions
- **AND** the system SHALL then generate the default config file

### Requirement: Default Config File Content
The generated default configuration file SHALL document all available options with their default values, and SHALL include a working "Neovim" profile that is not commented out.

#### Scenario: Background color default
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain a commented `background_color` field
- **AND** the value SHALL be `"#1a1b26"` (the current default)

#### Scenario: Hotkeys defaults
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain a commented `hotkeys` object
- **AND** the `hotkeys.tab` object SHALL document the default tab hotkeys (`Ctrl+Shift+1` through `Ctrl+Shift+0` for tabs 1-10)

#### Scenario: Default Neovim profile
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain an uncommented `profiles` array
- **AND** the first profile SHALL have name "Neovim"
- **AND** the first profile SHALL use the default icon
- **AND** the first profile SHALL use the user's home directory as working directory
- **AND** the profile SHALL include `Ctrl+Shift+F1` as its default hotkey

#### Scenario: Profile examples in comments
- **WHEN** the default config file is generated
- **THEN** the file SHALL include commented example profiles demonstrating:
  - A profile with custom `name`, `icon`, `working_directory`, and `hotkey` fields
  - A profile with minimal configuration (name only)
- **AND** each example SHALL include inline comments explaining the purpose of each field

### Requirement: JSONC File Support
The system SHALL support both `.jsonc` and `.json` file extensions for configuration, with `.jsonc` as the preferred format.

#### Scenario: Config file discovery priority
- **WHEN** the application starts
- **AND** both `config.jsonc` and `config.json` exist
- **THEN** the system SHALL load configuration from `config.jsonc`

#### Scenario: Fallback to .json
- **WHEN** the application starts
- **AND** `config.jsonc` does not exist
- **AND** `config.json` exists
- **THEN** the system SHALL load configuration from `config.json`

### Requirement: JSONC Comment Parsing
The system SHALL parse JSONC format (JSON with Comments) for both `.jsonc` and `.json` configuration files.

#### Scenario: Comment stripping
- **WHEN** loading a configuration file
- **THEN** the system SHALL strip `//` line comments before parsing
- **AND** comments inside JSON strings SHALL be preserved as literal text
- **AND** the remaining content SHALL be parsed as standard JSON

#### Scenario: Comment format
- **WHEN** the default config file is generated
- **THEN** comments SHALL use `//` line comment syntax
- **AND** commented-out JSON values SHALL be valid JSON if the `//` prefix is removed
- **AND** the file SHALL parse correctly with JSONC-aware parsers

#### Scenario: Structural validity
- **WHEN** all comment lines are removed from the generated file
- **THEN** the remaining content SHALL be valid JSON
- **AND** uncommenting any individual option SHALL result in valid JSON

### Requirement: Initial Tab Profile Selection
The system SHALL use the first profile in the configuration for the initial tab when the application starts.

#### Scenario: Profiles defined in config
- **WHEN** the application starts
- **AND** the config file contains one or more profiles
- **THEN** the initial tab SHALL be created using the first profile in the list

#### Scenario: No profiles in config (fallback)
- **WHEN** the application starts
- **AND** no profiles are defined in the configuration
- **THEN** the initial tab SHALL be created using the internally generated Default profile

### Requirement: Profile Title Configuration

The system SHALL allow users to configure a dynamic tab title for each profile via a `title` field in the profile definition, supporting string expansion tokens.

#### Scenario: Profile with custom title format

- **WHEN** the application starts
- **AND** a profile in the config file has a `title` field with a format string
- **THEN** tabs created with that profile SHALL display the expanded title
- **AND** the title SHALL be expanded using the supported tokens

#### Scenario: Profile without title field

- **WHEN** the application starts
- **AND** a profile in the config file does not have a `title` field
- **THEN** the profile SHALL default to using `%t` (Neovide window title) as the title format

#### Scenario: Generated Default profile title

- **WHEN** the application starts
- **AND** no profiles are defined in the configuration
- **AND** the system generates a Default profile
- **THEN** the generated Default profile SHALL have `%t` as its title format

### Requirement: Title Expansion Tokens

The system SHALL support the following tokens in profile title format strings:

- `%p` - Profile name
- `%w` - Working directory (displayed in `~/xxx` form for paths under the user's home directory)
- `%t` - Neovide window title (as reported by the Neovide window)

#### Scenario: Profile name token expansion

- **WHEN** a tab title format contains `%p`
- **THEN** `%p` SHALL be replaced with the profile's name

#### Scenario: Working directory token expansion

- **WHEN** a tab title format contains `%w`
- **AND** the working directory is under the user's home directory
- **THEN** `%w` SHALL be replaced with the path using `~` as the home directory prefix (e.g., `~/projects/foo`)

#### Scenario: Working directory not under home

- **WHEN** a tab title format contains `%w`
- **AND** the working directory is NOT under the user's home directory
- **THEN** `%w` SHALL be replaced with the full absolute path

#### Scenario: Neovide window title token expansion

- **WHEN** a tab title format contains `%t`
- **AND** the Neovide window is ready
- **THEN** `%t` SHALL be replaced with the current Neovide window title

#### Scenario: Neovide window not ready

- **WHEN** a tab title format contains `%t`
- **AND** the Neovide window is not yet ready (window handle not discovered)
- **THEN** `%t` SHALL be replaced with an empty string

#### Scenario: Combined token expansion

- **WHEN** a tab title format contains multiple tokens (e.g., `%p: %w`)
- **THEN** all tokens SHALL be expanded in place
- **AND** literal text between tokens SHALL be preserved

### Requirement: Title String Sanitization

The system SHALL strip leading and trailing whitespace, tab, and dash (`-`) characters from the final expanded title.

#### Scenario: Strip leading characters

- **WHEN** the expanded title has leading space, tab, or dash characters
- **THEN** those characters SHALL be stripped from the beginning

#### Scenario: Strip trailing characters

- **WHEN** the expanded title has trailing space, tab, or dash characters
- **THEN** those characters SHALL be stripped from the end

#### Scenario: Preserve internal characters

- **WHEN** the expanded title has space, tab, or dash characters in the middle
- **THEN** those characters SHALL be preserved

### Requirement: Title Refresh Timing

The system SHALL query and update the Neovide window title at specific times to keep tab titles synchronized.

#### Scenario: Title update on tab creation

- **WHEN** a new tab is created
- **THEN** the tab title SHALL be computed using the current Neovide window title (if available)

#### Scenario: Title update on tab activation

- **WHEN** a tab is activated (switched to)
- **THEN** the tab title SHALL be refreshed using the current Neovide window title

#### Scenario: Periodic title refresh

- **WHEN** the application is running with one or more tabs
- **THEN** the system SHALL periodically query the Neovide window title for the active tab
- **AND** the tab title SHALL be updated if it has changed
- **AND** the tab bar SHALL be repainted if the title changed

