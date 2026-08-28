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

pub mod embedded_fonts;
pub mod event_mapping;
pub mod fonts;
pub mod pixel_conversion;
mod platform;

use crate::app::App;
use crate::canvas::Canvas;
use crate::element::El;
use crate::font::FontError;
use crate::input::{Event, Input};
use crate::memory::Memory;
use crate::text::FontId;
use crate::theme::{Appearance, Theme};
use std::time::{Duration, Instant};

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

/// Opens a window and runs `app` in it until it is closed.
pub(crate) fn run<S>(
    options: WindowOptions,
    loaded: LoadedFonts,
    app: App<S>,
) -> Result<(), Error> {
    let mut window = platform::Window::open(&options)?;
    let (width, height, scale) = window.surface();

    let mut driver = FrameDriver::from_parts(app, loaded, width, height, scale);
    let mut events = Vec::new();

    while window.is_open() && driver.is_running() {
        events.clear();
        let wait = if driver.is_animating() {
            FRAME
        } else {
            driver.app_idle()
        };

        {
            // What the backend calls when the platform has taken the loop away.
            // It draws with no events of its own: a gesture the platform is
            // tracking is not one this program is being told about, and folding
            // the same click in twice would fire whatever it landed on twice.
            let mut redraw = |window: &platform::Window| {
                let (w, h, s) = window.surface();
                driver.resize(w, h, s);
                driver.set_appearance(window.appearance());
                driver.apply_events(vec![]);
                driver.step();
                if driver.pixels_changed() {
                    let _ = window.present(driver.canvas());
                }
            };
            window.pump(wait, &mut events, &mut redraw)?;
        }

        let (w, h, s) = window.surface();
        driver.resize(w, h, s);
        driver.set_appearance(window.appearance());
        driver.apply_events(std::mem::take(&mut events));
        driver.step();
        if driver.pixels_changed() {
            window.present(driver.canvas())?;
        }

        if driver.close_requested() {
            break;
        }
    }
    Ok(())
}

/// Drives a frame loop one step at a time, without a window or platform backend.
///
/// Used to verify that the frame rendering pipeline can be driven by events
/// without being tied to a blocking window loop. The same code path that a
/// [`crate::run`] loop uses is called here with synthetic input, no display, and full
/// control over timing.
pub struct FrameDriver<S> {
    app: App<S>,
    fonts: LoadedFonts,
    drawn: Canvas,
    presented: Canvas,
    memory: Memory,
    input: Input,
    drawn_at: Instant,
    drawn_changed: bool,
    appearance: Appearance,
}

impl<S: 'static> FrameDriver<S> {
    /// A frame driver showing `state`, described by `view`.
    ///
    /// Draws at 800×600 logical units, scale 1.0, with the test font.
    pub fn new(state: S, view: impl Fn(&S) -> El<S> + 'static) -> Self {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;
        const SCALE: f32 = 1.0;

        let app = App::new("test", state, view);
        let fonts = crate::testing::test_fonts();

        Self {
            app,
            drawn: Canvas::new(WIDTH, HEIGHT, SCALE),
            presented: Canvas::new(0, 0, SCALE),
            memory: Memory::new(),
            input: Input::new(),
            fonts,
            drawn_at: Instant::now(),
            drawn_changed: false,
            appearance: Appearance::Dark,
        }
    }
}

impl<S> FrameDriver<S> {
    /// Creates a frame driver from an existing app and loaded fonts.
    ///
    /// Used by the native event loop to drive frames with real window backend.
    pub fn from_parts(
        app: App<S>,
        fonts: LoadedFonts,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        Self {
            app,
            drawn: Canvas::new(width, height, scale),
            presented: Canvas::new(0, 0, scale),
            memory: Memory::new(),
            input: Input::new(),
            fonts,
            drawn_at: Instant::now(),
            drawn_changed: false,
            appearance: Appearance::Light,
        }
    }

    /// Applies events to the input state.
    ///
    /// Must be called before [`Self::step`] to have the events processed
    /// in the next frame.
    pub fn apply_events(&mut self, events: Vec<Event>) {
        for event in events {
            self.input.apply(event);
        }
    }

    /// Resizes the canvas to the given dimensions if they differ.
    ///
    /// Should be called before [`Self::step`] if the window size changes.
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) {
        if width != self.drawn.width()
            || height != self.drawn.height()
            || scale != self.drawn.scale()
        {
            self.drawn.resize(width, height, scale);
        }
    }

    /// Whether the app or memory indicate animation is in progress.
    pub fn is_animating(&self) -> bool {
        self.memory.is_animating()
    }

    /// Sets the appearance (light/dark mode) for the next frame.
    pub fn set_appearance(&mut self, appearance: Appearance) {
        self.appearance = appearance;
    }

    /// Draws one frame with the accumulated input and animations.
    ///
    /// Applies all changes from event handlers to the state, and presents
    /// the frame to the canvas. The canvas is updated whether or not pixels
    /// changed — use [`Self::has_drawn`] to check if this step's output
    /// differs from the last one.
    pub fn step(&mut self) {
        let now = Instant::now();
        self.memory
            .begin_frame(now.saturating_duration_since(self.drawn_at));
        self.drawn_at = now;

        self.input.begin_frame();

        self.fonts.fonts.set_scale(self.drawn.scale());
        let theme = Theme::new(self.appearance, self.fonts.ui_font, self.fonts.mono_font);
        self.drawn
            .clear_vertical(theme.palette.background, theme.palette.background_deep);

        self.app.frame(
            &mut self.drawn,
            &self.fonts.fonts,
            &self.input,
            &mut self.memory,
            &theme,
        );
        self.memory.end_frame(&self.input);

        self.drawn_changed = true;
    }

    /// The application's current state.
    pub fn state(&self) -> &S {
        self.app.state()
    }

    /// Whether a frame was drawn since this was last called.
    pub fn has_drawn(&mut self) -> bool {
        let changed = self.drawn_changed;
        self.drawn_changed = false;
        changed
    }

    /// The canvas that was just drawn.
    pub fn canvas(&self) -> &Canvas {
        &self.drawn
    }

    /// Checks if pixels changed and swaps the drawn and presented buffers.
    ///
    /// Returns true if the frame differs from the previously presented one,
    /// indicating it should be sent to the display.
    pub fn pixels_changed(&mut self) -> bool {
        if self.drawn.pixels() != self.presented.pixels() {
            std::mem::swap(&mut self.drawn, &mut self.presented);
            true
        } else {
            false
        }
    }

    /// The UI font ID.
    pub fn ui_font(&self) -> FontId {
        self.fonts.ui_font
    }

    /// The monospace font ID.
    pub fn mono_font(&self) -> FontId {
        self.fonts.mono_font
    }

    /// Whether the application is still running.
    pub fn is_running(&self) -> bool {
        self.app.is_running()
    }

    /// The application's idle timeout.
    pub fn app_idle(&self) -> Duration {
        self.app.idle()
    }

    /// Whether close was requested.
    pub fn close_requested(&self) -> bool {
        self.input.close_requested()
    }
}
