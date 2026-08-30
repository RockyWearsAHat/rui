//! Verify that Recipe 1 documentation is accurate against the actual codebase.
//!
//! This test ensures that:
//! - All commits mentioned in CLAUDE.md Recipe 1 exist (phase 1: clock abstraction, phase 2: frame driver, phase 3: WASM)
//! - All files mentioned exist and are accessible
//! - Key architectural elements exist: Backend trait (6 methods), turn() function, continues() helper, Page struct, run() implementations, clock abstraction (Moment), WASM exports and backend implementation
//! - Line number references in CLAUDE.md match current code locations

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn recipe_1_all_commits_exist() {
    // Phase 1 commits
    let phase_1_commits = vec!["77d4780"];

    // Phase 2 commits
    let phase_2_commits = vec!["531214f", "9afc9b1", "b6a1b2c", "2ef3c2b", "caa3066"];

    // Phase 3 commits
    let phase_3_commits = vec![
        "b116ac8", "32bf53d", "d820ff6", "e41376e", "929899a", "830033c", "2365866", "3062aba",
        "401a8a7", "ce4acad", "2df7f1c",
    ];

    let all_commits = [phase_1_commits, phase_2_commits, phase_3_commits].concat();

    for commit in all_commits {
        // Use rev-parse to check if the commit exists
        let output = Command::new("git")
            .arg("rev-parse")
            .arg(commit)
            .output()
            .expect("Failed to run git rev-parse");

        assert!(
            output.status.success(),
            "Commit {} mentioned in Recipe 1 documentation does not exist in git history",
            commit
        );
    }
}

#[test]
fn recipe_1_all_files_exist() {
    let files = vec![
        "src/shell/clock.rs",
        "src/shell/mod.rs",
        "src/app.rs",
        "src/wasm.rs",
        "src/shell/platform/wasm.rs",
        "tests/external_driving.rs",
        "Cargo.toml",
    ];

    for file in files {
        assert!(
            Path::new(file).exists(),
            "File {} mentioned in Recipe 1 documentation does not exist",
            file
        );
    }
}

#[test]
fn recipe_1_backend_trait_exists() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    assert!(
        content.contains("trait Backend"),
        "Backend trait not found in src/shell/mod.rs"
    );

    // Verify it has the six expected methods
    let trait_section = content
        .split("trait Backend")
        .next()
        .unwrap_or("")
        .to_string()
        + content
            .split("trait Backend")
            .nth(1)
            .unwrap_or("")
            .split("}\n")
            .next()
            .unwrap_or("");

    assert!(
        trait_section.contains("fn open"),
        "Backend::open() not found"
    );
    assert!(
        trait_section.contains("fn pump"),
        "Backend::pump() not found"
    );
    assert!(
        trait_section.contains("fn surface"),
        "Backend::surface() not found"
    );
    assert!(
        trait_section.contains("fn appearance"),
        "Backend::appearance() not found"
    );
    assert!(
        trait_section.contains("fn present"),
        "Backend::present() not found"
    );
    assert!(
        trait_section.contains("fn is_open"),
        "Backend::is_open() not found"
    );
}

#[test]
fn recipe_1_turn_function_exists() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    assert!(
        content.contains("fn turn<S>"),
        "turn() function not found in src/shell/mod.rs"
    );
}

#[test]
fn recipe_1_clock_abstraction_exists() {
    let content =
        fs::read_to_string("src/shell/clock.rs").expect("Failed to read src/shell/clock.rs");

    assert!(
        content.contains("struct Moment"),
        "Moment type not found in src/shell/clock.rs"
    );

    assert!(
        content.contains("fn now()"),
        "Moment::now() not found in src/shell/clock.rs"
    );
}

#[test]
fn recipe_1_external_driving_test_exists() {
    let content = fs::read_to_string("tests/external_driving.rs")
        .expect("Failed to read tests/external_driving.rs");

    assert!(
        content.contains("state_mut_between_frames_drives_the_next_frame"),
        "state_mut_between_frames_drives_the_next_frame test not found"
    );
}

