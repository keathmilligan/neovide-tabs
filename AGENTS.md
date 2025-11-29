<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Agent Instructions for neovide-tabs

## Build, Test, and Lint Commands
- Build: `cargo build` (dev) or `cargo build --release` (optimized)
- Run: `cargo run` or `cargo run --release`
- Test all: `cargo test`
- Test single: `cargo test test_name` or `cargo test module_name::test_name`
- Lint: `cargo clippy -- -D warnings` (fail on warnings)
- Format check: `cargo fmt -- --check`
- Format code: `cargo fmt`

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules. Use `use` statements, avoid wildcards unless common (prelude).
- **Formatting**: Follow rustfmt defaults (4 spaces, 100 char lines). Run `cargo fmt` before committing.
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Types**: Prefer explicit types for public APIs. Use type inference internally. Avoid `unwrap()` in production code.
- **Error Handling**: Use `Result<T, E>` and `?` operator. Create custom error types for public APIs. Use `anyhow` for applications, `thiserror` for libraries.
- **Comments**: Use `///` for public API docs, `//` for inline comments. Document all public items with examples where appropriate.
- **Safety**: Avoid `unsafe` unless absolutely necessary. Document safety invariants thoroughly.

## Platform Considerations
- This project targets Windows, Linux, and macOS. Test platform-specific code with `#[cfg(target_os = "...")]`.
- Use `std::path::Path` and `PathBuf` for cross-platform path handling, never hardcode path separators.
