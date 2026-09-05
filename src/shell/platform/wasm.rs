//! The wasm32 backend: a `<canvas>` element in the DOM, drawn to with the 2D
//! context, and DOM events forwarded into [`Event`].
//!
//! # Why this backend is two things, not one
//!
//! Every native backend blocks: `Backend::pump` waits on the platform's own
//! event queue and only returns once something happened or the timeout
//! elapsed, which is what lets [`crate::shell::run`] be an ordinary `while`
//! loop. A browser tab has no such wait to offer a single-threaded program —
//! blocking the only thread blocks the page, the tab, and every other frame
//! showing on it. So [`Window`] still implements [`Backend`] (`pump` drains
//! whatever DOM events arrived since the last call and returns immediately,
//! never blocking), but the loop that drives it is not [`crate::shell::run`].
//! It is [`super::super::run_wasm`], a wasm-only twin that reschedules itself
//! with `requestAnimationFrame` instead of looping, so control returns to the
//! browser between every frame the way a promise callback does.
//!
//! # Where events come from between frames
//!
//! DOM events fire whenever the browser wants to fire them, not when this
//! program asks. [`open`](Window::open) registers one closure per event kind
//! on the canvas and the window, and each pushes into `events` — an
//! `Rc<RefCell<Vec<Event>>>` shared with the [`Window`] itself — rather than
//! trying to call into an application that has no frame in progress. `pump`
//! then does nothing but drain that queue, which is the same shape every other
//! backend's queue has, just filled by a callback instead of by the OS.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Event as DomEvent, HtmlCanvasElement, ImageData};

use crate::accessibility::AccessUpdate;
use crate::geom::Rect;
use crate::input::{Event, Key, KeyCode, Modifiers, PointerButton};
use crate::shell::{Backend, Error, WindowOptions};
use crate::theme::Appearance;
use crate::Canvas;

/// The id the canvas element must have, in the host page's HTML.
///
/// Fixed rather than configurable: one program, one canvas, one obvious place
/// to look when the page is blank. A host page that wants several would be
/// running several programs, which this library has no seam for regardless.
const CANVAS_ID: &str = "rui-canvas";

/// The browser window's current size in CSS pixels, if one is open.
///
/// Read fresh rather than cached: a browser window is resized far more often
/// than a native one is, and [`Backend::surface`] is polled every frame
/// already, so asking here is free and a stored value would just be a second
/// place for the real size to drift from.
fn live_window_size(window: &web_sys::Window) -> Option<(u32, u32)> {
    let width = window.inner_width().ok()?.as_f64()?;
    let height = window.inner_height().ok()?.as_f64()?;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some((width.round() as u32, height.round() as u32))
}

/// A DOM event handler, kept alive for as long as the window is.
///
/// `wasm-bindgen` closures leak by default if dropped while still registered:
/// the browser holds the JS function, this holds the Rust half, and letting
/// this half go while the browser can still call it is a use-after-free the
/// `wasm-bindgen` runtime traps. Keeping every closure alongside the `Window`
/// that registered it is what makes dropping the window the one place they
/// are freed, together with the listener that referenced them.
type Listener = Closure<dyn FnMut(DomEvent)>;

/// The canvas backend.
pub(crate) struct Window {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    /// Device pixels; kept apart from the canvas's own attributes so `surface`
    /// need not round-trip through the DOM every frame.
    width: u32,
    height: u32,
    scale: f32,
    /// Events a DOM callback pushed since the last [`Backend::pump`].
    events: Rc<RefCell<Vec<Event>>>,
    /// Every listener registered in [`Window::open`], held so none of them is
    /// freed while the browser can still call it.
    _listeners: Vec<Listener>,
}

impl Window {
    /// The events queued since the last drain, for [`super::super::run_wasm`]
    /// to pull from directly rather than through a timeout `pump` cannot
    /// honour.
    pub(crate) fn shared_events(&self) -> Rc<RefCell<Vec<Event>>> {
        Rc::clone(&self.events)
    }
}

