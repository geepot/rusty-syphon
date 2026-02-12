# rusty-syphon

Rust bindings for the [Syphon](https://syphon.github.io) framework on **macOS**. Syphon lets applications share video and still images in real time (e.g. from a capture app to a streaming or VJ app).

## Features

- **Server directory** — Discover available Syphon servers.
- **OpenGL** — `OpenGLServer`, `OpenGLClient`, `OpenGLImage` (CGL context + GL textures, including helpers for headless context and rectangle textures).
- **Metal** — `MetalServer`, `MetalClient`, `MetalTexture` (use with the `metal` crate).

## Requirements

- macOS
- Xcode (or Xcode Command Line Tools) for building the Syphon framework and C/ObjC glue
- For building Syphon from the submodule: run `xcodebuild -downloadComponent MetalToolchain` if Metal-related build errors occur

## Building

Clone with the Syphon submodule:

```bash
git clone --recurse-submodules https://github.com/geepot/rusty-syphon
cd rusty-syphon
cargo build
```

The build will compile the Syphon framework from the `Syphon-Framework` submodule (or use an existing framework via `SYPHON_FRAMEWORK_PATH`).

## Examples

- **List servers** — Print available Syphon servers:

  ```bash
  cargo run --example list_servers
  ```

- **Roundtrip** — Send a test pattern via Syphon (Metal and OpenGL) and verify pixels match:

  ```bash
  cargo run --example roundtrip
  ```

## License

MIT OR Apache-2.0
