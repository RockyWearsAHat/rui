//! Verify project setup and hooks are properly configured.

use std::fs;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use std::os::unix::fs::PermissionsExt;

#[test]
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
fn the_wasm_backend_is_compiled_in_and_costs_the_native_builds_nothing() {
    use std::fs;
    use std::path::Path;

    assert!(
        Path::new("src/shell/platform/wasm.rs").exists(),
        "the browser backend belongs beside the native ones"
    );

    let platform_mod =
        fs::read_to_string("src/shell/platform/mod.rs").expect("failed to read platform/mod.rs");
    assert!(
        platform_mod.contains("target_arch = \"wasm32\""),
        "and must be chosen by a cfg, or it is never built"
    );

    // The crate's headline claim is no dependencies. `wasm-bindgen` is the one
    // exception and it is scoped to `wasm32`, so this asserts the scoping and
    // not merely the absence: a plain `[dependencies]` entry would pass a
    // "contains wasm-bindgen" check while quietly costing every native build.
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let plain: String = cargo_toml
        .split("[dependencies]")
        .nth(1)
        .expect("a [dependencies] section")
        .split("\n[")
        .next()
        .expect("its end")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();
    assert!(
        !plain.contains("wasm-bindgen") && !plain.contains("web-sys"),
        "the browser's crates must stay under [target.'cfg(target_arch = \"wasm32\")'.dependencies]"
    );
}
