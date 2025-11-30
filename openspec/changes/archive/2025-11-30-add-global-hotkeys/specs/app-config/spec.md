## ADDED Requirements

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
