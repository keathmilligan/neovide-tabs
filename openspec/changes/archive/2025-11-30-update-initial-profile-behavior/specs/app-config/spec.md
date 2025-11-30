## MODIFIED Requirements

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

## ADDED Requirements

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