#[test]
fn recipe_1_wasm_exports_exist() {
    let content = fs::read_to_string("src/wasm.rs").expect("Failed to read src/wasm.rs");

    assert!(
        content.contains("init_counter"),
        "init_counter() not found in src/wasm.rs"
    );
    assert!(
        content.contains("listen_counter"),
        "listen_counter() not found in src/wasm.rs"
    );
    assert!(
        content.contains("present_counter"),
        "present_counter() not found in src/wasm.rs"
    );
}

#[test]
fn recipe_1_wasm_backend_exists() {
    let content = fs::read_to_string("src/shell/platform/wasm.rs")
        .expect("Failed to read src/shell/platform/wasm.rs");

    assert!(
        content.contains("impl Backend"),
        "Backend trait implementation not found in wasm.rs"
    );
}

#[test]
fn recipe_1_line_numbers_accurate() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");
    let lines: Vec<&str> = content.lines().collect();

    // Line 64: use clock::Moment; (updated for Step 11 docs)
    assert!(
        lines[63].contains("use clock::Moment"),
        "use clock::Moment; not found at expected location. Got: {}",
        lines.get(63).unwrap_or(&"")
    );

    // Line 170: trait Backend (updated for Step 11 docs)
    assert!(
        lines[169].contains("trait Backend"),
        "trait Backend not found at expected location. Got: {}",
        lines.get(169).unwrap_or(&"")
    );

    // Line 204: struct Surface (updated for Step 11 docs)
    assert!(
        lines[203].contains("struct Surface"),
        "struct Surface not found at expected location. Got: {}",
        lines.get(203).unwrap_or(&"")
    );

    // Line 217: drawn_at: Moment, (updated for Step 11 docs)
    assert!(
        lines[216].contains("drawn_at:") && lines[216].contains("Moment"),
        "drawn_at: Moment not found at expected location. Got: {}",
        lines.get(216).unwrap_or(&"")
    );

    // Line 255: let now = Moment::now(); (updated for Step 11 docs)
    assert!(
        lines[254].contains("Moment::now"),
        "Moment::now() not found at expected location. Got: {}",
        lines.get(254).unwrap_or(&"")
    );

    // Line 343: fn turn<S> (updated for Step 11 docs)
    assert!(
        lines[342].contains("fn turn"),
        "fn turn<S> not found at expected location. Got: {}",
        lines.get(342).unwrap_or(&"")
    );

    // Line 387: pub(crate) fn run<S: 'static> for native (updated for Step 11 docs)
    assert!(
        lines[386].contains("pub(crate) fn run") && !lines[386].contains("wasm"),
        "pub(crate) fn run (native) not found at expected location. Got: {}",
        lines.get(386).unwrap_or(&"")
    );

    // Line 433: pub(crate) fn run<S: 'static> for WASM (updated for Step 11 docs, preceded by #[cfg(target_arch = "wasm32")])
    assert!(
        lines[432].contains("pub(crate) fn run"),
        "pub(crate) fn run (WASM) not found at expected location. Got: {}",
        lines.get(432).unwrap_or(&"")
    );
    // Verify WASM config is near the line before
    assert!(
        lines[429..432].iter().any(|l| l.contains("wasm32")),
        "WASM run() should be preceded by wasm32 cfg attribute"
    );
}

#[test]
fn recipe_1_continues_helper_exists() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    assert!(
        content.contains("fn continues<S>"),
        "continues() helper function not found in src/shell/mod.rs"
    );
}

#[test]
fn recipe_1_page_struct_exists() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    assert!(
        content.contains("struct Page<S>"),
        "Page struct not found in src/shell/mod.rs"
    );
}

#[test]
fn recipe_1_continues_called_in_native_loop() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    // Verify continues() is called in the native run() loop (while continues(...))
    assert!(
        content.contains("while continues(&window, &surface, &app)"),
        "continues() not called in native run() loop"
    );
}

#[test]
fn recipe_1_page_used_in_wasm_loop() {
    let content = fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    // Verify Page struct is created and used in WASM run()
    assert!(
        content.contains("let page = Rc::new(RefCell::new(Some(Page {"),
        "Page struct not instantiated in WASM run()"
    );
}
