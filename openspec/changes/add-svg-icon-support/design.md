## Context

The current icon loading system in `src/icons.rs` uses the `image` crate to load raster images (PNG). The `image` crate does not support SVG format because SVG is a vector format requiring rasterization before it can be used as a bitmap.

Users want SVG support to leverage vector icons that scale cleanly on high-DPI displays.

## Goals / Non-Goals

**Goals:**
- Support SVG files as profile icons alongside existing PNG support
- Rasterize SVG to the correct icon size (16x16 for tabs) at load time
- Integrate seamlessly with the existing icon caching system

**Non-Goals:**
- Supporting animated SVG
- Runtime scaling of icons (icons are pre-rendered to fixed size)
- Supporting SVG for the application window icon (remains PNG only)

## Decisions

**Decision: Use `resvg` crate for SVG rendering**

`resvg` is the de-facto standard for SVG rendering in Rust. It provides:
- High-quality rendering with anti-aliasing
- Good SVG compliance for typical icon files
- Direct output to a pixel buffer compatible with our bitmap creation

Alternatives considered:
- `usvg` + custom rendering: More complex, requires manual rasterization
- `cairo-rs`: Heavy dependency, platform-specific build complexity
- `librsvg` bindings: Requires GTK runtime, not suitable for Windows-first app

**Decision: Detect format by file extension**

The icon path is checked for `.svg` extension to determine whether to use SVG or raster loading. This is simple and matches user expectations (they name their files appropriately).

Alternative considered:
- Magic byte detection: More robust but unnecessary complexity for this use case

**Decision: Rasterize at ICON_SIZE (16x16)**

SVG files are rasterized to the standard icon size at load time. The resulting bitmap is cached like any other icon.

## Risks / Trade-offs

- **Binary size increase**: `resvg` adds approximately 1-2MB to the binary. Acceptable for the functionality gained.
- **Build time increase**: `resvg` compilation adds some build time. Mitigated by only adding it as a direct dependency.

## Open Questions

None - this is a straightforward addition with well-defined scope.
