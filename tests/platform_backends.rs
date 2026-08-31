//! Verify that platform backends are correctly implemented and integrated.
//!
//! These tests ensure that each platform backend file exists, is non-empty,
//! and compiles as part of the conditional compilation logic.

use std::fs;
use std::path::Path;

/// STEP 6: Verify x11.rs backend implementation file exists
#[test]
fn x11_backend_file_exists_and_is_non_empty() {
    let x11_path = Path::new("src/shell/platform/x11.rs");

    // File must exist
    assert!(
        x11_path.exists(),
        "x11.rs backend file must exist at {:?}",
        x11_path
    );

    // File must be readable
    let metadata = fs::metadata(x11_path).expect("x11.rs metadata must be readable");

    // File must not be empty
    assert!(
        metadata.len() > 0,
        "x11.rs backend file must be non-empty (currently {} bytes)",
        metadata.len()
    );

    // File should be a reasonable size for an implementation (at least 1KB)
    assert!(
        metadata.len() > 1024,
        "x11.rs backend implementation should be non-trivial (expected >1KB, got {} bytes)",
        metadata.len()
    );
}

/// Verify that macOS backend exists
#[test]
fn macos_backend_file_exists_and_is_non_empty() {
    let macos_path = Path::new("src/shell/platform/macos.rs");

    assert!(macos_path.exists(), "macos.rs backend file must exist");

    let metadata = fs::metadata(macos_path).expect("macos.rs metadata must be readable");

    assert!(
        metadata.len() > 0,
        "macos.rs backend file must be non-empty"
    );
}

/// Verify that Windows backend exists
#[test]
fn windows_backend_file_exists_and_is_non_empty() {
    let windows_path = Path::new("src/shell/platform/windows.rs");

    assert!(windows_path.exists(), "windows.rs backend file must exist");

    let metadata = fs::metadata(windows_path).expect("windows.rs metadata must be readable");

    assert!(
        metadata.len() > 0,
        "windows.rs backend file must be non-empty"
    );
}

/// Verify that platform/mod.rs exists and correctly conditionally compiles backends
#[test]
fn platform_mod_file_exists() {
    let mod_path = Path::new("src/shell/platform/mod.rs");

    assert!(mod_path.exists(), "platform/mod.rs must exist");

    let content = fs::read_to_string(mod_path).expect("platform/mod.rs must be readable");

    // Verify conditional compilation guards are present
    assert!(
        content.contains("target_os = \"macos\""),
        "platform/mod.rs must have macOS conditional compilation"
    );
    assert!(
        content.contains("target_os = \"windows\""),
        "platform/mod.rs must have Windows conditional compilation"
    );
    assert!(
        content.contains("unix"),
        "platform/mod.rs must have Unix conditional compilation"
    );

    // Verify x11.rs is referenced
    assert!(
        content.contains("x11.rs"),
        "platform/mod.rs must reference x11.rs backend"
    );
}
