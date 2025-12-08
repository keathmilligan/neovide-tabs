## 1. Dependencies
- [x] 1.1 Add `resvg` crate to Cargo.toml dependencies

## 2. Implementation
- [x] 2.1 Add `load_svg_as_bitmap` function in `src/icons.rs` to load and rasterize SVG files
- [x] 2.2 Modify `load_icon` function to detect `.svg` extension and route to SVG loader
- [x] 2.3 Handle SVG parsing errors gracefully (return None to trigger fallback)

## 3. Testing
- [x] 3.1 Add unit test to verify SVG loading with a sample SVG file
- [x] 3.2 Add unit test for invalid SVG handling (malformed content)
- [x] 3.3 Manual test: configure a profile with an SVG icon and verify it displays correctly

## 4. Validation
- [x] 4.1 Run `cargo clippy -- -D warnings` to check for lint issues
- [x] 4.2 Run `cargo test` to verify all tests pass
- [x] 4.3 Run `cargo build --release` to verify release build succeeds
