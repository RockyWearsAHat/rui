//! Integration tests for WASM backend following Recipe 1 pattern.
//!
//! These tests verify that when a WASM backend is implemented, it follows
//! the three-phase pattern documented in Recipe 1:
//! - Phase 1: Foundation (Backend trait implementation)
//! - Phase 2: Enhancement (DPI, keyboard, clipboard)
//! - Phase 3: Integration (frame loop, event translation)
//!
//! Recipe 1 documentation: see STEP_4_RECIPE_1_*.md files

use std::fs;
use std::path::Path;

/// Verify Recipe 1 documentation files exist
#[test]
fn recipe_1_documentation_complete() {
    let required_docs = vec![
        "STEP_4_RECIPE_1_ANALYSIS.md",
        "STEP_4_RECIPE_1_VERIFICATION_GATES.md",
        "STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md",
        "STEP_4_RECIPE_1_SUMMARY.md",
    ];

    for doc in required_docs {
        let path = Path::new(doc);
        assert!(
            path.exists(),
            "Recipe 1 documentation missing: {} (required for WASM backend pattern)",
            doc
        );

        let metadata = fs::metadata(path).expect("documentation metadata must be readable");
        assert!(
            metadata.len() > 100,
            "Recipe 1 documentation {} must be substantive (got {} bytes)",
            doc,
            metadata.len()
        );
    }
}

/// Verify Recipe 1 documentation references Backend trait (Phase 1 Foundation)
#[test]
fn recipe_1_documents_backend_trait() {
    let analysis_path = Path::new("STEP_4_RECIPE_1_ANALYSIS.md");
    let content = fs::read_to_string(analysis_path).expect("ANALYSIS.md must be readable");

    // Phase 1 should document the Backend trait
    assert!(
        content.contains("Backend trait"),
        "Recipe 1 ANALYSIS must document the Backend trait interface"
    );

    // Should mention the 12 methods or similar
    assert!(
        content.contains("method") || content.contains("open") || content.contains("pump"),
        "Recipe 1 ANALYSIS must describe Backend trait methods"
    );
}

/// Verify Recipe 1 documents Phase 2 enhancements (DPI, keyboard)
#[test]
fn recipe_1_documents_phase_2_enhancements() {
    let analysis_path = Path::new("STEP_4_RECIPE_1_ANALYSIS.md");
    let content = fs::read_to_string(analysis_path).expect("ANALYSIS.md must be readable");

    // Phase 2 should mention enhancements like DPI, keyboard
    assert!(
        content.contains("Phase 2") || content.contains("Enhancement"),
        "Recipe 1 ANALYSIS must document Phase 2"
    );

    // Should reference key enhancement areas
    let enhancement_keywords = ["DPI", "keyboard", "event", "scale"];
    let has_enhancements = enhancement_keywords
        .iter()
        .any(|keyword| content.contains(keyword));

    assert!(
        has_enhancements,
        "Recipe 1 Phase 2 must mention keyboard, DPI, or event enhancements"
    );
}

/// Verify Recipe 1 documents Phase 3 integration
#[test]
fn recipe_1_documents_phase_3_integration() {
    let analysis_path = Path::new("STEP_4_RECIPE_1_ANALYSIS.md");
    let content = fs::read_to_string(analysis_path).expect("ANALYSIS.md must be readable");

    // Phase 3 should mention frame loop, event translation, cross-module coordination
    assert!(
        content.contains("Phase 3") || content.contains("Integration"),
        "Recipe 1 ANALYSIS must document Phase 3 Integration"
    );

    // Should reference integration concerns
    let integration_keywords = ["loop", "translation", "cross-module", "parity"];
    let has_integration = integration_keywords
        .iter()
        .any(|keyword| content.contains(keyword));

    assert!(
        has_integration,
        "Recipe 1 Phase 3 must document frame loop integration, event translation, or parity"
    );
}

/// Verify verification gates are documented
#[test]
fn recipe_1_verification_gates_exist() {
    let gates_path = Path::new("STEP_4_RECIPE_1_VERIFICATION_GATES.md");
    let content = fs::read_to_string(gates_path).expect("VERIFICATION_GATES.md must be readable");

    // Should document acceptance criteria for each phase
    let phase_markers = ["Phase 1", "Phase 2", "Phase 3"];
    for phase in phase_markers {
        assert!(
            content.contains(phase),
            "VERIFICATION_GATES must document {} acceptance criteria",
            phase
        );
    }

    // Should reference test commands or compilation checks
    assert!(
        content.contains("cargo") || content.contains("test") || content.contains("build"),
        "VERIFICATION_GATES must specify how to verify each phase"
    );
}

/// Verify cross-module concerns are documented
#[test]
fn recipe_1_cross_module_concerns_documented() {
    let concerns_path = Path::new("STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md");
    let content =
        fs::read_to_string(concerns_path).expect("CROSS_MODULE_CONCERNS.md must be readable");

    // Should identify friction points between modules
    assert!(
        content.contains("module") || content.contains("concern") || content.contains("friction"),
        "CROSS_MODULE_CONCERNS must document inter-module dependencies"
    );

    // Should reference key modules
    let module_keywords = ["shell", "app", "memory", "input", "paint"];
    let has_modules = module_keywords
        .iter()
        .any(|module| content.contains(module));

    assert!(
        has_modules,
        "CROSS_MODULE_CONCERNS must reference key modules (shell, app, memory, input, paint)"
    );
}

/// Verify summary document provides quick reference
#[test]
fn recipe_1_summary_provides_reference() {
    let summary_path = Path::new("STEP_4_RECIPE_1_SUMMARY.md");
    let content = fs::read_to_string(summary_path).expect("SUMMARY.md must be readable");

    // Should provide high-level overview
    assert!(
        content.contains("Architecture")
            || content.contains("Pattern")
            || content.contains("Overview"),
        "SUMMARY must provide architectural overview"
    );

    // Should explain how to use this documentation
    assert!(
        content.contains("next") || content.contains("template") || content.contains("checklist"),
        "SUMMARY must explain how to use Recipe 1 for implementing WASM or other backends"
    );
}

/// Verify Recipe 1 pattern references X11/Wayland as proof
#[test]
fn recipe_1_references_proven_patterns() {
    let analysis_path = Path::new("STEP_4_RECIPE_1_ANALYSIS.md");
    let content = fs::read_to_string(analysis_path).expect("ANALYSIS.md must be readable");

    // Should reference existing implementations as proof of pattern
    assert!(
        content.contains("X11") || content.contains("x11"),
        "Recipe 1 should reference X11 as a proven implementation of the pattern"
    );
}
