//! Rust bindings for the [Syphon](https://syphon.github.io) framework on macOS.
//!
//! Syphon allows applications to share video and still images in real time.
//! This crate exposes:
//! - **Server directory**: discover available Syphon servers.
//! - **OpenGL**: `OpenGLServer`, `OpenGLClient`, `OpenGLImage` (CGL context + GL textures).
//! - **Metal**: `MetalServer`, `MetalClient`, `MetalTexture` (MTLDevice/MTLTexture pointers).

mod ffi;
mod safe;

pub use safe::*;
