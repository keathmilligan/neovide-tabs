# Design: Add Tab Profiles

## Context

The neovide-tabs application currently opens all new tabs with the wrapper's current working directory. Users who work across multiple projects need a way to quickly open tabs configured for specific directories without manual navigation. The Windows Terminal provides a similar feature with profiles that can be selected from a dropdown.

### Stakeholders
- End users who work with multiple project directories
- Developers maintaining the codebase

### Constraints
- Must work within the existing Win32 window framework
- Icon rendering must be efficient (no per-frame image loading)
- Must gracefully handle missing/invalid icon files
- Dropdown UI must feel native to Windows

## Goals / Non-Goals

### Goals
- Allow users to define named profiles with custom working directories
- Provide visual distinction between tabs via profile icons and names
- Make profile selection discoverable via dropdown UI
- Maintain backward compatibility (existing configs without profiles work as before)

### Non-Goals
- Profile-specific Neovim configurations (only working directory is profile-specific)
- Profile import/export functionality
- Keyboard shortcuts for specific profiles (can be added later)
- Profile editing UI (profiles are config-file managed)

## Decisions

### Decision 1: Profile Storage Location
**Choice**: Store profiles in the existing `config.json` file under a `profiles` array.

**Rationale**: 
- Keeps all configuration in one place
- Follows the existing configuration pattern
- No additional file discovery needed

**Alternatives considered**:
- Separate `profiles.json` file - rejected as unnecessary complexity
- Per-profile files - rejected as overengineering for the use case

### Decision 2: Default Profile Generation
**Choice**: Generate a "Default" profile at runtime if no profiles exist or if no profile is explicitly named "Default".

**Rationale**:
- Ensures there's always a fallback profile for the + button
- Users don't need to configure anything to get started
- Consistent behavior whether config file exists or not

**Alternatives considered**:
- Require users to define at least one profile - rejected as poor UX
- Create config file on first run - rejected to maintain current silent-failure pattern

### Decision 3: Icon Handling
**Choice**: Use a default icon (`neovide.png`) from a known location when no icon is specified. Icons are specified by filename and loaded from a `icons/` subdirectory next to the config file.

**Rationale**:
- Simple path resolution
- Bundled default icon ensures consistent appearance
- Relative paths prevent config file portability issues

**Icon location**: `~/.config/neovide-tabs/icons/`

**Alternatives considered**:
- Absolute paths only - rejected as less user-friendly
- Embed icons in executable - rejected as inflexible
- System icon registry - rejected as platform-specific complexity

### Decision 4: Tab Display Format
**Choice**: Display icon (16x16 pixels) followed by profile name in each tab, replacing the current "Tab N" label.

**Rationale**:
- Provides immediate visual identification
- Consistent with Windows Terminal behavior
- Icon size fits comfortably in the 24px tab content height

**Alternatives considered**:
- Icon only - rejected as insufficient identification
- Name only - rejected as less visually scannable
- Tooltip on hover - decided to also include this for full profile details

### Decision 5: Dropdown UI Pattern
**Choice**: Render a small dropdown arrow button (16px wide) immediately to the right of the + button. Clicking it shows a native-style dropdown menu with profile icons and names.

**Rationale**:
- Follows established Windows Terminal pattern
- Minimal UI footprint
- Discoverable but not intrusive

**Alternatives considered**:
- Right-click context menu on + button - rejected as less discoverable
- Separate menu bar - rejected as inconsistent with current minimal UI
- Popup panel - rejected as heavier weight than needed

### Decision 6: Working Directory for Default Profile
**Choice**: The generated Default profile uses the user's home directory as working directory.

**Rationale**:
- Predictable behavior regardless of how the app was launched
- Consistent with typical shell default behavior
- Easy to understand for users

**Alternatives considered**:
- Use current working directory - rejected as unpredictable (depends on how app was launched)
- No working directory (inherit from wrapper) - rejected as same issue

## Data Model

### Profile Structure
```json
{
  "profiles": [
    {
      "name": "Default",
      "icon": "neovide.png",
      "working_directory": "~"
    },
    {
      "name": "Work Project",
      "icon": "work.png",
      "working_directory": "~/projects/work"
    }
  ]
}
```

### Rust Structures
```rust
#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub icon: PathBuf,        // Resolved full path
    pub working_directory: PathBuf,  // Resolved full path
}
```

## Risks / Trade-offs

### Risk: Icon Loading Performance
**Risk**: Loading icons on every paint could cause UI lag.
**Mitigation**: Cache loaded icon bitmaps in memory. Load icons once during config parsing.

### Risk: Missing Icon Files
**Risk**: User specifies icon that doesn't exist.
**Mitigation**: Fall back to default icon and log a warning (no error dialog).

### Risk: Invalid Working Directory
**Risk**: User specifies directory that doesn't exist.
**Mitigation**: Fall back to home directory with warning log. Neovide will still launch.

### Trade-off: Dropdown vs Context Menu
Dropdown takes horizontal space but is more discoverable. Accepted trade-off for better UX.

## Open Questions

None - all questions resolved during design.
