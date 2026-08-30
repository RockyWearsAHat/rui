//! The window: the only part of the library that talks to an operating system.
//!
//! Deliberately the smallest part. A backend does five things — open a window,
//! say how big it is and whether the desktop is light or dark, hand over the
//! events it received, and copy a buffer of pixels onto the screen. Everything
//! else is decided above it, identically everywhere, which is why porting to a
//! new platform is a few hundred lines against that surface and why a defect in
//! a widget can never be a platform defect.
//!
//! # The loop
//!
//! Wait for input, fold it into the frame's [`Input`], draw the whole interface,
//! and present it if it came out different from the last one.
//!
//! There is no partial redraw and no dirty tracking. A system that works out
//! *which region* to repaint is a system that can work it out wrongly, and the
//! symptom is a stale pixel still showing a service as running after it has
//! died. Comparing the finished frame with the previous one has the same effect
//! on cost with none of that risk: it cannot conclude that something changed
//! when it did not, or the reverse. This matters because sending a frame to the
//! compositor costs several times what drawing it does, and most interfaces
//! spend nearly all of their life displaying the same picture.
//!
//! # Two speeds, chosen by whether anything is moving
//!
//! While anything is mid-animation — a hover fading in, a list settling after a
//! click — the loop comes back within `FRAME`. Once everything has settled it
//! goes back to waiting [`App::idle_timeout`], and a window nobody is touching
//! costs what it always did. Nothing here knows what is animating or why: the
//! interface answers [`Memory::is_animating`] and the loop believes it.
//!
//! # When the platform takes the loop away
//!
//! A window system may run a loop of its own that does not return until a
//! gesture ends. macOS resizes a window that way, and a program that only draws
//! from its own loop draws nothing for the whole drag — the compositor stretches
//! the last frame to each new size, so the window smears. So drawing a frame is
//! not something only this loop can do: `Backend::pump` is handed a way to
//! draw one, for a backend to call when the platform has taken over.

pub mod clock;
pub mod embedded_fonts;
pub mod event_mapping;
pub mod fonts;
pub mod pixel_conversion;
mod platform;

use crate::app::App;
use crate::canvas::Canvas;
use crate::font::FontError;
use crate::input::{Event, Input};
use crate::memory::Memory;
use crate::text::{FontId, Fonts};
use crate::theme::{Appearance, Theme};
use clock::Moment;
use std::time::Duration;

pub use fonts::{load_system_fonts, LoadedFonts};

/// How long the loop waits between frames while something is animating.
///
/// A hundred and twenty a second rather than sixty: the wait is an upper bound
/// on latency and not a frame rate, so asking to come back more often costs
/// nothing when nothing is moving, and halves the worst case when something is.
const FRAME: Duration = Duration::from_millis(8);

/// How a window should be opened.
#[derive(Debug, Clone)]
pub struct WindowOptions {
    /// The title bar's text.
    pub title: String,
    /// Initial width, in logical units.
    pub width: f32,
    /// Initial height, in logical units.
    pub height: f32,
    /// The smallest width the window may be dragged to.
    pub min_width: f32,
    /// The smallest height.
    pub min_height: f32,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "rui".into(),
            width: 960.0,
            height: 640.0,
            min_width: 420.0,
            min_height: 320.0,
        }
    }
}

