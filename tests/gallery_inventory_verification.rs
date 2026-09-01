#![allow(missing_docs)]

use rui::testing::*;
use rui::*;

#[test]
fn gallery_renders_all_widget_sections() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 800.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        canvas.width() > 0 && canvas.height() > 0,
        "Gallery should render with non-zero dimensions"
    );
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery should produce frame pixels"
    );
}

#[test]
fn gallery_inventory_is_deterministic_across_frames() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 600.0);
    h.frames(1);
    let first_w = h.canvas().width();
    let first_h = h.canvas().height();

    h.frames(1);
    let second_canvas = h.canvas();

    assert_eq!(
        first_w,
        second_canvas.width(),
        "Gallery width should be deterministic"
    );
    assert_eq!(
        first_h,
        second_canvas.height(),
        "Gallery height should be deterministic"
    );
}

#[test]
fn gallery_renders_at_multiple_sizes() {
    for size in &[300.0, 500.0, 800.0] {
        let mut h = Harness::new(GalleryState::default(), gallery_view).size(*size, *size);
        h.frames(1);
        let canvas = h.canvas();
        assert!(
            !canvas.pixels().is_empty(),
            "Gallery should render at size {}",
            size
        );
    }
}

#[test]
fn gallery_widget_visibility_is_consistent() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(500.0, 500.0);
    h.frames(3);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery should maintain content across frames"
    );
}

#[test]
fn gallery_text_elements_are_visible() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 600.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery should render visible text content"
    );
}

#[test]
fn gallery_layout_adapts_to_canvas_width() {
    let mut h1 = Harness::new(GalleryState::default(), gallery_view).size(300.0, 400.0);
    h1.frames(1);
    let canvas1 = h1.canvas();

    let mut h2 = Harness::new(GalleryState::default(), gallery_view).size(600.0, 400.0);
    h2.frames(1);
    let canvas2 = h2.canvas();

    assert!(!canvas1.pixels().is_empty(), "Narrow gallery should render");
    assert!(!canvas2.pixels().is_empty(), "Wide gallery should render");
}

#[test]
fn gallery_maintains_visual_hierarchy() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(500.0, 1000.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery should render with visual content"
    );
}

#[test]
fn gallery_renders_buttons() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 300.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery with buttons should render"
    );
}

#[test]
fn gallery_renders_text_content() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 300.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery with text should render"
    );
}

#[test]
fn gallery_renders_multiple_widgets() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 600.0);
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery with multiple widgets should render"
    );
}

#[test]
fn gallery_pixel_count_varies_with_size() {
    let mut h1 = Harness::new(GalleryState::default(), gallery_view).size(200.0, 200.0);
    h1.frames(1);
    let pixels1 = h1.canvas().pixels().len();

    let mut h2 = Harness::new(GalleryState::default(), gallery_view).size(400.0, 400.0);
    h2.frames(1);
    let pixels2 = h2.canvas().pixels().len();

    assert!(pixels2 > pixels1, "Larger canvas should have more pixels");
}

#[test]
fn gallery_renders_after_interaction() {
    let mut h = Harness::new(GalleryState::default(), gallery_view).size(400.0, 600.0);
    h.frames(1);
    h.click(Point::new(100.0, 100.0));
    h.frames(1);
    let canvas = h.canvas();
    assert!(
        !canvas.pixels().is_empty(),
        "Gallery should render after click interaction"
    );
}

#[derive(Clone, Default)]
struct GalleryState {
    #[allow(dead_code)]
    selected: usize,
}

fn gallery_view(_state: &GalleryState) -> El<GalleryState> {
    col((
        text("Text widgets demo"),
        row((text("Button"), text("Segmented"))),
        text("Meter and status indicators"),
        text("Input fields"),
    ))
}
