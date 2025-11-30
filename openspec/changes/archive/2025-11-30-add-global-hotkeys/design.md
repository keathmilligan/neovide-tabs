## Context

neovide-tabs manages multiple Neovide instances in a tabbed interface. Currently, tab switching requires clicking in the tab bar when the wrapper window is focused. Users want system-wide hotkeys to activate tabs or open profiles without switching focus first. This is a cross-cutting change affecting configuration parsing and window message handling.

## Goals / Non-Goals

**Goals:**
- Register global hotkeys on application startup using Win32 `RegisterHotKey` API
- Handle `WM_HOTKEY` messages in the window procedure
- Support configurable hotkey bindings in the JSON config file
- Provide sensible defaults (Ctrl+Shift+1-0 for tabs, Ctrl+Shift+F1 for Default profile)
- Only register profile hotkeys for profiles that actually exist
- Gracefully handle hotkey registration conflicts (another app has the hotkey)
- Clean up hotkeys on application exit using `UnregisterHotKey`

**Non-Goals:**
- Cross-platform hotkey support (Windows-only, matching project constraints)
- Runtime hotkey rebinding (requires app restart)
- Visual indicator for registered hotkeys
- Hotkey sequence support (e.g., leader keys)

## Decisions

### Decision: Use Win32 RegisterHotKey directly instead of external crate

**Rationale:** The `windows` crate already provides all necessary bindings for `RegisterHotKey`, `UnregisterHotKey`, and `WM_HOTKEY` message handling. Adding an external crate (e.g., `win-hotkeys`, `global-hotkey`) would introduce unnecessary dependencies when the existing Win32 bindings suffice. The implementation is straightforward and keeps the dependency footprint minimal.

**Alternatives considered:**
- `win-hotkeys` crate - Adds thread-based event loop, unnecessary complexity for our message-pump architecture
- `global-hotkey` crate - Cross-platform focused, heavier than needed for Windows-only app

### Decision: Hotkey IDs use i32 range with distinct ranges for tab vs profile hotkeys

**Rationale:** `RegisterHotKey` requires unique integer IDs. Using range 1-10 for tab hotkeys and 101+ for profile hotkeys provides clear separation and allows easy identification in `WM_HOTKEY` handler. Profile hotkey IDs are 101 + profile_index.

### Decision: Profile hotkeys defined inline with profile configuration

**Rationale:** Defining the hotkey as a field within each profile keeps related configuration together. This is more intuitive than a separate mapping section and makes it clear which hotkey belongs to which profile.

**Config structure:**
```json
{
  "profiles": [
    {
      "name": "Default",
      "hotkey": "Ctrl+Shift+F1"
    },
    {
      "name": "Work",
      "icon": "work.png",
      "working_directory": "~/projects",
      "hotkey": "Ctrl+Shift+F2"
    }
  ],
  "hotkeys": {
    "tab": {
      "Ctrl+Shift+1": 1,
      "Ctrl+Shift+2": 2
    }
  }
}
```

### Decision: Only register hotkeys for profiles that exist

**Rationale:** Rather than registering 12 profile hotkeys by default (most of which would do nothing), we only register hotkeys for profiles that are actually configured. This:
- Avoids "wasting" system-wide hotkey registrations
- Reduces conflicts with other applications
- Makes the behavior predictable: if a profile exists and has a hotkey, it works

The generated "Default" profile gets `Ctrl+Shift+F1` by default.

### Decision: Profile hotkeys open OR activate existing tab

**Rationale:** If a profile hotkey (e.g., Ctrl+Shift+F1 for "Default") is pressed and a tab with that profile is already open, we should activate that tab rather than create a duplicate. This matches the mental model of "go to this profile" rather than "create new instance of this profile." If users want multiple tabs of the same profile, they can use the + button or dropdown.

**Note:** Implementation will search tabs by profile index to find matching tabs.

### Decision: Graceful handling of hotkey conflicts

**Rationale:** Other applications may have registered the same hotkeys. If `RegisterHotKey` fails, log a warning but continue with other hotkeys. Do not fail application startup due to hotkey conflicts.

## Risks / Trade-offs

- **Risk:** Hotkey conflicts with other applications
  - **Mitigation:** Log warnings, continue without conflicting hotkey, document defaults so users can reconfigure
  
- **Risk:** User confusion about which hotkeys are active
  - **Mitigation:** Document defaults clearly, consider future enhancement to show registered hotkeys in title bar menu

- **Trade-off:** Inline profile hotkeys vs separate hotkey section
  - **Decision:** Inline keeps related config together; separate tab hotkey section kept since tabs are positional not named

## Migration Plan

No migration needed - this is a new feature. Existing config files without `hotkey` fields on profiles will use default hotkey for the Default profile only.

## Open Questions

1. Should profile hotkeys activate the *first* tab with that profile, or the *most recently used* tab?
   - **Proposed:** First tab with that profile (simpler implementation, predictable behavior)

2. Should there be a hotkey to create a *new* tab with a profile even if one exists?
   - **Proposed:** No, keep it simple. Use + button for new tabs. Hotkeys are for quick access.
