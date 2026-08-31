//! What actually reaches the pixels.
//!
//! The other suites assert about rectangles and state, which a renderer could
//! satisfy while drawing nothing at all. These assert on the buffer: that marks
//! are made where they were meant to be, that the same description twice is the
//! same picture, and that an animation both moves and then stops.

use rui_native::testing::Harness;
use rui_native::{button, col, draw, row, spacer, text, Align, Appearance, El, Radius, Size, Tone};

/// Nothing to hold: these are about pixels.
#[derive(Default)]
struct Nothing;

/// A state a hover test can watch.
#[derive(Default)]
struct Watched {
    clicks: u32,
}

/// A harness showing `view`, at a size these tests can reason about.
fn showing(view: impl Fn(&Nothing) -> El<Nothing> + 'static) -> Harness<Nothing> {
    Harness::new(Nothing, view).size(200.0, 100.0)
}

#[test]
fn a_window_is_drawn_in_the_ground_of_whichever_appearance_is_in_force() {
    let mut dark = showing(|_| col(())).appearance(Appearance::Dark);
    let mut light = showing(|_| col(())).appearance(Appearance::Light);
    dark.frame();
    light.frame();

    let dark_ground = dark.pixel(100, 50).expect("inside the window");
    let light_ground = light.pixel(100, 50).expect("inside the window");
    assert_ne!(
        dark_ground, light_ground,
        "one description, two appearances"
    );
    assert!(
        light_ground.luminance() > dark_ground.luminance(),
        "and the light one is lighter"
    );
}

#[test]
fn text_makes_marks_where_the_layout_put_it() {
    let mut harness = showing(|_| col(text("abc").text_size(20.0)).align(Align::Start));
    harness.frame();

    let rect = harness.rect_of("abc").expect("the run is on screen");
    assert!(harness.marked(rect), "the run should have drawn something");
    assert!(
        !harness.marked(rui_native::Rect::new(
            rect.max_x() + 20.0,
            rect.y,
            40.0,
            rect.h
        )),
        "and nothing beyond its own end"
    );
}

#[test]
fn an_empty_window_is_marked_nowhere() {
    let mut harness = showing(|_| col(()));
    harness.frame();
    assert!(!harness.marked(rui_native::Rect::new(0.0, 0.0, 200.0, 100.0)));
}

#[test]
fn the_same_description_twice_is_the_same_picture() {
    // What the loop relies on to decide it has nothing to present: a frame that
    // came out identical is one the screen already shows. A renderer with any
    // state carried between frames would fail this.
    let mut harness = showing(|_| {
        col((
            text("Services").text_size(14.0),
            button("Restart"),
            spacer().h(10.0),
        ))
        .pad(8.0)
    });
    harness.frame();
    let first: Vec<u32> = harness.canvas().pixels().to_vec();

    harness.frame();
    assert_eq!(
        first,
        harness.canvas().pixels(),
        "an unchanged interface must redraw identically"
    );
}

#[test]
fn two_different_words_are_two_different_pictures() {
    let mut one = showing(|_| col(text("aaaa").text_size(20.0)).align(Align::Start));
    let mut other = showing(|_| col(text("bbbb").text_size(20.0)).align(Align::Start));
    one.frame();
    other.frame();
    assert_ne!(one.canvas().pixels(), other.canvas().pixels());
}

#[test]
fn a_hover_eases_in_over_several_frames_and_then_settles() {
    let mut harness = Harness::new(Watched::default(), |_: &Watched| {
        col(button("Restart").on_click(|watched: &mut Watched| watched.clicks += 1))
    })
    .size(200.0, 100.0);

    harness.frame();
    assert!(
        !harness.is_animating(),
        "an interface nobody is touching must not keep drawing"
    );
    let at_rest: Vec<u32> = harness.canvas().pixels().to_vec();

    harness.hover_text("Restart");
    assert!(
        harness.is_animating(),
        "the pointer arriving starts the hover moving"
    );
    let part_way: Vec<u32> = harness.canvas().pixels().to_vec();
    assert_ne!(at_rest, part_way, "and it is visibly under way");

    // Time is given, not read, so an animation is stepped rather than waited
    // for. A second is far past any hover the theme allows itself.
    harness.frames(60);
    assert!(
        !harness.is_animating(),
        "an animation that never settles never stops drawing"
    );
    let settled: Vec<u32> = harness.canvas().pixels().to_vec();
    assert_ne!(
        part_way, settled,
        "it got further than where it was part way"
    );

    harness.frame();
    assert_eq!(
        settled,
        harness.canvas().pixels(),
        "and having settled, it stays"
    );
}

#[test]
fn a_disabled_control_is_drawn_differently_from_an_available_one() {
    let mut available = showing(|_| col(button("Start").on_click(|_: &mut Nothing| {})));
    let mut unavailable = showing(|_| {
        col(button("Start")
            .on_click(|_: &mut Nothing| {})
            .disabled(true))
    });
    available.frame();
    unavailable.frame();
    assert_ne!(
        available.canvas().pixels(),
        unavailable.canvas().pixels(),
        "a control nobody can use has to look like one"
    );
}

