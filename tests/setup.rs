//! Verify project setup and hooks are properly configured.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use std::os::unix::fs::PermissionsExt;

// Serialize git-state-modifying tests to prevent parallel execution conflicts
static GIT_LOCK: Mutex<()> = Mutex::new(());

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

    let _guard = GIT_LOCK.lock().unwrap();

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
fn pre_commit_hook_rejects_unformatted_code() {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    let _guard = GIT_LOCK.lock().unwrap();

    // Create a temporary directory for the test repo to avoid polluting the main repo
    let temp_dir = std::env::temp_dir().join(format!("rui_hook_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("failed to create temp directory");

    // Copy the pre-commit hook to the temp dir
    let git_dir = temp_dir.join(".git/hooks");
    fs::create_dir_all(&git_dir).expect("failed to create .git/hooks");
    let hook_src = PathBuf::from(".git/hooks/pre-commit");
    let hook_dst = git_dir.join("pre-commit");
    fs::copy(&hook_src, &hook_dst).expect("failed to copy pre-commit hook");

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&hook_dst, perms).expect("failed to set hook permissions");
    }

    // Initialize a git repo in the temp directory
    let init_output = Command::new("git")
        .current_dir(&temp_dir)
        .arg("init")
        .output()
        .expect("failed to run git init");
    assert!(
        init_output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    // Configure git user for commits
    let _ = Command::new("git")
        .current_dir(&temp_dir)
        .arg("config")
        .arg("user.email")
        .arg("test@example.com")
        .output();
    let _ = Command::new("git")
        .current_dir(&temp_dir)
        .arg("config")
        .arg("user.name")
        .arg("Test User")
        .output();

    // 1. Write unformatted Rust code
    let test_file = temp_dir.join("bad_code.rs");
    let bad_code = "fn     test_func(  ) {\n    let  x=1;\n}\n";
    fs::write(&test_file, bad_code).expect("failed to write test file");

    // 2. Stage the file
    let stage_output = Command::new("git")
        .current_dir(&temp_dir)
        .arg("add")
        .arg("bad_code.rs")
        .output()
        .expect("failed to run git add");
    assert!(
        stage_output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&stage_output.stderr)
    );

    // 3. Attempt commit (should fail)
    let commit_output = Command::new("git")
        .current_dir(&temp_dir)
        .arg("commit")
        .arg("-m")
        .arg("test: verify hook rejects unformatted code")
        .output()
        .expect("failed to run git commit");

    let stderr = String::from_utf8_lossy(&commit_output.stderr);

    // 4. Assert commit failed
    assert!(
        !commit_output.status.success(),
        "pre-commit hook should have rejected unformatted code, but commit succeeded. stderr: {}",
        stderr
    );

    // 5. Assert error message indicates formatting issue
    assert!(
        stderr.contains("Diff")
            || stderr.contains("code is not formatted")
            || stderr.contains("error")
            || stderr.contains("failed"),
        "hook stderr should indicate formatting issue: {}",
        stderr
    );

    // 6. Cleanup: remove temp directory
    let _ = fs::remove_dir_all(&temp_dir);
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
        platform_mod.contains("#[path = \"wasm.rs\"]\nmod backend;")
            || platform_mod.contains("#[path = \"wasm.rs\"]\n#["),
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

    // Phase 1 & 2: Wayland backend with feature flag
    assert!(
        platform_mod.contains("target_os = \"linux\"")
            && platform_mod.contains("feature = \"wayland\""),
        "wayland backend must be conditionally selected with feature = \"wayland\""
    );

    // X11 backend (fallback for non-wayland Linux)
    assert!(
        platform_mod.contains("unix") && platform_mod.contains("not(feature = \"wayland\")"),
        "x11 backend must be fallback for unix platforms when wayland feature is not enabled"
    );

    // Verify all backends are assigned to the mod named 'backend'
    let backend_assignments = platform_mod
        .matches("mod backend;")
        .collect::<Vec<_>>()
        .len();
    assert_eq!(
        backend_assignments, 8,
        "exactly 8 backend cfgs should each define 'mod backend;' (wasm, ios, android, macos, windows, wayland, x11, unsupported)"
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
fn wasm_tests_pass_on_target() {
    use std::process::Command;

    let output = Command::new("cargo")
        .arg("test")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()
        .expect("failed to run cargo test for wasm32 target");

    assert!(
        output.status.success(),
        "WASM tests failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
