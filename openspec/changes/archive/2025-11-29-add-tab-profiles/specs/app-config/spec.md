## ADDED Requirements

### Requirement: Profile Configuration
The system SHALL allow users to define tab profiles in the configuration file, where each profile specifies a name, optional icon, and optional working directory.

#### Scenario: Config file contains valid profiles array
- **WHEN** the application starts
- **AND** the config file contains a `profiles` array with one or more profile objects
- **AND** each profile object contains at least a `name` field
- **THEN** the profiles SHALL be parsed and made available for tab creation
- **AND** profiles missing an `icon` field SHALL default to `neovide.png`
- **AND** profiles missing a `working_directory` field SHALL default to the user's home directory

#### Scenario: Config file contains empty profiles array
- **WHEN** the application starts
- **AND** the config file contains an empty `profiles` array
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide.png", and working directory set to the user's home directory

#### Scenario: Config file has no profiles field
- **WHEN** the application starts
- **AND** the config file exists but has no `profiles` field
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide.png", and working directory set to the user's home directory

#### Scenario: Profile with invalid working directory
- **WHEN** a profile specifies a `working_directory` that does not exist
- **THEN** the profile SHALL fall back to using the user's home directory
- **AND** no error SHALL be displayed to the user

### Requirement: Default Profile Generation
The system SHALL ensure a profile named "Default" always exists by generating one at runtime if not defined in the configuration.

#### Scenario: No Default profile in configuration
- **WHEN** the application starts
- **AND** no profile with the name "Default" exists in the configuration
- **THEN** a Default profile SHALL be generated with name "Default", icon "neovide.png", and working directory set to the user's home directory
- **AND** the generated Default profile SHALL be inserted at the beginning of the profiles list

#### Scenario: User-defined Default profile exists
- **WHEN** the application starts
- **AND** a profile named "Default" is defined in the configuration
- **THEN** the user-defined Default profile SHALL be used without modification

### Requirement: Profile Icon Loading
The system SHALL load profile icons from the icons directory relative to the configuration file location.

#### Scenario: Icon file exists
- **WHEN** a profile specifies an icon filename
- **AND** the file exists at `~/.config/neovide-tabs/icons/<filename>`
- **THEN** the icon SHALL be loaded and cached for rendering

#### Scenario: Icon file does not exist
- **WHEN** a profile specifies an icon filename
- **AND** the file does not exist at the expected location
- **THEN** a default fallback icon SHALL be used
- **AND** no error SHALL be displayed to the user

#### Scenario: Default icon for profile without icon field
- **WHEN** a profile does not specify an `icon` field
- **THEN** the icon SHALL default to `neovide.png`
