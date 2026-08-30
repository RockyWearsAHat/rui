//! Verify that Recipe 1 documentation is accurate against the actual codebase.
//!
//! This test ensures that:
//! - All commits mentioned in CLAUDE.md Recipe 1 exist
//! - All files mentioned exist and are accessible
//! - Key architectural elements (Backend trait, turn() function, run() implementations) exist at expected locations

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

    // Line 55: use clock::Moment;
    assert!(
        lines[54].contains("use clock::Moment"),
        "Line 55: use clock::Moment; not found at expected line. Got: {}",
        lines.get(54).unwrap_or(&"")
    );

    // Line 152: trait Backend
    assert!(
        lines[151].contains("trait Backend"),
        "Line 152: trait Backend not found at expected line. Got: {}",
        lines.get(151).unwrap_or(&"")
    );

    // Line 186: struct Surface
    assert!(
        lines[185].contains("struct Surface"),
        "Line 186: struct Surface not found at expected line. Got: {}",
        lines.get(185).unwrap_or(&"")
    );

    // Line 199: drawn_at: Moment,
    assert!(
        lines[198].contains("drawn_at:") && lines[198].contains("Moment"),
        "Line 199: drawn_at: Moment not found at expected line. Got: {}",
        lines.get(198).unwrap_or(&"")
    );

    // Line 237: let now = Moment::now();
    assert!(
        lines[236].contains("Moment::now"),
        "Line 237: Moment::now() not found at expected line. Got: {}",
        lines.get(236).unwrap_or(&"")
    );

    // Line 325: fn turn<S>
    assert!(
        lines[324].contains("fn turn"),
        "Line 325: fn turn<S> not found at expected line. Got: {}",
        lines.get(324).unwrap_or(&"")
    );

    // Line 369: pub(crate) fn run<S: 'static> for native
    assert!(
        lines[368].contains("pub(crate) fn run") && !lines[368].contains("wasm"),
        "Line 369: pub(crate) fn run (native) not found at expected line. Got: {}",
        lines.get(368).unwrap_or(&"")
    );

    // Line 415: pub(crate) fn run<S: 'static> for WASM (preceded by #[cfg(target_arch = "wasm32")])
    assert!(
        lines[414].contains("pub(crate) fn run"),
        "Line 415: pub(crate) fn run (WASM) not found at expected line. Got: {}",
        lines.get(414).unwrap_or(&"")
    );
    // Verify WASM config is on the line before
    assert!(
        lines[411..414].iter().any(|l| l.contains("wasm32")),
        "WASM run() at line 415 should be preceded by wasm32 cfg attribute"
    );
}
