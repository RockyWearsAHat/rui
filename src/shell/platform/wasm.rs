//! The backend for a browser.
//!
//! The page is the window: there is nothing to open, only a `<canvas
//! id="surface">` to find, and presenting a frame is `putImageData` onto its
//! 2D context. That is the whole of the difference — the canvas handed to
//! [`present`](Backend::present) was drawn by the same `Surface::draw` that
//! feeds AppKit and Win32, so the picture is identical by construction.
//!
//! No `unsafe`: the platform here is JavaScript rather than C, and every call
//! into it goes through `wasm-bindgen`, which is safe on both sides.

use crate::geom::Point;
use crate::input::Modifiers;
use crate::shell::event_mapping;
use crate::shell::pixel_conversion::convert_pixels_to_rgba;
use crate::theme::Appearance;
use crate::{Canvas, Event};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{Clamped, JsCast};

use crate::shell::{Backend, Error, WindowOptions};

/// The id of the element a rui page is drawn into.
const SURFACE: &str = "surface";

/// What the page's listeners have caught since the last [`Backend::pump`].
///
/// Shared rather than owned, because a browser delivers an event to a callback
/// registered long before it and never to a caller asking for one: the queue is
/// the only place the two sides meet, so both have to be able to reach it.
type EventQueue = Rc<RefCell<Vec<Event>>>;

/// A registered DOM listener, which is to say a Rust closure JavaScript holds.
///
/// Named because it is passed about as a value rather than only called: it is
/// what [`listeners`] hands back and what [`Window`] keeps alive.
type Listener = Closure<dyn FnMut(web_sys::Event)>;

/// Handler function type: takes an event and the surface it landed on, and
/// returns an `Event` if it is one the toolkit has a use for.
///
/// The surface is passed because a pointer's position arrives measured from the
/// corner of the viewport, and everything below here only ever thinks in units
/// measured from the corner of its own canvas.
type EventHandler = fn(&web_sys::Event, &web_sys::HtmlCanvasElement) -> Option<Event>;

/// Where on the interface a pointer event landed, in the units it draws in.
///
/// Two corrections, and they are separate things. The canvas sits somewhere in
/// a page, so its own corner comes off first. Then the element's drawing buffer
/// need not be the size the page gives the element — a display with a scale
/// factor is exactly the case where it is not — so the offset is carried into
/// the buffer's own scale and back out of the interface's, which leaves the
/// logical position every other backend reports.
fn canvas_point(surface: &web_sys::HtmlCanvasElement, event: &web_sys::MouseEvent) -> Point {
    let rect = surface.get_bounding_client_rect();
    let scale = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
    let across = |offset: f64, shown: f64, buffer: f64| {
        if shown > 0.0 && scale > 0.0 {
            (offset * buffer / shown / scale) as f32
        } else {
            offset as f32
        }
    };
    Point::new(
        across(
            event.client_x() as f64 - rect.left(),
            rect.width(),
            surface.width() as f64,
        ),
        across(
            event.client_y() as f64 - rect.top(),
            rect.height(),
            surface.height() as f64,
        ),
    )
}

/// Handles a simple pointer down event: extracts button and position.
fn handle_pointer_down(
    event: &web_sys::Event,
    surface: &web_sys::HtmlCanvasElement,
) -> Option<Event> {
    event
        .dyn_ref::<web_sys::MouseEvent>()
        .and_then(|mouse_event| {
            let button = event_mapping::map_pointer_button(mouse_event.button() as u16)?;
            let position = canvas_point(surface, mouse_event);
            Some(Event::PointerDown { position, button })
        })
}

/// Handles a simple pointer up event: extracts button and position.
fn handle_pointer_up(
    event: &web_sys::Event,
    surface: &web_sys::HtmlCanvasElement,
) -> Option<Event> {
    event
        .dyn_ref::<web_sys::MouseEvent>()
        .and_then(|mouse_event| {
            let button = event_mapping::map_pointer_button(mouse_event.button() as u16)?;
            let position = canvas_point(surface, mouse_event);
            Some(Event::PointerUp { position, button })
        })
}

