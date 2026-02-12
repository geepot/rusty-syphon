//! Raw FFI bindings to the Syphon C glue. macOS only.

#![allow(non_camel_case_types)]

#[cfg(target_os = "macos")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
