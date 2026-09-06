//! Tests for markdown rendering.

use rui::testing::Harness;
use rui::{markdown, markdown_with};

struct State;

#[test]
fn markdown_renders_three_heading_levels() {
    let view = |_: &State| markdown("# Heading 1\n## Heading 2\n### Heading 3");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Heading 1"));
    assert!(harness.shows("Heading 2"));
    assert!(harness.shows("Heading 3"));
}

#[test]
fn markdown_renders_paragraphs_split_on_blank_lines() {
    let view = |_: &State| markdown("First paragraph.\n\nSecond paragraph.");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("First paragraph."));
    assert!(harness.shows("Second paragraph."));
}

#[test]
fn markdown_renders_bullet_and_numbered_lists() {
    let view = |_: &State| markdown("- Item 1\n- Item 2\n\n1. First\n2. Second");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Item 1"));
    assert!(harness.shows("Item 2"));
    assert!(harness.shows("First"));
    assert!(harness.shows("Second"));
}

#[test]
fn markdown_fenced_block_becomes_a_code_view() {
    let view = |_: &State| markdown("```rust\nfn main() {}\n```");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("fn main() {}"));
}

#[test]
fn markdown_fence_info_string_picks_the_language() {
    let view = |_: &State| markdown("```python\ndef hello():\n    pass\n```");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("def hello():"));
    assert!(harness.shows("    pass"));
}

#[test]
fn markdown_inline_code_is_monospaced() {
    let view = |_: &State| markdown("The `code` is here.");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("The "));
    assert!(harness.shows("code"));
    assert!(harness.shows(" is here."));
}

#[test]
fn markdown_bold_is_bold_and_emphasis_is_not_italic() {
    let view =
        |_: &State| markdown("**bold** and __also bold__ and *emphasis* and _also emphasis_");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("bold"));
    assert!(harness.shows("also bold"));
    assert!(harness.shows("emphasis"));
    assert!(harness.shows("also emphasis"));
}

#[test]
fn markdown_link_reports_its_href_when_clicked() {
    let view = |_: &State| {
        markdown_with(
            "[Click me](https://example.com)",
            |_: &mut State, href: String| {
                println!("Clicked link: {}", href);
            },
        )
    };

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Click me"));

    // Click on the link text
    harness.click_text("Click me");
    harness.frame();
}

#[test]
fn markdown_table_becomes_a_table_with_a_header() {
    let view = |_: &State| markdown("| Header 1 | Header 2 |\n|---|---|\n| Cell 1 | Cell 2 |");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Header 1"));
    assert!(harness.shows("Header 2"));
    assert!(harness.shows("Cell 1"));
    assert!(harness.shows("Cell 2"));
}

#[test]
fn markdown_strips_html_and_keeps_its_text() {
    let view = |_: &State| markdown("Some <b>HTML</b> text <a href=\"#\">here</a>");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Some "));
    assert!(harness.shows("HTML"));
    assert!(harness.shows(" text "));
    assert!(harness.shows("here"));
}

#[test]
fn markdown_image_becomes_a_placeholder() {
    let view = |_: &State| markdown("Some text ![alt text](image.png) more text");

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(harness.shows("Some text "));
    assert!(harness.shows("[image: alt text]"));
    assert!(harness.shows(" more text"));
}

#[test]
fn markdown_of_a_readme_opening_with_a_badge_does_not_panic() {
    let readme = r#"[![Build Status](https://example.com/badge.svg)](https://example.com)

# Project Title

A description of the project.

## Features

- Feature 1
- Feature 2

## Installation

```bash
cargo install my-project
```

## Usage

Run `my-project --help` for more information.
"#;

    let view = |_: &State| markdown(readme);

    let mut harness = Harness::new(State, view).size(400.0, 800.0);
    harness.frame();

    assert!(harness.shows("Project Title"));
    assert!(harness.shows("Features"));
}