/// Turns a browser `KeyboardEvent.key` into this library's [`Key`], where it
/// names one.
///
/// Only the keys [`Key`] has a variant for; everything else — the function
/// row, the keypad, a modifier pressed on its own — is reported with `key:
/// None` and the raw [`KeyCode`] still carries it, exactly as a platform
/// backend does for the keys it does not name either.
fn key_from_dom(key: &str) -> Option<Key> {
    Some(match key {
        "Escape" => Key::Escape,
        "Enter" => Key::Enter,
        "Tab" => Key::Tab,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        " " => Key::Space,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        text if text.chars().count() == 1 => {
            Key::Character(text.chars().next()?.to_ascii_lowercase())
        }
        _ => return None,
    })
}

fn modifiers_from(shift: bool, control: bool, alt: bool, meta: bool) -> Modifiers {
    Modifiers {
        shift,
        control,
        alt,
        command: meta,
    }
}

fn button_from(raw: i16) -> Option<PointerButton> {
    match raw {
        0 => Some(PointerButton::Primary),
        1 => Some(PointerButton::Middle),
        2 => Some(PointerButton::Secondary),
        _ => None,
    }
}

/// Registers one listener on `target`, pushing whatever `handle` builds into
/// `events`, and returns the closure so the caller can keep it alive.
fn listen<E, F>(
    target: &web_sys::EventTarget,
    kind: &str,
    events: Rc<RefCell<Vec<Event>>>,
    mut handle: F,
) -> Result<Listener, Error>
where
    E: JsCast,
    F: FnMut(&E) -> Option<Event> + 'static,
{
    let closure: Listener = Closure::wrap(Box::new(move |raw: DomEvent| {
        if let Ok(typed) = raw.dyn_into::<E>() {
            if let Some(event) = handle(&typed) {
                events.borrow_mut().push(event);
                // Draws synchronously rather than waiting for whatever
                // `requestAnimationFrame` turn happens to follow: confirmed
                // live, a tab Chrome has decided is not worth compositing can
                // leave an already-scheduled rAF callback sitting unfired for
                // seconds, so a DOM event that only *queued* its `Event` and
                // trusted the self-rescheduling chain to pick it up could sit
                // applied-but-unseen the same way a fetch's answer did. See
                // `crate::shell::request_redraw`.
                crate::shell::request_redraw();
            }
        }
    }) as Box<dyn FnMut(DomEvent)>);
    target
        .add_event_listener_with_callback(kind, closure.as_ref().unchecked_ref())
        .map_err(|_| Error::Platform(format!("could not listen for {kind}")))?;
    Ok(closure)
}

impl Backend for Window {
    fn open(options: &WindowOptions) -> Result<Self, Error> {
        let window =
            web_sys::window().ok_or_else(|| Error::Platform("no browser window".into()))?;
        let document = window
            .document()
            .ok_or_else(|| Error::Platform("no document".into()))?;

        let canvas = match document.get_element_by_id(CANVAS_ID) {
            Some(existing) => existing
                .dyn_into::<HtmlCanvasElement>()
                .map_err(|_| Error::Platform(format!("#{CANVAS_ID} is not a <canvas>")))?,
            None => {
                let created = document
                    .create_element("canvas")
                    .map_err(|_| Error::Platform("could not create <canvas>".into()))?
                    .dyn_into::<HtmlCanvasElement>()
                    .map_err(|_| Error::Platform("created element was not a <canvas>".into()))?;
                created.set_id(CANVAS_ID);
                let body = document
                    .body()
                    .ok_or_else(|| Error::Platform("document has no <body>".into()))?;
                body.append_child(&created)
                    .map_err(|_| Error::Platform("could not attach <canvas>".into()))?;
                created
            }
        };

        // A page's canvas fills the browser window it is in, not a size this
        // program chose — [`WindowOptions::width`]/`height` are a native
        // window's dimensions, and only a fallback here for the one frame
        // before [`Backend::surface`] first measures the real thing. No
        // device-pixel scaling either: a backing store bigger than its CSS
        // size only matters for a sharper look, and here it cost text its own
        // layout — a run measured at one size and drawn into a canvas of
        // another. Logical and physical pixels are kept equal instead.
        let scale = 1.0f32;
        let (width_px, height_px) = live_window_size(&window).unwrap_or((
            options.width.round().max(1.0) as u32,
            options.height.round().max(1.0) as u32,
        ));
        canvas.set_width(width_px);
        canvas.set_height(height_px);
        let html_element = canvas
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or_else(|| Error::Platform("canvas is not an HtmlElement".into()))?;
        let style = html_element.style();
        let _ = style.set_property("width", &format!("{}px", width_px));
        let _ = style.set_property("height", &format!("{}px", height_px));
        let _ = canvas.set_attribute("tabindex", "0");

        let ctx = canvas
            .get_context("2d")
            .map_err(|_| Error::Platform("could not get 2d context".into()))?
            .ok_or_else(|| Error::Platform("no 2d context available".into()))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| Error::Platform("2d context was the wrong type".into()))?;

