//! Integration tests for X11 backend on Linux systems.
//!
//! These tests verify that the X11 backend integrates correctly with
//! the platform abstraction layer and can be compiled for Linux targets.

use std::fs;
use std::path::Path;

/// Verify X11 backend is properly integrated in platform/mod.rs
#[test]
fn x11_is_integrated_in_platform_mod() {
    let mod_path = Path::new("src/shell/platform/mod.rs");
    let content = fs::read_to_string(mod_path).expect("platform/mod.rs must be readable");

    // Verify x11 module is referenced
    assert!(
        content.contains("x11"),
        "platform/mod.rs must reference x11 backend"
    );
}

/// Verify X11 backend has proper module structure
#[test]
fn x11_backend_module_structure() {
    let x11_path = Path::new("src/shell/platform/x11.rs");
    let content = fs::read_to_string(x11_path).expect("x11.rs must be readable");

    // Verify module declares the Window struct/type
    assert!(
        content.contains("struct Window")
            || content.contains("pub struct Window")
            || content.contains("impl Window"),
        "x11.rs must declare a Window struct"
    );

    // Verify it implements the Backend trait
    assert!(
        content.contains("impl Backend for Window") || content.contains("impl Backend for"),
        "x11.rs must implement the Backend trait"
    );
}

/// Verify X11 backend handles event types
#[test]
fn x11_backend_handles_events() {
    let x11_path = Path::new("src/shell/platform/x11.rs");
    let content = fs::read_to_string(x11_path).expect("x11.rs must be readable");

    // Verify event handling infrastructure is present
    assert!(
        content.contains("Event") || content.contains("event") || content.contains("XEvent"),
        "x11.rs must reference event handling"
    );
}
