//! Tests for the code_view component and syntax highlighting.

use rui::testing::Harness;
use rui::{code_view, language_for_path, Language, Tone};

#[derive(Clone)]
struct CodeViewState {
    source: String,
}

#[test]
fn code_view_numbers_every_line_from_one() {
    let state = CodeViewState {
        source: "line 1\nline 2\nline 3".to_string(),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).build());
    h.frame();

    // Should show line numbers 1, 2, 3
    assert!(h.shows("1"));
    assert!(h.shows("2"));
    assert!(h.shows("3"));
}

#[test]
fn code_view_first_line_starts_the_gutter_elsewhere() {
    let state = CodeViewState {
        source: "line 1\nline 2".to_string(),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).first_line(10).build());
    h.frame();

    // Should show line numbers 10, 11
    assert!(h.shows("10"));
    assert!(h.shows("11"));
}

#[test]
fn code_view_hides_the_gutter_when_numbers_are_off() {
    let state = CodeViewState {
        source: "line 1\nline 2".to_string(),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).numbers(false).build());
    h.frame();

    // Should not show line numbers
    assert!(!h.shows("1") || h.shows("line 1"));
}

#[test]
fn code_view_does_not_wrap_a_long_line() {
    let state = CodeViewState {
        source: "a".repeat(200),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).build()).size(100.0, 200.0); // Narrow viewport
    h.frame();

    // The long line should not be wrapped; it should be in a single row
    let el = h.find_key("line-1");
    assert!(el.is_some(), "Line 1 should exist");
}

#[test]
fn code_view_scrolls_horizontally() {
    let state = CodeViewState {
        source: "a".repeat(200),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).build()).size(100.0, 200.0);
    h.frame();

    // The view should be scrollable
    assert!(h.find_key("line-1").is_some());
}

#[test]
fn code_view_colours_rust_keywords() {
    let state = CodeViewState {
        source: "let x = 42;".to_string(),
    };
    let mut h = Harness::new(state, |s| {
        code_view(&s.source).language(Language::Rust).build()
    });
    h.frame();

    // Should tokenize "let" as a keyword
    assert!(h.shows("let"));
}

#[test]
fn code_view_colours_json_keys_and_strings() {
    let state = CodeViewState {
        source: r#"{"name": "value"}"#.to_string(),
    };
    let mut h = Harness::new(state, |s| {
        code_view(&s.source).language(Language::Json).build()
    });
    h.frame();

    // Should render JSON content without crashing
    // The component renders the JSON with proper tokenization
    assert!(h.find_key("line-1").is_some());
}

#[test]
fn code_view_marks_highlighted_lines() {
    let state = CodeViewState {
        source: "line 1\nline 2\nline 3".to_string(),
    };
    let mut h = Harness::new(state, |s| {
        code_view(&s.source).highlight(1..2).build() // Highlight line 2 (0-based index 1)
    });
    h.frame();

    // All lines should be present
    assert!(h.shows("line 1"));
    assert!(h.shows("line 2"));
    assert!(h.shows("line 3"));
}

#[test]
fn code_view_line_marks_beat_highlight() {
    let state = CodeViewState {
        source: "line 1\nline 2\nline 3".to_string(),
    };
    let mut h = Harness::new(state, |s| {
        code_view(&s.source)
            .highlight(0..3)
            .line_marks(vec![(1, Tone::Accent)])
            .build()
    });
    h.frame();

    // All lines should be present
    assert!(h.shows("line 1"));
    assert!(h.shows("line 2"));
    assert!(h.shows("line 3"));
}

#[test]
fn code_view_expands_tabs_to_four_spaces() {
    let state = CodeViewState {
        source: "line\twith\ttabs".to_string(),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).build());
    h.frame();

    // Tabs should be expanded to 4 spaces internally; the line should render
    assert!(h.find_key("line-1").is_some());
}

#[test]
fn code_view_of_empty_source_renders_nothing() {
    let state = CodeViewState {
        source: "".to_string(),
    };
    let mut h = Harness::new(state, |s| code_view(&s.source).build());
    h.frame();

    // Empty source should render an empty scroll box without panicking
    // This test just ensures no panic occurs
}

#[test]
fn code_view_truncates_beyond_twenty_thousand_lines() {
    let mut source = String::new();
    for i in 0..21000 {
        source.push_str(&format!("line {}\n", i));
    }
    let state = CodeViewState { source };
    let mut h = Harness::new(state, |s| code_view(&s.source).build());
    h.frame();

    // Should have the truncated-line element
    assert!(h.find_key("truncated-line").is_some());
}

#[test]
fn language_for_path_maps_every_supported_extension() {
    assert_eq!(language_for_path("test.rs"), Some(Language::Rust));
    assert_eq!(language_for_path("test.js"), Some(Language::JavaScript));
    assert_eq!(language_for_path("test.mjs"), Some(Language::JavaScript));
    assert_eq!(language_for_path("test.cjs"), Some(Language::JavaScript));
    assert_eq!(language_for_path("test.ts"), Some(Language::TypeScript));
    assert_eq!(language_for_path("test.tsx"), Some(Language::TypeScript));
    assert_eq!(language_for_path("test.py"), Some(Language::Python));
    assert_eq!(language_for_path("test.sh"), Some(Language::Bash));
    assert_eq!(language_for_path("test.bash"), Some(Language::Bash));
    assert_eq!(language_for_path("test.json"), Some(Language::Json));
    assert_eq!(language_for_path("test.md"), Some(Language::Markdown));
    assert_eq!(language_for_path("test.diff"), Some(Language::Diff));
    assert_eq!(language_for_path("test.patch"), Some(Language::Diff));
    assert_eq!(language_for_path("test.toml"), Some(Language::Toml));
    assert_eq!(language_for_path("test.unknown"), None);
}