#[test]
fn an_applications_own_drawing_sees_what_a_button_sees() {
    // The whole point of the escape hatch: a control the library has never
    // heard of reacts exactly as one it ships does, because it is handed the
    // same answer about the pointer.
    let mut harness = Harness::new(Watched::default(), |_: &Watched| {
        col(draw(Size::new(60.0, 30.0), |painter, rect| {
            let tone = if painter.visual().hovered {
                Tone::Ok
            } else {
                Tone::Bad
            };
            painter.fill(rect, Radius::None, tone);
        })
        .key("custom")
        .on_click(|watched: &mut Watched| watched.clicks += 1))
    })
    .size(200.0, 100.0);

    harness.frame();
    let rect = harness
        .find_key("custom")
        .expect("the control is on screen")
        .rect;
    let at_rest = harness
        .pixel(rect.center().x as u32, rect.center().y as u32)
        .expect("a pixel");

    harness.move_pointer(rect.center());
    let hovered = harness
        .pixel(rect.center().x as u32, rect.center().y as u32)
        .expect("a pixel");
    assert_ne!(
        at_rest, hovered,
        "custom drawing must be able to answer the pointer"
    );
}

#[test]
fn a_higher_pixel_density_draws_more_pixels_of_the_same_picture() {
    let mut plain = showing(|_| col(text("abc").text_size(20.0)));
    let mut dense = showing(|_| col(text("abc").text_size(20.0))).scale(2.0);
    plain.frame();
    dense.frame();

    assert_eq!(plain.canvas().width(), 200);
    assert_eq!(
        dense.canvas().width(),
        400,
        "twice the density is twice the pixels across"
    );
    assert_eq!(
        dense.canvas().pixels().len(),
        plain.canvas().pixels().len() * 4
    );
}

#[test]
fn a_layer_is_drawn_over_what_it_covers() {
    let mut harness = showing(|_| {
        col(row(text("underneath"))
            .h(40.0)
            .fill(Tone::Bad)
            .key("under")
            .add(
                col(())
                    .size(200.0, 40.0)
                    .fill(Tone::Ok)
                    .key("over")
                    .layer(rui_native::Anchor::Over),
            ))
    });
    harness.frame();

    let under = harness.find_key("under").expect("the pane").rect;
    let drawn = harness
        .pixel(under.center().x as u32, under.center().y as u32)
        .expect("a pixel");
    let mut alone = showing(|_| col(row(()).h(40.0).fill(Tone::Ok)));
    alone.frame();
    let expected = alone
        .pixel(under.center().x as u32, under.center().y as u32)
        .expect("a pixel");

    assert_eq!(
        drawn, expected,
        "the layer, not what it covers, is what is on top"
    );
}

#[test]
fn a_frame_can_be_written_out_as_a_png() {
    // What a project's screenshots should be made with: the same code that
    // draws the real window, so they cannot fall out of date with it.
    let mut harness = showing(|_| col(text("Services").text_size(14.0)).pad(8.0));
    harness.frame();

    let path = std::env::temp_dir().join("rui-rendering-test.png");
    harness
        .save_png(&path)
        .expect("the frame should be writable");

    let written = std::fs::read(&path).expect("the file should be there");
    assert_eq!(&written[..8], b"\x89PNG\r\n\x1a\n", "and be a PNG");
    std::fs::remove_file(&path).ok();
}

// The browser suite. Only compiled for wasm — everywhere else there is no page
// to draw into or listen to, and these assertions would have nothing to say.
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// Builds the page the backend expects to find: one canvas, named `surface`.
#[cfg(target_arch = "wasm32")]
fn surface_on_the_page(width: u32, height: u32) -> web_sys::HtmlCanvasElement {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .expect("a window")
        .document()
        .expect("a document");
    let surface: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .expect("a canvas")
        .dyn_into()
        .expect("a canvas element");
    surface.set_id("surface");
    surface.set_width(width);
    surface.set_height(height);
    document
        .body()
        .expect("a body")
        .append_child(&surface)
        .expect("appended");
    surface
}

/// The browser is a backend like any other: the same frame, the same buffer.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm_backend_renders_to_canvas() {
    use wasm_bindgen::JsCast;

    let mut harness = showing(|_| {
        draw(Size::new(200.0, 100.0), |painter, rect| {
            painter.fill(rect, Radius::None, Tone::Bad)
        })
    });
    harness.frame();
    let red = harness.pixel(100, 50).expect("the rectangle covers it");

    let surface = surface_on_the_page(harness.canvas().width(), harness.canvas().height());
    rui_native::shell::present(harness.canvas()).expect("the frame should reach the canvas");

    // Read the pixels back out of the DOM, not out of rui.
    let context: web_sys::CanvasRenderingContext2d = surface
        .get_context("2d")
        .expect("a context")
        .expect("a context")
        .dyn_into()
        .expect("a 2d context");
    let read = context
        .get_image_data(100.0, 50.0, 1.0, 1.0)
        .expect("the pixel")
        .data();

    assert_eq!(
        (read[0], read[1], read[2], read[3]),
        (red.r, red.g, red.b, 255),
        "what rui drew is what the DOM holds"
    );
}

/// A browser hands events to callbacks and never to a caller that asked for
/// them, so the backend must have somewhere to put them: opening the page's
/// surface registers those listeners and leaves an empty queue behind.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm_backend_event_listeners() {
    surface_on_the_page(200, 100);

    assert_eq!(
        rui_native::shell::listen().expect("the listeners should attach"),
        Vec::new(),
        "a page nobody has touched has caught nothing"
    );
}
