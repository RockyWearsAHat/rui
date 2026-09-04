//! Verify project setup and hooks are properly configured.

// Pre-commit hook tests are skipped in automated test suites
// because they require complex git setup and environment state.
// The hook is manually verified: run `bash .git/hooks/pre-commit`
// to test formatting and linting checks.

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
    let version_part = version_string
        .split_whitespace()
        .nth(1)
        .expect("rustc version output missing version");
    let mut parts = version_part.split('.');
    let major: u32 = parts
        .next()
        .expect("rustc version missing major")
        .parse()
        .expect("invalid major version");
    let minor: u32 = parts
        .next()
        .expect("rustc version missing minor")
        .parse()
        .expect("invalid minor version");

    assert!(
        (major, minor) >= (1, 85),
        "rustc version {}.{} is below minimum required version 1.85",
        major,
        minor
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

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm_builds_declare_the_cdylib_crate_type() {
    use std::fs;

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("crate-type = [\"cdylib\", \"rlib\"]"),
        "wasm builds require cdylib crate-type to generate JavaScript bindings"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm_pack_release_profile_disables_wasm_opt() {
    use std::fs;

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("[package.metadata.wasm-pack.profile.release]"),
        "wasm-pack release profile section must be configured in Cargo.toml"
    );
    assert!(
        cargo_toml.contains("wasm-opt = false"),
        "wasm-pack must disable wasm-opt optimization for bulk memory compatibility"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn window_backend_selector_gates_wasm_correctly() {
    use std::fs;

    let platform_mod =
        fs::read_to_string("src/shell/platform/mod.rs").expect("failed to read platform/mod.rs");

    // Verify wasm cfg is present and ordered correctly
    let wasm_index = platform_mod
        .find("target_arch = \"wasm32\"")
        .expect("wasm32 cfg must be defined");
    let unsupported_index = platform_mod
        .find("not(any(")
        .expect("unsupported fallback must be defined");

    assert!(
        wasm_index < unsupported_index,
        "wasm backend must be checked before unsupported fallback to ensure wasm32 targets compile"
    );

    // Verify the path assignment for wasm
    assert!(
        platform_mod.contains("#[path = \"wasm.rs\"]"),
        "wasm.rs must be selected when target_arch = wasm32"
    );

    // Verify other platforms select their correct backends
    assert!(
        platform_mod.contains("target_os = \"macos\""),
        "macos backend must be defined"
    );
    assert!(
        platform_mod.contains("target_os = \"windows\""),
        "windows backend must be defined"
    );
    assert!(
        platform_mod
            .contains("all(unix, not(target_os = \"macos\"), not(target_arch = \"wasm32\"))"),
        "x11 backend must be selected for unix platforms excluding macos and wasm"
    );

    // Verify all backends are assigned to the mod named 'backend'
    let backend_assignments = platform_mod
        .matches("mod backend;")
        .collect::<Vec<_>>()
        .len();
    assert_eq!(
        backend_assignments, 6,
        "exactly 6 backend cfgs should each define 'mod backend;' (wasm, macos, windows, wayland, x11, unsupported)"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wayland_backend_is_compile_selectable_via_feature_flag() {
    use std::fs;

    let platform_mod =
        fs::read_to_string("src/shell/platform/mod.rs").expect("failed to read platform/mod.rs");

    // Verify wayland backend exists and is conditionally compiled
    assert!(
        platform_mod.contains("path = \"wayland.rs\""),
        "wayland.rs must be selected when feature is enabled"
    );

    // Verify the wayland feature guard is present
    assert!(
        platform_mod.contains("feature = \"wayland\""),
        "wayland backend must be gated with feature = \"wayland\""
    );

    // Verify x11 backend is the fallback (negation of wayland feature)
    assert!(
        platform_mod.contains("not(feature = \"wayland\")"),
        "x11 backend must be selected when wayland feature is not enabled"
    );

    // Verify the feature is documented in Cargo.toml
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("[features]"),
        "Cargo.toml must have a [features] section"
    );
    assert!(
        cargo_toml.contains("wayland"),
        "wayland feature must be declared in Cargo.toml"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm_module_is_exported_from_lib_with_correct_guard() {
    use std::fs;

    let lib_rs = fs::read_to_string("src/lib.rs").expect("failed to read src/lib.rs");

    // Find the line with `pub mod wasm;`
    let wasm_pub_line = lib_rs
        .lines()
        .position(|line| line.contains("pub mod wasm;"))
        .expect("wasm module must be exported as 'pub mod wasm;'");

    // Check that the preceding lines contain the cfg guard
    let context_start = wasm_pub_line.saturating_sub(2);
    let context_end = wasm_pub_line + 1;
    let context = lib_rs
        .lines()
        .skip(context_start)
        .take(context_end - context_start)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        context.contains("target_arch = \"wasm32\""),
        "wasm module must be guarded with #[cfg(target_arch = \"wasm32\")] before 'pub mod wasm;'\nContext:\n{}",
        context
    );

    // Verify the guard is actually on the line immediately before pub mod wasm
    let guard_line = lib_rs
        .lines()
        .nth(wasm_pub_line - 1)
        .expect("there should be a line before pub mod wasm;");

    assert!(
        guard_line.contains("#[cfg") || (guard_line.is_empty() && wasm_pub_line >= 2),
        "the line immediately before 'pub mod wasm;' should be the cfg attribute"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm32_target_compiles_counter_example() {
    use std::process::Command;

    let output = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("-p")
        .arg("rui-native")
        .arg("--example")
        .arg("counter")
        .output()
        .expect("failed to run cargo build for wasm32 target");

    assert!(
        output.status.success(),
        "wasm32-unknown-unknown target failed to compile counter example:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm_pack_generates_web_bindings() {
    use std::process::Command;

    let output = Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--release")
        .arg("--out-dir")
        .arg("pkg")
        .output()
        .expect("failed to run wasm-pack build");

    assert!(
        output.status.success(),
        "wasm-pack build failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify generated artifacts exist
    let pkg_exists = std::path::Path::new("pkg/rui_native_bg.wasm").exists();
    assert!(
        pkg_exists,
        "wasm-pack should generate pkg/rui_native_bg.wasm"
    );

    let js_exists = std::path::Path::new("pkg/rui_native.js").exists();
    assert!(js_exists, "wasm-pack should generate pkg/rui_native.js");
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wasm_builds_for_target() {
    use std::process::Command;

    // Just verify WASM target compiles, don't run tests that require browser drivers
    let output = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("-p")
        .arg("rui-native")
        .output()
        .expect("failed to run cargo build for wasm32 target");

    assert!(
        output.status.success(),
        "WASM build failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn wayland_feature_flag_compiles_successfully() {
    use std::process::Command;

    // Verify that cargo build --features wayland compiles the wayland backend
    let output = Command::new("cargo")
        .arg("build")
        .arg("--features")
        .arg("wayland")
        .arg("-p")
        .arg("rui-native")
        .output()
        .expect("failed to run cargo build --features wayland");

    assert!(
        output.status.success(),
        "wayland feature build failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
