## MODIFIED Requirements

### Requirement: Profile Icon Loading
The system SHALL load profile icons using the following resolution: the default icon is loaded from the data directory, and user-defined icons are loaded from full paths. Both PNG and SVG formats are supported for user-defined icons.

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

#### Scenario: SVG icon loading
- **WHEN** a profile specifies an icon path ending in `.svg`
- **AND** the file exists and contains valid SVG content
- **THEN** the SVG SHALL be rasterized to the icon size (16x16 pixels)
- **AND** the rasterized bitmap SHALL be cached for rendering

#### Scenario: Invalid SVG file
- **WHEN** a profile specifies an icon path ending in `.svg`
- **AND** the file exists but contains invalid SVG content
- **THEN** the default fallback icon (green square) SHALL be used
- **AND** no error SHALL be displayed to the user
