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

/// Which simple DOM events (no data extraction) the page is listened to for.
///
/// These events map directly to rui Events without needing to extract data.
const SIMPLE_EVENTS: [(&str, Event); 1] = [("pointerleave", Event::PointerLeft)];

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

    // Simple events that map directly without data extraction
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

    // Pointer down events: extract button and position
    {
        let queue = Rc::clone(queue);
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Ok(mouse_event) = event.dyn_into::<web_sys::MouseEvent>() {
                let button = mouse_event.button() as u16;
                if let Some(button) = event_mapping::map_pointer_button(button) {
                    let position =
                        Point::new(mouse_event.client_x() as f32, mouse_event.client_y() as f32);
                    queue
                        .borrow_mut()
                        .push(Event::PointerDown { position, button });
                }
            }
        });
        surface
            .add_event_listener_with_callback("mousedown", listener.as_ref().unchecked_ref())
            .map_err(platform("mousedown"))?;
        all_listeners.push(listener);
    }

    // Pointer up events: extract button and position
    {
        let queue = Rc::clone(queue);
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Ok(mouse_event) = event.dyn_into::<web_sys::MouseEvent>() {
                let button = mouse_event.button() as u16;
                if let Some(button) = event_mapping::map_pointer_button(button) {
                    let position =
                        Point::new(mouse_event.client_x() as f32, mouse_event.client_y() as f32);
                    queue
                        .borrow_mut()
                        .push(Event::PointerUp { position, button });
                }
            }
        });
        surface
            .add_event_listener_with_callback("mouseup", listener.as_ref().unchecked_ref())
            .map_err(platform("mouseup"))?;
        all_listeners.push(listener);
    }

    // Keyboard down events: extract key and modifiers
    {
        let queue = Rc::clone(queue);
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Ok(keyboard_event) = event.dyn_into::<web_sys::KeyboardEvent>() {
                if let Some(key) = event_mapping::map_keyboard_code_to_key(&keyboard_event.code()) {
                    let modifiers = extract_modifiers(&keyboard_event);
                    queue.borrow_mut().push(Event::KeyDown { key, modifiers });
                }
            }
        });
        surface
            .add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
            .map_err(platform("keydown"))?;
        all_listeners.push(listener);
    }

    // Keyboard up events: extract key and modifiers
    {
        let queue = Rc::clone(queue);
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Ok(keyboard_event) = event.dyn_into::<web_sys::KeyboardEvent>() {
                if let Some(key) = event_mapping::map_keyboard_code_to_key(&keyboard_event.code()) {
                    let modifiers = extract_modifiers(&keyboard_event);
                    queue.borrow_mut().push(Event::KeyUp { key, modifiers });
                }
            }
        });
        surface
            .add_event_listener_with_callback("keyup", listener.as_ref().unchecked_ref())
            .map_err(platform("keyup"))?;
        all_listeners.push(listener);
    }

    // Text input events: extract data field and filter control characters
    {
        let queue = Rc::clone(queue);
        let listener = Listener::new(move |event: web_sys::Event| {
            if let Ok(input_event) = event.dyn_into::<web_sys::InputEvent>() {
                if let Some(data) = input_event.data() {
                    let filtered: String = data
                        .chars()
                        .filter(|c| {
                            let code = *c as u32;
                            code >= 32 || *c == '\t' || *c == '\n'
                        })
                        .collect();
                    if !filtered.is_empty() {
                        queue.borrow_mut().push(Event::Text(filtered));
                    }
                }
            }
        });
        surface
            .add_event_listener_with_callback("textinput", listener.as_ref().unchecked_ref())
            .map_err(platform("textinput"))?;
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
        // timeout is a promise to come back rather than a promise to sleep, and
        // whatever the listeners caught before now is the whole of this frame's
        // input.
        events.append(&mut self.event_queue.borrow_mut());
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

        // `Canvas` holds one packed `u32` per pixel, which little-endian memory
        // reads back as b, g, r, a. `ImageData` wants r, g, b, a, so the bytes
        // are unpacked from the word rather than read out of it — the same
        // arithmetic on either endianness, and the only per-pixel work the
        // browser backend does that the native ones do not.
        let mut rgba = Vec::with_capacity(canvas.pixels().len() * 4);
        for &pixel in canvas.pixels() {
            rgba.push((pixel >> 16) as u8);
            rgba.push((pixel >> 8) as u8);
            rgba.push(pixel as u8);
            rgba.push((pixel >> 24) as u8);
        }

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
