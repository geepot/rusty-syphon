//! Rust bindings for [Syphon](https://syphon.github.io) (macOS) and [Spout](https://spout.zeal.co/) (Windows).
//!
//! Share video frames between applications: **Syphon on macOS**, **Spout on Windows**.
//!
//! - **macOS**: Server directory, `SyphonOptions`, OpenGL and Metal servers/clients, CGL/GL helpers.
//! - **Windows**: `Spout` type for sender and receiver (OpenGL textures), sender list discovery.

mod ffi;
mod safe;

pub use safe::*;
