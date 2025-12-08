## ADDED Requirements

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
