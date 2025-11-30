## ADDED Requirements

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
The generated default configuration file SHALL document all available options with their default values as comments.

#### Scenario: Background color default
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain a commented `background_color` field
- **AND** the value SHALL be `"#1a1b26"` (the current default)

#### Scenario: Hotkeys defaults
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain a commented `hotkeys` object
- **AND** the `hotkeys.tab` object SHALL document the default tab hotkeys (`Ctrl+Shift+1` through `Ctrl+Shift+0` for tabs 1-10)

#### Scenario: Profile examples
- **WHEN** the default config file is generated
- **THEN** the file SHALL contain a commented `profiles` array
- **AND** the array SHALL include example profiles demonstrating:
  - A profile with custom `name`, `icon`, `working_directory`, and `hotkey` fields
  - A profile with minimal configuration (name only)
- **AND** each example profile SHALL include inline comments explaining the purpose of each field

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
