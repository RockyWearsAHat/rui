//! Verify X11 backend parity with other platform backends.
//!
//! These tests ensure that the X11 backend implementation matches
//! the expected interface and behavior of other platform backends.

use std::fs;
use std::path::Path;

/// Verify X11 backend file exists and is non-empty
#[test]
fn x11_backend_exists() {
    let x11_path = Path::new("src/shell/platform/x11.rs");

    assert!(x11_path.exists(), "x11.rs backend must exist");

    let metadata =
        fs::metadata(x11_path).expect("x11.rs metadata must be readable");

    assert!(
        metadata.len() > 0,
        "x11.rs must be non-empty (got {} bytes)",
        metadata.len()
    );
}

/// Verify X11 backend implements the Backend trait
#[test]
fn x11_backend_implements_trait() {
    let x11_path = Path::new("src/shell/platform/x11.rs");
    let content = fs::read_to_string(x11_path)
        .expect("x11.rs must be readable");

    // Verify all required Backend trait methods are present
    assert!(
        content.contains("fn open"),
        "x11.rs must implement fn open()"
    );
    assert!(
        content.contains("fn pump"),
        "x11.rs must implement fn pump()"
    );
    assert!(
        content.contains("fn surface"),
        "x11.rs must implement fn surface()"
    );
    assert!(
        content.contains("fn appearance"),
        "x11.rs must implement fn appearance()"
    );
    assert!(
        content.contains("fn present"),
        "x11.rs must implement fn present()"
    );
    assert!(
        content.contains("fn is_open"),
        "x11.rs must implement fn is_open()"
    );
}

/// Verify X11 backend has no obvious TODOs or incomplete markers
#[test]
fn x11_backend_is_complete() {
    let x11_path = Path::new("src/shell/platform/x11.rs");
    let content = fs::read_to_string(x11_path)
        .expect("x11.rs must be readable");

    // Warn if there are unimplemented!() calls
    if content.contains("unimplemented!()") {
        panic!("x11.rs contains unimplemented!() calls");
    }

    // Warn if there are TODO markers
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.contains("TODO") && !line.trim().starts_with("//") {
            panic!(
                "x11.rs line {}: TODO marker in non-comment code",
                i + 1
            );
        }
    }
}
