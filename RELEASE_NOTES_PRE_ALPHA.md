# RevEngine 0.2.0-pre-alpha.1

**Status:** pre-alpha — experimental build for early testing. Not a stable release.

## What is included

- Native Rust desktop editor built with egui/eframe.
- 3D scene editing viewport and scene/project serialization.
- Procedural scene generation and AI-provider configuration.
- Blender bridge and project scaffolding.
- Import/export bridges for Godot, Roblox, Unity and Unreal project formats.
- Dependency doctor and asset optimization helpers.

## Validation performed

- `cargo check --all-targets` — passed.
- `cargo test --all-targets` — 19 passed, 0 failed.
- `cargo clippy --all-targets` — completed with non-fatal style warnings; see known limitations.
- `cargo build --release` — passed on Windows x64.
- `cargo fmt --all` — applied.

## Known limitations

- This is a pre-alpha build. Expect incomplete workflows, rough UI edges, and possible crashes or data loss; back up projects before testing.
- Clippy reports remaining style warnings (mostly nested-if simplifications and minor idioms). They do not currently prevent compilation.
- Blender workflows require a compatible Blender installation and a configured executable path.
- AI features require a valid API key and a compatible provider endpoint/model; no API credentials are bundled.
- External engine import/export compatibility depends on the target engine version and project structure. Validate generated projects in the target application.
- Automated tests cover core parsing, path safety, project scaffolding, procedural scenes and integration bridges; they do not replace interactive GUI, GPU/driver, Blender, or end-to-end testing.

## Windows quick start

1. Download and extract the Windows x64 package.
2. Run `RevEngine.exe`.
3. Create a new project and save it in a writable folder.
4. Configure Blender and AI provider credentials in Settings only if those integrations are needed.
5. Keep backups of all projects while testing this pre-alpha.

## Feedback

When reporting a bug, include Windows version, GPU/driver, steps to reproduce, expected vs. actual behavior, and logs if available. Do not include API keys or other secrets.
