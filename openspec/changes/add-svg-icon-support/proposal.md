# Change: Add SVG Support for Tab Icons

## Why

Users want to use SVG files as tab icons for profiles. SVG is a popular vector format that scales cleanly at any resolution, making it ideal for icons that need to look crisp on high-DPI displays. Currently, the application only supports raster formats (PNG) for profile icons.

## What Changes

- Add SVG file format support for profile icons
- Add `resvg` crate dependency for SVG rasterization
- Extend icon loading logic to detect and handle `.svg` files
- Rasterize SVG files to the target icon size (16x16 for tabs) at load time

## Impact

- Affected specs: `app-config` (Profile Icon Loading requirement)
- Affected code: `src/icons.rs` (icon loading functions), `Cargo.toml` (new dependency)
- New dependency: `resvg` crate for SVG rendering