/// Why a window could not be opened or drawn.
#[derive(Debug)]
pub enum Error {
    /// No usable font was found on this machine.
    NoFont {
        /// The files that were looked for.
        searched: Vec<String>,
    },
    /// A font file was found but could not be parsed.
    Font(FontError),
    /// A file could not be read.
    Io(std::io::Error),
    /// The windowing system refused, or is not there.
    Platform(String),
    /// This platform has no backend compiled in.
    Unsupported,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFont { searched } => {
                write!(
                    formatter,
                    "no usable font found; looked for {}",
                    searched.join(", ")
                )
            }
            Self::Font(error) => write!(formatter, "the font could not be read: {error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Platform(message) => write!(formatter, "{message}"),
            Self::Unsupported => write!(
                formatter,
                "this platform has no window backend; macOS, Windows, and X11 are supported"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FontError> for Error {
    fn from(error: FontError) -> Self {
        Self::Font(error)
    }
}

/// What a backend must be able to do.
///
/// Every method is about the platform rather than about the interface. Anything
/// a backend could decide for itself — what a click means, where an element is —
/// is decided above it.
trait Backend: Sized {
    /// Opens the window.
    fn open(options: &WindowOptions) -> Result<Self, Error>;

    /// Collects pending events, waiting up to `timeout` for the first.
    ///
    /// `redraw` draws and presents one frame immediately, for a backend to call
    /// when the platform has taken the loop away and will not give it back until
    /// a gesture ends. It takes `&Self` and not `&mut Self` because the backend
    /// is inside `pump` when it calls it.
    fn pump(
        &mut self,
        timeout: Duration,
        events: &mut Vec<Event>,
        redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error>;

    /// The drawable size in device pixels, and the display's scale factor.
    fn surface(&self) -> (u32, u32, f32);

    /// Whether the desktop is currently light or dark.
    fn appearance(&self) -> Appearance;

    /// Copies a frame onto the screen.
    fn present(&self, canvas: &Canvas) -> Result<(), Error>;

    /// Whether the window is still on screen.
    fn is_open(&self) -> bool;
}

/// Everything one frame is drawn from, apart from the window and the program.
///
/// Held together because a frame is drawn from two places — the loop below, and
/// the backend itself while the platform has taken the loop away.
struct Surface {
    /// The frame being drawn.
    drawn: Canvas,
    /// The frame on screen, so an identical one is not sent again.
    ///
    /// A second canvas rather than a saved copy of the first: presenting swaps
    /// the two, so recognising an unchanged frame costs one comparison and never
    /// a copy of the surface.
    presented: Canvas,
    input: Input,
    memory: Memory,
    /// When the previous frame was drawn, so animation advances by elapsed time
    /// rather than by a count of frames.
    drawn_at: Moment,
    ui_font: FontId,
    mono_font: FontId,
    /// A failure from a frame drawn inside the platform's own loop.
    ///
    /// There is nothing to return it to from in there, and dropping it would
    /// turn a window that can no longer present into one that silently freezes.
    failed: Option<Error>,
}

impl Surface {
    /// The surface a window starts with: sized to it, with nothing on screen.
    fn new<B: Backend>(window: &B, ui_font: FontId, mono_font: FontId) -> Self {
        let (width, height, scale) = window.surface();
        Self {
            drawn: Canvas::new(width, height, scale),
            // Deliberately empty rather than the surface's size: nothing has
            // been presented yet, and an empty canvas differs from every frame,
            // so the first one is sent instead of being mistaken for a repeat.
            presented: Canvas::new(0, 0, scale),
            input: Input::new(),
            memory: Memory::new(),
            drawn_at: Moment::now(),
            ui_font,
            mono_font,
            failed: None,
        }
    }

    /// Folds `events` in, draws the whole interface, and presents it if it came
    /// out different from what is already on screen.
    fn draw<B: Backend, S>(
        &mut self,
        window: &B,
        fonts: &mut Fonts,
        app: &mut App<S>,
        events: &mut Vec<Event>,
    ) -> Result<(), Error> {
        let now = Moment::now();
        self.memory.begin_frame(now.since(self.drawn_at));
        self.drawn_at = now;

        self.input.begin_frame();
        for event in events.drain(..) {
            self.input.apply(event);
        }

        let (width, height, scale) = window.surface();
        if width != self.drawn.width()
            || height != self.drawn.height()
            || scale != self.drawn.scale()
        {
            self.drawn.resize(width, height, scale);
        }
        fonts.set_scale(scale);

        let theme = Theme::new(window.appearance(), self.ui_font, self.mono_font);
        self.drawn
            .clear_vertical(theme.palette.background, theme.palette.background_deep);
        app.frame(
            &mut self.drawn,
            fonts,
            &self.input,
            &mut self.memory,
            &theme,
        );
        self.memory.end_frame(&self.input);

        if self.drawn.pixels() != self.presented.pixels() {
            window.present(&self.drawn)?;
            std::mem::swap(&mut self.drawn, &mut self.presented);
        }
        Ok(())
    }
}

/// Draws `canvas` into the page's `<canvas id="surface">`.
///
/// The browser's way in, until it has a loop of its own. [`run`] below waits on
/// the platform and does not return, which is exactly what a page's one thread
/// may never do — so a browser presents frames it drew itself, one at a time,
/// through here. The buffer that arrives is an ordinary [`Canvas`], drawn by
/// the same code that draws every native frame.
#[cfg(target_arch = "wasm32")]
pub fn present(canvas: &Canvas) -> Result<(), Error> {
    platform::Window::open(&WindowOptions::default())?.present(canvas)
}

/// Listens to the page's `<canvas id="surface">` and collects what it caught.
///
/// The other half of [`present`], and here for the same reason: a page that
/// drives its own frames needs the events that arrived before one as much as it
/// needs somewhere to put the pixels after it. The events that come back are
/// the ordinary [`Event`]s every backend produces, so what a browser did with a
/// gesture is indistinguishable, from here up, from what a desktop did with it.
#[cfg(target_arch = "wasm32")]
pub fn listen() -> Result<Vec<Event>, Error> {
    let mut events = Vec::new();
    platform::Window::open(&WindowOptions::default())?.pump(
        Duration::ZERO,
        &mut events,
        &mut |_| {},
    )?;
    Ok(events)
}

/// Returns the browser's current color scheme preference.
///
/// Queries the system via `window.matchMedia("(prefers-color-scheme: dark)")`
/// and returns `Appearance::Dark` if the system prefers dark mode, or
/// `Appearance::Light` otherwise.
#[cfg(target_arch = "wasm32")]
pub fn get_appearance() -> Appearance {
    platform::Window::open(&WindowOptions::default())
        .map(|w| w.appearance())
        .unwrap_or(Appearance::Light)
}

/// One turn of the loop: collect what arrived, draw it, present it if it came
/// out different from the frame already on screen.
///
/// The whole of a frame, and everything the two drivers below have in common.
/// They differ only in what asks for the next one — a wait on the platform, or
/// the browser calling back — so this is deliberately the entire body of both.
/// A frame drawn in a page and a frame drawn on a desktop are the same code,
/// and there is nowhere for the two to drift apart.
fn turn<S>(
    window: &mut platform::Window,
    surface: &mut Surface,
    fonts: &mut Fonts,
    app: &mut App<S>,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    events.clear();
    let wait = if surface.memory.is_animating() {
        FRAME
    } else {
        app.idle()
    };

    {
        // What the backend calls when the platform has taken the loop away.
        // It draws with no events of its own: a gesture the platform is
        // tracking is not one this program is being told about, and folding
        // the same click in twice would fire whatever it landed on twice.
        let mut redraw = |window: &platform::Window| {
            if let Err(error) = surface.draw(window, fonts, app, &mut Vec::new()) {
                surface.failed = Some(error);
            }
        };
        window.pump(wait, events, &mut redraw)?;
    }
    if let Some(error) = surface.failed.take() {
        return Err(error);
    }

    surface.draw(window, fonts, app, events)
}

/// Whether the loop should come back for another frame.
fn continues<S>(window: &platform::Window, surface: &Surface, app: &App<S>) -> bool {
    window.is_open() && app.is_running() && !surface.input.close_requested()
}

/// Opens a window and runs `app` in it until it is closed.
///
/// The desktop driver: this thread belongs to the interface from here until the
/// window goes away, which is what lets it wait — for input, or for the idle
/// timeout — instead of spinning.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run<S: 'static>(
    options: WindowOptions,
    loaded: LoadedFonts,
    mut app: App<S>,
) -> Result<(), Error> {
    let LoadedFonts {
        mut fonts,
        ui_font,
        mono_font,
    } = loaded;
    let mut window = platform::Window::open(&options)?;
    let mut surface = Surface::new(&window, ui_font, mono_font);
    let mut events = Vec::new();

    while continues(&window, &surface, &app) {
        turn(&mut window, &mut surface, &mut fonts, &mut app, &mut events)?;
    }
    Ok(())
}

/// Draws `app` into the page, one frame per repaint, and returns immediately.
///
/// The browser driver, and the one place where the loop above cannot be the
/// loop. A page has a single thread; it is the thread everything else on the
/// page also runs on; and a program that never gives it back gives back nothing
/// — no events, no timers, no repaint, and a tab the browser eventually offers
/// to kill. So there is no `while` here. [`turn`] goes to
/// `requestAnimationFrame` instead, and the browser calls it when it is about
/// to repaint. Only the driving differs: the frame that comes out is drawn by
/// the same code that draws every native one.
///
/// Returning before a single frame has been drawn is the honest answer rather
/// than a shortcut. What this call can fail at is finding the page's canvas and
/// asking for the first frame, and by the time it returns both have happened. A
/// frame that fails later has no caller left to return to — the stack it is on
/// belongs to the browser — so it is reported to the console and the loop stops
/// asking for frames, rather than being dropped on the floor and leaving a
/// surface that has quietly stopped repainting with nothing said.
///
/// Every repaint and no idle timeout, deliberately. `requestAnimationFrame` is
/// the browser's own answer to when drawing is worth doing, and it already
/// stops entirely for a tab nobody is looking at, which is the same bargain
/// `App::idle_timeout` strikes on a desktop. A frame that comes out identical
/// to the one on the canvas is still compared and still not presented, so the
/// expensive half of an idle frame is not paid either way.
#[cfg(target_arch = "wasm32")]
pub(crate) fn run<S: 'static>(
    options: WindowOptions,
    loaded: LoadedFonts,
    app: App<S>,
) -> Result<(), Error> {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;

    /// Everything the page's loop owns between one frame and the next.
    ///
    /// Held together in one cell because the callback owns all of it and the
    /// browser owns the callback: there is no stack frame left to keep any of
    /// it alive between repaints.
    struct Page<S> {
        window: platform::Window,
        surface: Surface,
        fonts: Fonts,
        app: App<S>,
        events: Vec<Event>,
    }

    let LoadedFonts {
        fonts,
        ui_font,
        mono_font,
    } = loaded;
    let window = platform::Window::open(&options)?;
    let surface = Surface::new(&window, ui_font, mono_font);
    let page = Rc::new(RefCell::new(Some(Page {
        window,
        surface,
        fonts,
        app,
        events: Vec::new(),
    })));

    // The one knot: the callback has to be able to ask for the frame after it,
    // which means holding the very thing it is. It is built empty and then
    // filled in with itself, and the cycle that makes is what keeps the loop
    // alive without anything on the page having to remember it.
    let next: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let first = Rc::clone(&next);

    *first.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut held = page.borrow_mut();
        let Some(open) = held.as_mut() else {
            return;
        };
        let Page {
            window,
            surface,
            fonts,
            app,
            events,
        } = open;

        let carry_on = match turn(window, surface, fonts, app, events) {
            Ok(()) => continues(window, surface, app),
            Err(error) => {
                report(&error);
                false
            }
        };

        if carry_on {
            if let Some(callback) = next.borrow().as_ref() {
                if let Err(error) = schedule(callback) {
                    report(&error);
                    *held = None;
                }
            }
        } else {
            // Nothing will ask for another frame, so let go of the window, the
            // fonts, and the state: everything the loop was holding is freed
            // here. The callback itself outlives it — dropping a closure from
            // inside the call the browser is making to it would free the frame
            // that call is standing on — but an empty cell is all that is left.
            *held = None;
        }
    }) as Box<dyn FnMut()>));

    // Bound rather than returned straight: the borrow has to be let go of
    // before `first` is, and a tail expression would hold it past that.
    let started = schedule(
        first
            .borrow()
            .as_ref()
            .expect("the callback was just put there"),
    );
    started
}

/// Asks the browser to call `callback` before its next repaint.
#[cfg(target_arch = "wasm32")]
fn schedule(callback: &wasm_bindgen::closure::Closure<dyn FnMut()>) -> Result<(), Error> {
    use wasm_bindgen::JsCast;

    web_sys::window()
        .ok_or_else(|| Error::Platform("no window to draw frames in".into()))?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map(|_id| ())
        .map_err(|error| Error::Platform(format!("requestAnimationFrame refused: {error:?}")))
}

/// Says what went wrong somewhere the browser's own tools will show it.
///
/// The end of the line for a frame that failed inside a callback: there is no
/// caller to hand an error back to, and a page that simply stopped repainting
/// with nothing said would be indistinguishable from one that finished.
#[cfg(target_arch = "wasm32")]
fn report(error: &Error) {
    web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("rui: {error}")));
}