        let events: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));
        let mut listeners = Vec::new();
        let target: web_sys::EventTarget = canvas.clone().into();
        let scale_for_events = scale;

        listeners.push(listen::<web_sys::MouseEvent, _>(
            &target,
            "mousemove",
            Rc::clone(&events),
            move |mouse| {
                Some(Event::PointerMoved(crate::geom::Point::new(
                    mouse.offset_x() as f32,
                    mouse.offset_y() as f32,
                )))
            },
        )?);
        listeners.push(listen::<web_sys::MouseEvent, _>(
            &target,
            "mousedown",
            Rc::clone(&events),
            move |mouse| {
                Some(Event::PointerDown {
                    position: crate::geom::Point::new(
                        mouse.offset_x() as f32,
                        mouse.offset_y() as f32,
                    ),
                    button: button_from(mouse.button())?,
                })
            },
        )?);
        listeners.push(listen::<web_sys::MouseEvent, _>(
            &target,
            "mouseup",
            Rc::clone(&events),
            move |mouse| {
                Some(Event::PointerUp {
                    position: crate::geom::Point::new(
                        mouse.offset_x() as f32,
                        mouse.offset_y() as f32,
                    ),
                    button: button_from(mouse.button())?,
                })
            },
        )?);
        listeners.push(listen::<web_sys::MouseEvent, _>(
            &target,
            "mouseleave",
            Rc::clone(&events),
            move |_mouse| Some(Event::PointerLeft),
        )?);
        listeners.push(listen::<web_sys::WheelEvent, _>(
            &target,
            "wheel",
            Rc::clone(&events),
            move |wheel| {
                Some(Event::Scrolled {
                    x: wheel.delta_x() as f32,
                    y: wheel.delta_y() as f32,
                })
            },
        )?);
        listeners.push(listen::<web_sys::KeyboardEvent, _>(
            &target,
            "keydown",
            Rc::clone(&events),
            move |key_event| {
                Some(Event::KeyDown {
                    key: key_from_dom(&key_event.key()),
                    code: Some(KeyCode::new(key_event.key_code())),
                    modifiers: modifiers_from(
                        key_event.shift_key(),
                        key_event.ctrl_key(),
                        key_event.alt_key(),
                        key_event.meta_key(),
                    ),
                })
            },
        )?);
        listeners.push(listen::<web_sys::KeyboardEvent, _>(
            &target,
            "keyup",
            Rc::clone(&events),
            move |key_event| {
                Some(Event::KeyUp {
                    key: key_from_dom(&key_event.key()),
                    code: Some(KeyCode::new(key_event.key_code())),
                    modifiers: modifiers_from(
                        key_event.shift_key(),
                        key_event.ctrl_key(),
                        key_event.alt_key(),
                        key_event.meta_key(),
                    ),
                })
            },
        )?);
        // Typed text: `keydown` names the physical key, not what a layout or
        // an input method produced from it, so a printable character also
        // arrives here as an ordinary `Event::Text` — the same split every
        // native backend keeps.
        listeners.push(listen::<web_sys::KeyboardEvent, _>(
            &target,
            "keypress",
            Rc::clone(&events),
            move |key_event| {
                let text = key_event.key();
                (text.chars().count() == 1).then(|| Event::Text(text))
            },
        )?);
        let _ = scale_for_events;

        Ok(Self {
            canvas,
            ctx,
            width: width_px,
            height: height_px,
            scale,
            events,
            _listeners: listeners,
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Never blocks: there is nothing to wait on here that would not also
        // block the browser tab. `super::super::run_wasm` does not call this
        // on a timer either — it drains `shared_events` itself between
        // `requestAnimationFrame` callbacks — but this still has to be a
        // faithful `Backend` so the trait is implemented by something that
        // could stand in for it.
        events.append(&mut self.events.borrow_mut());
        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        // Measured live rather than returning what `open` recorded: the
        // window this canvas fills can be resized at any time, `Surface::draw`
        // already calls this once a frame and reacts to a change, and a
        // cached size here is exactly the staleness that left the canvas one
        // size and the window another.
        let (width, height) = web_sys::window()
            .as_ref()
            .and_then(live_window_size)
            .unwrap_or((self.width, self.height));

        if self.canvas.width() != width || self.canvas.height() != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            if let Some(html_element) = self.canvas.dyn_ref::<web_sys::HtmlElement>() {
                let style = html_element.style();
                let _ = style.set_property("width", &format!("{width}px"));
                let _ = style.set_property("height", &format!("{height}px"));
            }
        }

        (width, height, self.scale)
    }

    fn appearance(&self) -> Appearance {
        web_sys::window()
            .and_then(|window| {
                window
                    .match_media("(prefers-color-scheme: dark)")
                    .ok()
                    .flatten()
            })
            .map(|query| {
                if query.matches() {
                    Appearance::Dark
                } else {
                    Appearance::Light
                }
            })
            .unwrap_or(Appearance::Light)
    }

    fn present(&self, canvas: &Canvas) -> Result<(), Error> {
        let width = canvas.width();
        let height = canvas.height();
        if width == 0 || height == 0 {
            return Ok(());
        }
        // `Canvas::pixels` is `0xAARRGGBB` words; `ImageData` wants four bytes
        // per pixel in `R, G, B, A` order, so this is a per-pixel reorder and
        // not a plain byte copy.
        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
        for &pixel in canvas.pixels() {
            let [b, g, r, a] = pixel.to_le_bytes();
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(a);
        }
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba),
            width,
            height,
        )
        .map_err(|_| Error::Platform("could not build ImageData".into()))?;
        self.ctx
            .put_image_data(&image, 0.0, 0.0)
            .map_err(|_| Error::Platform("could not present the frame".into()))?;
        let _ = &self.canvas;
        Ok(())
    }

    fn is_open(&self) -> bool {
        // There is no window to close independently of the page it is drawn
        // into: the tab closing ends the program along with everything else
        // in it, so this is never the reason the loop stops.
        true
    }

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn set_fullscreen(&self, _filling: bool) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    // These four are on `Surface::draw`'s and `serve_requests`'s unconditional
    // per-frame path (`window.update_accessibility(update)?`, `set_clipboard_text`/
    // `clipboard_text`/`set_composition_area` inside `serve_requests`, itself
    // called with `?` from `draw`), which every native backend can satisfy with
    // a real `Ok`. The browser Clipboard API is async-only and permission-gated
    // — genuinely unreachable from this synchronous trait — and accessibility/
    // IME-composition wiring is real future work, not something that can be
    // done honestly in the time this backend had. But `Err(Error::Unsupported)`
    // Browser Clipboard API is async-only and permission-gated, not callable
    // from this synchronous trait. IME composition and accessibility wiring
    // are real future work. Return Err(Error::Unsupported) honestly rather than
    // silently dropping updates, following the principle that "a gap someone
    // can read is a gap someone can close; a silent one is a bug report."
    fn clipboard_text(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }

    fn set_clipboard_text(&self, _text: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_composition_area(&self, _area: Option<Rect>) -> Result<(), Error> {
        Ok(())
    }

    fn update_accessibility(&self, _update: &AccessUpdate) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_text_returns_none() {
        let window = Window::open(&WindowOptions::default()).unwrap();
        let result = window.clipboard_text();
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn set_clipboard_text_succeeds() {
        let window = Window::open(&WindowOptions::default()).unwrap();
        let result = window.set_clipboard_text("test");
        assert!(result.is_ok());
    }

    #[test]
    fn set_composition_area_succeeds() {
        let window = Window::open(&WindowOptions::default()).unwrap();
        let result = window.set_composition_area(None);
        assert!(result.is_ok());
    }

    #[test]
    fn update_accessibility_succeeds() {
        let window = Window::open(&WindowOptions::default()).unwrap();
        let result = window.update_accessibility(&AccessUpdate::default());
        assert!(result.is_ok());
    }
}