/// Handles the pointer moving: where it now is, in the interface's own units.
fn handle_pointer_moved(
    event: &web_sys::Event,
    surface: &web_sys::HtmlCanvasElement,
) -> Option<Event> {
    event
        .dyn_ref::<web_sys::MouseEvent>()
        .map(|mouse_event| Event::PointerMoved(canvas_point(surface, mouse_event)))
}

/// Handles a keyboard down event: extracts key and modifiers.
fn handle_key_down(event: &web_sys::Event, _surface: &web_sys::HtmlCanvasElement) -> Option<Event> {
    event
        .dyn_ref::<web_sys::KeyboardEvent>()
        .and_then(|keyboard_event| {
            let key = event_mapping::map_keyboard_code_to_key(&keyboard_event.code())?;
            let modifiers = extract_modifiers(keyboard_event);
            Some(Event::KeyDown { key, modifiers })
        })
}

/// Handles a keyboard up event: extracts key and modifiers.
fn handle_key_up(event: &web_sys::Event, _surface: &web_sys::HtmlCanvasElement) -> Option<Event> {
    event
        .dyn_ref::<web_sys::KeyboardEvent>()
        .and_then(|keyboard_event| {
            let key = event_mapping::map_keyboard_code_to_key(&keyboard_event.code())?;
            let modifiers = extract_modifiers(keyboard_event);
            Some(Event::KeyUp { key, modifiers })
        })
}

/// Handles a text input event: extracts and filters text data.
fn handle_text_input(
    event: &web_sys::Event,
    _surface: &web_sys::HtmlCanvasElement,
) -> Option<Event> {
    event
        .dyn_ref::<web_sys::InputEvent>()
        .and_then(|input_event| {
            let data = input_event.data()?;
            let filtered = event_mapping::filter_text_input_data(&data);
            if !filtered.is_empty() {
                Some(Event::Text(filtered))
            } else {
                None
            }
        })
}

/// Handles a scroll wheel event: extracts and normalizes delta values.
fn handle_scroll(event: &web_sys::Event, _surface: &web_sys::HtmlCanvasElement) -> Option<Event> {
    event
        .dyn_ref::<web_sys::WheelEvent>()
        .and_then(|wheel_event| {
            let (x, y) = event_mapping::normalize_wheel_delta(
                wheel_event.delta_x(),
                wheel_event.delta_y(),
                wheel_event.delta_mode(),
            );
            Some(Event::Scrolled { x, y })
        })
}

/// Which simple DOM events (no data extraction) the page is listened to for.
///
/// These events map directly to rui Events without needing to extract data.
const SIMPLE_EVENTS: [(&str, Event); 1] = [("pointerleave", Event::PointerLeft)];

/// Table of events that require data extraction and their handlers.
const EXTRACTED_EVENTS: &[(&str, EventHandler)] = &[
    ("mousedown", handle_pointer_down),
    ("mouseup", handle_pointer_up),
    ("mousemove", handle_pointer_moved),
    ("keydown", handle_key_down),
    ("keyup", handle_key_up),
    ("textinput", handle_text_input),
    ("wheel", handle_scroll),
];

/// The page's canvas, and the context that writes pixels to it.
pub(crate) struct Window {
    /// Kept alongside the context because the element, not the context, is
    /// what knows the size the page has given the surface.
    surface: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    event_queue: EventQueue,
    /// The registered listeners, held for no reason but to keep them alive:
    /// dropping a `Closure` unregisters the JavaScript function behind it, so a
    /// window that let go of these would stop hearing about the page.
    _listeners: Vec<Listener>,
}

/// Turns a JavaScript exception into the error the shell speaks.
fn platform(what: &str) -> impl FnOnce(wasm_bindgen::JsValue) -> Error + '_ {
    move |error| Error::Platform(format!("{what}: {error:?}"))
}

/// Extracts modifier state from a KeyboardEvent.
fn extract_modifiers(event: &web_sys::KeyboardEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        control: event.ctrl_key(),
        alt: event.alt_key(),
        command: event.meta_key(),
    }
}

