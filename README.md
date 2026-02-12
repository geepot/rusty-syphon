# rusty-syphon-spout

Rust bindings for [Syphon](https://syphon.github.io) (macOS) and [Spout](https://spout.zeal.co/) (Windows) — share video frames between applications in real time (e.g. from a capture app to a streaming or VJ app). **Syphon = macOS only; Spout = Windows only.**

## Features

- **macOS (Syphon)** — Server directory, `SyphonOptions`, OpenGL and Metal servers/clients, CGL/GL helpers (see crate docs).
- **Windows (Spout)** — `Spout` type for sending and receiving OpenGL textures, sender list discovery. Uses the [Spout2](https://github.com/leadedge/Spout2) SDK (built from the submodule).

## Requirements

- **macOS**: Xcode (or Command Line Tools). For Syphon from submodule: `xcodebuild -downloadComponent MetalToolchain` if needed.
- **Windows**: CMake and a MSVC or MinGW toolchain. The build compiles the Spout2 submodule and links `SpoutLibrary.dll`.

## Building

Clone with **both** submodules (Syphon for macOS, Spout2 for Windows):

```bash
git clone --recurse-submodules https://github.com/geepot/rusty-syphon-spout
cd rusty-syphon-spout
cargo build
```

- On **macOS**: builds Syphon from the `Syphon-Framework` submodule (or set `SYPHON_FRAMEWORK_PATH`).
- On **Windows**: builds Spout2 from the `Spout2` submodule and the Spout glue; `SpoutLibrary.dll` is copied next to the binary when possible.
- **Prebuilt**: put frameworks/DLLs in `prebuilt/macos/` or `prebuilt/windows/` to skip building from source; see [prebuilt/README.md](prebuilt/README.md).

## Examples

- **List servers** — Print available Syphon servers:

  ```bash
  cargo run --example list_servers
  ```

- **Roundtrip** — Send a test pattern via Syphon (Metal and OpenGL) and verify pixels match:

  ```bash
  cargo run --example roundtrip
  ```

## Testing

- **Host tests (current platform):**

  ```bash
  cargo test --all-targets
  ```

  On Apple Silicon, this requires an arm64-compatible `Syphon.framework` available via
  `prebuilt/macos/`, `SYPHON_FRAMEWORK_PATH`, or a successful local `Syphon-Framework` build.

- **macOS compatibility tests (Intel target):** compile and run Syphon unit tests/examples with staged framework loading.

  ```bash
  cargo test --target x86_64-apple-darwin --all-targets
  ```

- **Windows GNU compatibility tests (compile-only from macOS/Linux):**

  ```bash
  cargo test --target x86_64-pc-windows-gnu --all-targets --no-run
  ```

The cross-platform test suite is in `tests/platform_api.rs`:
- macOS: API-surface checks plus CGL/notification smoke tests.
- Windows: API-surface checks for `Spout` and `SpoutSenderInfo` behavior checks.

Runtime roundtrip integration tests are in `tests/roundtrip_runtime.rs` and are
`#[ignore]` by default because they require live graphics/runtime support.

Run them explicitly:

```bash
# macOS Syphon runtime roundtrip
cargo test --test roundtrip_runtime syphon_opengl_roundtrip_runtime -- --ignored

# Windows Spout runtime roundtrip
cargo test --test roundtrip_runtime spout_image_roundtrip_runtime -- --ignored
```

## CI

GitHub Actions CI runs on every push/PR:

- `macOS (Syphon)`:
  - `cargo check --target x86_64-apple-darwin --all-targets`
  - `cargo test --target x86_64-apple-darwin --all-targets`
- `Windows (Spout)`:
  - `cargo check --all-targets`
  - `cargo test --all-targets`

Workflow file: `.github/workflows/ci.yml`.

## Release Validation

Before publishing a release, run:

```bash
# 1) Linux/macOS host compile sanity
cargo check --all-targets

# 2) macOS Syphon compatibility
cargo test --target x86_64-apple-darwin --all-targets

# 3) Windows GNU compatibility from non-Windows hosts (compile-only)
cargo test --target x86_64-pc-windows-gnu --all-targets --no-run
```

Recommended on a Windows machine/runner as final verification:

```bash
cargo test --all-targets
```

## License

MIT
