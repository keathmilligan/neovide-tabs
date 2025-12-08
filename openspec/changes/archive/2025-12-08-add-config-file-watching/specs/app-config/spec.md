## ADDED Requirements

### Requirement: Configuration File Watching

The system SHALL monitor the configuration file for changes and reload configuration dynamically when modifications are detected.

#### Scenario: Config file modified while application running

- **WHEN** the application is running
- **AND** the user modifies and saves the config file (`config.jsonc` or `config.json`)
- **THEN** the system SHALL detect the file change within 1 second
- **AND** the system SHALL reload and parse the configuration
- **AND** the system SHALL apply applicable changes to the running application

#### Scenario: Config file created after startup

- **WHEN** the application is running with default configuration
- **AND** the user creates a new config file
- **THEN** the system SHALL detect the new file
- **AND** the system SHALL load and apply the new configuration

#### Scenario: Config file deleted while application running

- **WHEN** the application is running with a loaded configuration
- **AND** the user deletes the config file
- **THEN** the system SHALL continue running with the current configuration
- **AND** no error SHALL be displayed to the user

### Requirement: Configuration Reload Debouncing

The system SHALL debounce configuration file change events to avoid excessive reloading during editing.

#### Scenario: Rapid file saves

- **WHEN** the config file is modified multiple times in quick succession (within 500ms)
- **THEN** the system SHALL wait until modifications stop for at least 250ms before reloading
- **AND** only one reload operation SHALL occur for the batch of changes

#### Scenario: Single file save

- **WHEN** the config file is modified once
- **THEN** the system SHALL reload the configuration after the debounce delay (250ms)

### Requirement: Invalid Configuration Handling on Reload

The system SHALL gracefully handle invalid configuration during hot-reload without disrupting the running application.

#### Scenario: Config file contains invalid JSON during reload

- **WHEN** a config file change is detected
- **AND** the modified file contains invalid JSON
- **THEN** the current configuration SHALL be preserved
- **AND** a warning SHALL be logged
- **AND** no error dialog SHALL be displayed to the user
- **AND** the system SHALL continue watching for further changes

#### Scenario: Config file temporarily unreadable

- **WHEN** a config file change is detected
- **AND** the file cannot be read (locked by editor, permission error)
- **THEN** the system SHALL retry reading after a short delay (100ms)
- **AND** if still unreadable, the current configuration SHALL be preserved
- **AND** a warning SHALL be logged

### Requirement: Background Color Hot-Reload

The system SHALL apply background color changes immediately when the configuration is reloaded.

#### Scenario: Background color changed in config

- **WHEN** the configuration is reloaded
- **AND** the `background_color` value has changed
- **THEN** the title bar background SHALL be updated to the new color
- **AND** the window SHALL be repainted to reflect the change
- **AND** subsequent resize operations SHALL use the new background color

### Requirement: Profile List Hot-Reload

The system SHALL update the available profiles when the configuration is reloaded.

#### Scenario: New profile added to config

- **WHEN** the configuration is reloaded
- **AND** a new profile has been added to the `profiles` array
- **THEN** the new profile SHALL be available in the profile dropdown menu
- **AND** if the new profile has a hotkey, the hotkey SHALL be registered

#### Scenario: Profile removed from config

- **WHEN** the configuration is reloaded
- **AND** a profile has been removed from the `profiles` array
- **THEN** the profile SHALL no longer appear in the profile dropdown menu
- **AND** existing tabs created with that profile SHALL continue to function
- **AND** existing tabs SHALL retain their original profile settings (icon, title format, working directory)

#### Scenario: Profile modified in config

- **WHEN** the configuration is reloaded
- **AND** an existing profile's properties have changed (icon, title, hotkey, working directory)
- **THEN** the profile dropdown SHALL show the updated profile information
- **AND** existing tabs using that profile SHALL update their icon on the next title refresh
- **AND** existing tabs using that profile SHALL update their title format on the next title refresh

### Requirement: Hotkey Hot-Reload

The system SHALL re-register global hotkeys when the configuration is reloaded and hotkey settings have changed.

#### Scenario: Tab hotkey configuration changed

- **WHEN** the configuration is reloaded
- **AND** the `hotkeys.tab` configuration has changed
- **THEN** all existing tab hotkeys SHALL be unregistered
- **AND** new tab hotkeys SHALL be registered according to the updated configuration

#### Scenario: Profile hotkey changed

- **WHEN** the configuration is reloaded
- **AND** a profile's hotkey has been added, removed, or changed
- **THEN** the old hotkey (if any) SHALL be unregistered
- **AND** the new hotkey (if any) SHALL be registered

#### Scenario: Hotkey conflicts after reload

- **WHEN** the configuration is reloaded
- **AND** a hotkey in the new configuration conflicts with another application
- **THEN** a warning SHALL be logged for the conflicting hotkey
- **AND** other non-conflicting hotkeys SHALL still be registered
- **AND** the application SHALL continue running normally

### Requirement: Tab Display Update on Config Reload

The system SHALL update the display of currently-open tabs when relevant profile configuration changes.

#### Scenario: Profile icon path changed

- **WHEN** the configuration is reloaded
- **AND** a profile's icon path has changed
- **AND** tabs exist using that profile
- **THEN** those tabs SHALL display the new icon on the next periodic title refresh
- **AND** the tab bar SHALL be repainted to show the updated icons

#### Scenario: Profile title format changed

- **WHEN** the configuration is reloaded
- **AND** a profile's title format has changed
- **AND** tabs exist using that profile
- **THEN** those tabs SHALL use the new title format on the next periodic title refresh
- **AND** the tab bar SHALL be repainted if the displayed title changes
