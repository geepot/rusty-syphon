//! Raw FFI bindings: Syphon (macOS) and Spout (Windows).

#![allow(non_camel_case_types)]

#[cfg(target_os = "macos")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(target_os = "windows")]
include!(concat!(env!("OUT_DIR"), "/spout_bindings.rs"));
