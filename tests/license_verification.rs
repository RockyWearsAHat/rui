//! Verification tests for the LICENSE file and Cargo.toml license declaration.
//!
//! These tests ensure that the rui project has a valid MIT License file
//! and that the Cargo.toml properly declares the license metadata.

use std::fs;
use std::path::Path;

#[test]
fn license_file_exists() {
    let license_path = Path::new("LICENSE");
    assert!(
        license_path.exists(),
        "LICENSE file must exist in project root"
    );
}

#[test]
fn license_file_contains_mit_text() {
    let license_text = fs::read_to_string("LICENSE").expect("Failed to read LICENSE file");

    // Check for required MIT License components
    assert!(
        license_text.contains("MIT License"),
        "LICENSE must contain 'MIT License' header"
    );
    assert!(
        license_text.contains("Permission is hereby granted"),
        "LICENSE must contain MIT permission clause"
    );
    assert!(
        license_text.contains("THE SOFTWARE IS PROVIDED \"AS IS\""),
        "LICENSE must contain MIT disclaimer clause"
    );
}

#[test]
fn license_file_has_minimum_length() {
    let license_text = fs::read_to_string("LICENSE").expect("Failed to read LICENSE file");

    // MIT License is typically 1000+ bytes
    assert!(
        license_text.len() >= 1000,
        "LICENSE file must be at least 1000 bytes (got {} bytes)",
        license_text.len()
    );
}

#[test]
fn cargo_toml_declares_license() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml.contains("license = \"MIT\""),
        "Cargo.toml must declare license = \"MIT\""
    );
}

#[test]
fn license_is_mit() {
    let license_text = fs::read_to_string("LICENSE").expect("Failed to read LICENSE file");

    // Check that it's specifically MIT, not some other license
    assert!(
        license_text.contains("Copyright") || license_text.contains("copyright"),
        "LICENSE must contain copyright notice"
    );
    assert!(
        license_text.lines().count() > 20,
        "LICENSE must have substantial content (MIT License has ~28 lines)"
    );
}

#[test]
fn readme_references_license() {
    let readme = fs::read_to_string("README.md").expect("Failed to read README.md");

    assert!(
        readme.contains("LICENSE") || readme.contains("license"),
        "README.md should reference the LICENSE file"
    );
}
