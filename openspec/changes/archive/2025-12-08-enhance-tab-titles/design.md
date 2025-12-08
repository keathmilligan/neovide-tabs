## Context

Tabs currently display static profile names. This change adds dynamic tab titles that can include contextual information like the working directory or the Neovide window title (which reflects the current buffer/file being edited).

The Neovide window title is set by Neovim and typically includes the current filename, so `%t` provides users with visibility into what each tab is editing.

## Goals / Non-Goals

- **Goals**:
  - Allow users to customize tab titles with dynamic content
  - Provide sensible defaults that show useful information out of the box
  - Keep tab titles in sync with Neovide's window title as the user navigates files
  
- **Non-Goals**:
  - Complex templating or scripting (keep to simple `%X` token expansion)
  - Real-time title updates (periodic polling is sufficient)
  - Custom formatting beyond the supported tokens

## Decisions

### Token Syntax: `%X` style
- **Decision**: Use `%p`, `%w`, `%t` format (similar to terminal title escape sequences)
- **Rationale**: Simple, familiar pattern used in many terminal emulators and shell prompts

### Working Directory Format: `~/xxx`
- **Decision**: Convert home directory prefix to `~` for display
- **Rationale**: More compact and user-friendly than full paths

### Title Stripping Characters
- **Decision**: Strip leading/trailing space, tab, and dash (`-`) characters
- **Rationale**: Neovide titles often include leading/trailing dashes or spaces as separators; stripping them produces cleaner tab titles

### Default Title: `%t`
- **Decision**: Default to showing the Neovide window title
- **Rationale**: Provides the most useful information (current file/buffer) without requiring configuration

### Title Refresh Strategy
- **Decision**: Query title on tab open, tab activation, and via periodic timer
- **Rationale**: 
  - On open/activation ensures immediate accuracy
  - Periodic refresh (every 1-2 seconds) keeps title in sync while user edits
  - Reuses existing process polling timer to minimize overhead

## Risks / Trade-offs

- **Performance**: Periodic title queries add minimal overhead (single Win32 call per visible tab)
- **Title Staleness**: With periodic polling, titles may lag behind actual state by up to the poll interval; this is acceptable for tab titles

## Open Questions

None - the requirements are clear and the implementation is straightforward.
