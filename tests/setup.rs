//! Verify project setup and hooks are properly configured.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[test]
fn pre_commit_hook_exists_and_is_executable() {
    let hook_path = PathBuf::from(".git/hooks/pre-commit");

    // Check that the hook file exists
    assert!(
        hook_path.exists(),
        "pre-commit hook not found at {}",
        hook_path.display()
    );

    // Check that it's a file
    assert!(
        hook_path.is_file(),
        "pre-commit hook is not a regular file: {}",
        hook_path.display()
    );

    // Check that it's executable
    let metadata = fs::metadata(&hook_path).expect("failed to read hook metadata");
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let is_executable = (mode & 0o111) != 0;

    assert!(
        is_executable,
        "pre-commit hook is not executable (mode: {:o})",
        mode
    );
}

#[test]
fn pre_commit_hook_runs_successfully_when_code_is_clean() {
    use std::process::Command;

    let output = Command::new("bash")
        .arg(".git/hooks/pre-commit")
        .output()
        .expect("failed to execute pre-commit hook");

    assert!(
        output.status.success(),
        "pre-commit hook failed with: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rustc_version_meets_minimum_requirement() {
    use std::process::Command;

    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("failed to run rustc --version");

    assert!(
        output.status.success(),
        "rustc --version failed with: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let version_string = String::from_utf8_lossy(&output.stdout);
    let version =
        parse_rustc_version(&version_string).expect("failed to parse rustc version from output");

    assert!(
        version >= (1, 85),
        "rustc version {} is below minimum required version 1.85",
        format_version(&version)
    );
}

#[test]
fn cargo_version_is_available() {
    use std::process::Command;

    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .expect("failed to run cargo --version");

    assert!(
        output.status.success(),
        "cargo --version failed with: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn parse_rustc_version(version_string: &str) -> Option<(u32, u32)> {
    let version_part = version_string.split_whitespace().nth(1)?;

    let mut parts = version_part.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;

    Some((major, minor))
}

fn format_version(version: &(u32, u32)) -> String {
    format!("{}.{}", version.0, version.1)
}

#[test]
fn wasm_backend_scaffold_removed() {
    use std::fs;
    use std::path::Path;

    let wasm_platform_file = Path::new("src/shell/platform/wasm.rs");
    assert!(
        !wasm_platform_file.exists(),
        "wasm.rs backend file should be removed"
    );

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    assert!(
        !cargo_toml.contains("wasm-bindgen"),
        "Cargo.toml should not contain wasm-bindgen dependency"
    );
    assert!(
        !cargo_toml.contains("web-sys"),
        "Cargo.toml should not contain web-sys dependency"
    );

    let platform_mod =
        fs::read_to_string("src/shell/platform/mod.rs").expect("failed to read platform/mod.rs");
    assert!(
        !platform_mod.contains("target_arch = \"wasm32\""),
        "platform/mod.rs should not have wasm32 conditionals"
    );
}
