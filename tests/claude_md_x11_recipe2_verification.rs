//! Verify X11 Recipe 2 commits with documented phase boundaries.
//!
//! This test verifies that the four X11 backend implementation commits
//! documented in CLAUDE.md exist in git history with the correct commit
//! messages and line counts per phase.
//!
//! Phase 1 (Foundation): a67d578, 748 lines
//! Phase 2 (Enhancement): c42c0f0, 1220 lines total
//! Phase 3 (Integration): 80e3003, 1321 lines total
//! Phase 4 (Polish): 991167a, 1368 lines total

#[test]
fn x11_recipe_2_phase_1_foundation() {
    // Phase 1 (a67d578): Foundation — 748 lines of x11.rs added
    // Message: "Give the interface library a foundation you can build controls on"
    let output = std::process::Command::new("git")
        .args(["show", "a67d578", "--format=%s", "-s"])
        .current_dir(".")
        .output()
        .expect("Failed to run git");

    let message = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        message.contains("Give the interface library a foundation"),
        "Phase 1 message mismatch: {}",
        message
    );

    // Verify x11.rs file size
    let output = std::process::Command::new("git")
        .args(["show", "a67d578:src/shell/platform/x11.rs"])
        .current_dir(".")
        .output()
        .expect("Failed to get Phase 1 x11.rs");

    let lines = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .lines()
        .count();
    assert_eq!(lines, 748, "Phase 1: expected 748 lines, got {}", lines);
}

#[test]
fn x11_recipe_2_phase_2_enhancement() {
    // Phase 2 (c42c0f0): Enhancement — full Backend trait + event translation
    // Message: "Bring the library up to the selfhost workspace's current state"
    let output = std::process::Command::new("git")
        .args(["show", "c42c0f0", "--format=%s", "-s"])
        .current_dir(".")
        .output()
        .expect("Failed to run git");

    let message = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        message.contains("Bring the library up"),
        "Phase 2 message mismatch: {}",
        message
    );

    // Verify x11.rs file size
    let output = std::process::Command::new("git")
        .args(["show", "c42c0f0:src/shell/platform/x11.rs"])
        .current_dir(".")
        .output()
        .expect("Failed to get Phase 2 x11.rs");

    let lines = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .lines()
        .count();
    assert_eq!(lines, 1220, "Phase 2: expected 1220 lines, got {}", lines);
}

#[test]
fn x11_recipe_2_phase_3_integration() {
    // Phase 3 (80e3003): Integration — frame loop wiring, cross-module coordination
    // Message: "The four primitives a remote-desktop viewport needs"
    let output = std::process::Command::new("git")
        .args(["show", "80e3003", "--format=%s", "-s"])
        .current_dir(".")
        .output()
        .expect("Failed to run git");

    let message = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        message.contains("four primitives"),
        "Phase 3 message mismatch: {}",
        message
    );

    // Verify x11.rs file size
    let output = std::process::Command::new("git")
        .args(["show", "80e3003:src/shell/platform/x11.rs"])
        .current_dir(".")
        .output()
        .expect("Failed to get Phase 3 x11.rs");

    let lines = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .lines()
        .count();
    assert_eq!(lines, 1321, "Phase 3: expected 1321 lines, got {}", lines);
}

#[test]
fn x11_recipe_2_phase_4_polish() {
    // Phase 4 (991167a): Polish — documentation refinements, widget exemplar
    // Message: "Recipe 2: Implement star_rating widget exemplar with test"
    let output = std::process::Command::new("git")
        .args(["show", "991167a", "--format=%s", "-s"])
        .current_dir(".")
        .output()
        .expect("Failed to run git");

    let message = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        message.contains("star_rating widget exemplar"),
        "Phase 4 message mismatch: {}",
        message
    );

    // Verify x11.rs file size
    let output = std::process::Command::new("git")
        .args(["show", "991167a:src/shell/platform/x11.rs"])
        .current_dir(".")
        .output()
        .expect("Failed to get Phase 4 x11.rs");

    let lines = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .lines()
        .count();
    assert_eq!(lines, 1368, "Phase 4: expected 1368 lines, got {}", lines);
}

#[test]
fn x11_recipe_2_all_commits_exist() {
    // Verify all four commits are in history
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "--all"])
        .current_dir(".")
        .output()
        .expect("Failed to run git log");

    let history = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        history.contains("a67d578"),
        "Phase 1 commit a67d578 not found"
    );
    assert!(
        history.contains("c42c0f0"),
        "Phase 2 commit c42c0f0 not found"
    );
    assert!(
        history.contains("80e3003"),
        "Phase 3 commit 80e3003 not found"
    );
    assert!(
        history.contains("991167a"),
        "Phase 4 commit 991167a not found"
    );
}