/// Puts listeners on `surface` for pointer and keyboard events, each pushing to `queue`.
///
/// Returns them rather than forgetting them: a listener should last exactly as
/// long as the window it is filling the queue for, and one that outlives it is
/// a callback writing into a page that has moved on.
fn listeners(
    surface: &web_sys::HtmlCanvasElement,
    queue: &EventQueue,
) -> Result<Vec<Listener>, Error> {
    let mut all_listeners = Vec::new();

    // Register simple events that map directly without data extraction
    for (name, event) in &SIMPLE_EVENTS {
        let queue = Rc::clone(queue);
        let event = event.clone();
        let listener = Listener::new(move |_: web_sys::Event| {
            queue.borrow_mut().push(event.clone());
        });
        surface
            .add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
            .map_err(platform(name))?;
        all_listeners.push(listener);
    }

    // Register events that require data extraction from handler table
    for (name, handler) in EXTRACTED_EVENTS {
        let queue = Rc::clone(queue);
        let element = surface.clone();
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Some(rui_event) = handler(&event, &element) {
                queue.borrow_mut().push(rui_event);
            }
        });
        surface
            .add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
            .map_err(platform(name))?;
        all_listeners.push(listener);
    }

    Ok(all_listeners)
}

impl Backend for Window {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        // The options describe a window to open, and there is none: a page has
        // already decided how big its canvas is and what its title bar says.
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| Error::Platform("no document; is this running in a browser?".into()))?;
        let surface: web_sys::HtmlCanvasElement = document
            .get_element_by_id(SURFACE)
            .ok_or_else(|| Error::Platform(format!("the page has no <canvas id=\"{SURFACE}\">")))?
            .dyn_into()
            .map_err(|_| Error::Platform(format!("#{SURFACE} is not a <canvas>")))?;
        let context = surface
            .get_context("2d")
            .map_err(platform("the 2d context"))?
            .ok_or_else(|| Error::Platform("the canvas gave no 2d context".into()))?
            .dyn_into()
            .map_err(|_| Error::Platform("the context is not a 2d context".into()))?;
        let event_queue = EventQueue::default();
        let _listeners = listeners(&surface, &event_queue)?;
        Ok(Self {
            surface,
            context,
            event_queue,
            _listeners,
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Nothing is waited for: a page's one thread may never block, so the
        // timeout is a promise to come back rather than a promise to sleep.
        //
        // What the listeners caught is not all handed over at once, though. A
        // frame folds the events it is given into a single `Input`, so a second
        // press and release in the same batch overwrites the first and that
        // click is silently lost. A native loop never risks it, because it
        // wakes on each event; a frame loop does, because the queue keeps
        // filling between frames and a hidden tab is given no frames at all. So
        // a batch ends at the release that completes a click and the rest waits
        // — the next frame is a sixtieth of a second away, and asks again.
        let mut queued = self.event_queue.borrow_mut();
        let cut = queued
            .iter()
            .position(|event| matches!(event, Event::PointerUp { .. }))
            .map_or(queued.len(), |release| release + 1);
        events.extend(queued.drain(..cut));
        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        let scale = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio() as f32);
        (self.surface.width(), self.surface.height(), scale)
    }

    fn appearance(&self) -> Appearance {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(query)) = window.match_media("(prefers-color-scheme: dark)") {
                if query.matches() {
                    return Appearance::Dark;
                }
            }
        }
        Appearance::Light
    }

    fn present(&self, canvas: &Canvas) -> Result<(), Error> {
        if canvas.width() == 0 || canvas.height() == 0 {
            return Ok(());
        }

        let rgba = convert_pixels_to_rgba(canvas.pixels());

        let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&rgba),
            canvas.width(),
            canvas.height(),
        )
        .map_err(platform("the frame could not be wrapped as ImageData"))?;
        self.context
            .put_image_data(&image, 0.0, 0.0)
            .map_err(platform("the frame could not be put on the canvas"))
    }

    fn is_open(&self) -> bool {
        // A page's canvas does not close; the tab closes around it.
        true
    }
}
