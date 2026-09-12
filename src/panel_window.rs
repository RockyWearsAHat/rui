//! Panel window management for tray icon dropdowns.
//!
//! This module provides a cross-platform API for creating and managing
//! borderless panel windows that float above all other windows, typically
//! used as dropdown panels anchored to a tray icon.
//!
//! A panel window is:
//! - Borderless and without title bar
//! - Non-activating (doesn't steal focus or foreground the app)
//! - Floating (appears above regular windows)
//! - Dismissible (via Escape key or click outside)
//! - Able to display custom rendered content
//!
//! # Dismissal
//!
//! Panel windows can be dismissed in two ways:
//! 1. **Escape key**: When the panel has keyboard focus
//! 2. **Click-away**: When the user clicks outside the panel bounds
//!
//! You can register a dismissal callback that fires when either event occurs.
//! The callback will be invoked on the main thread.
//!
//! # Usage
//!
//! ```ignore
//! use rui::panel_window::{PanelWindow, PanelOptions};
//!
//! let options = PanelOptions::new(100.0, 200.0, 300.0, 400.0);
//! let panel = PanelWindow::new(options)?;
//! panel.show()?;
//!
//! // Register dismissal handler
//! panel.on_dismiss(|| {
//!     println!("Panel dismissed!");
//! })?;
//!
//! // Update position, hide, etc.
//! panel.set_position(150.0, 250.0)?;
//! drop(panel);  // Closes and cleans up the window
//! ```

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
use macos::PanelWindowInner;

#[cfg(target_os = "windows")]
use windows::PanelWindowInner;

#[cfg(target_os = "linux")]
use linux::PanelWindowInner;

#[cfg(target_os = "macos")]
pub use macos::Widget;

#[cfg(target_os = "windows")]
pub use windows::Widget;

#[cfg(target_os = "linux")]
pub use linux::Widget;

use crate::Error;

/// Configuration for creating a panel window.
///
/// Specifies the initial position and size of the panel in screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct PanelOptions {
    /// X coordinate of the panel's top-left corner in screen space.
    pub x: f64,
    /// Y coordinate of the panel's top-left corner in screen space.
    pub y: f64,
    /// Width of the panel in logical points.
    pub width: f64,
    /// Height of the panel in logical points.
    pub height: f64,
}

impl PanelOptions {
    /// Create a new PanelOptions with the given position and size.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        PanelOptions {
            x,
            y,
            width,
            height,
        }
    }
}

/// A borderless, non-activating floating window suitable for dropdown panels.
///
/// A panel window is used for temporary UI that floats above other windows,
/// such as a dropdown anchored to a tray icon. It does not steal focus
/// or activate the application.
///
/// The window is created hidden; call `show()` to display it. When dropped,
/// the window is closed automatically.
///
/// The panel supports two dismissal mechanisms:
/// - **Escape key**: When the user presses Escape, the panel closes
/// - **Click-away**: When the user clicks outside the panel bounds, it closes
///
/// These dismissals are automatic; you can optionally register a callback
/// via `on_dismiss()` to be notified when the panel closes.
pub struct PanelWindow {
    inner: PanelWindowInner,
}

impl PanelWindow {
    /// Create a new panel window with the given options.
    ///
    /// The window is created in a hidden state. Call `show()` to display it.
    ///
    /// # Arguments
    /// - `options`: Configuration specifying position and size in screen coordinates.
    ///
    /// # Errors
    /// Returns an error if the panel window cannot be created (e.g., unsupported platform).
    pub fn new(options: PanelOptions) -> Result<Self, Error> {
        let inner = PanelWindowInner::new(options)?;
        Ok(PanelWindow { inner })
    }

    /// Show the panel window.
    ///
    /// Makes the window visible at its current position. Does not activate
    /// the window or foreground the application.
    pub fn show(&self) -> Result<(), Error> {
        self.inner.show()
    }

    /// Hide the panel window.
    ///
    /// Makes the window invisible. The window remains in memory and can be
    /// shown again with `show()`.
    pub fn hide(&self) -> Result<(), Error> {
        self.inner.hide()
    }

    /// Move the panel window to a new position.
    ///
    /// # Arguments
    /// - `x`: New X coordinate in screen space.
    /// - `y`: New Y coordinate in screen space.
    pub fn set_position(&self, x: f64, y: f64) -> Result<(), Error> {
        self.inner.set_position(x, y)
    }

    /// Resize the panel window.
    ///
    /// # Arguments
    /// - `width`: New width in logical points.
    /// - `height`: New height in logical points.
    pub fn set_size(&self, width: f64, height: f64) -> Result<(), Error> {
        self.inner.set_size(width, height)
    }

    /// Check if the panel window is currently visible.
    pub fn is_visible(&self) -> Result<bool, Error> {
        self.inner.is_visible()
    }

    /// Register a callback to be invoked when the panel is dismissed.
    ///
    /// The callback will fire when:
    /// - The user presses the Escape key while the panel has focus
    /// - The user clicks outside the panel bounds
    ///
    /// The callback is invoked on the main thread. You can use this to
    /// update your application state, hide the panel, or clean up resources.
    ///
    /// # Arguments
    /// - `callback`: A closure that takes no arguments and returns `()`.
    ///   The closure is called when dismissal is triggered.
    ///
    /// # Notes
    /// - Only one dismissal callback is active at a time; calling this
    ///   method replaces any previous callback.
    /// - The callback does NOT automatically hide the panel; you must
    ///   call `hide()` yourself if desired.
    pub fn on_dismiss<F>(&self, callback: F) -> Result<(), Error>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.on_dismiss(Box::new(callback))
    }

    /// Adds a non-interactive line of text to the panel, in its own
    /// coordinate space (origin at the panel's bottom-left, in points).
    pub fn add_label(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text: &str,
    ) -> Result<Widget, Error> {
        self.inner.add_label(x, y, width, height, text)
    }

    /// Adds a clickable button. `tag` is the value passed back to
    /// [`Self::on_action`] when this button is pressed, so the caller can
    /// tell its buttons apart without keeping its own id-to-widget map.
    pub fn add_button(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        title: &str,
        tag: i64,
    ) -> Result<Widget, Error> {
        self.inner.add_button(x, y, width, height, title, tag)
    }

    /// Changes a label's or button's text.
    pub fn set_text(&self, widget: &Widget, text: &str) -> Result<(), Error> {
        self.inner.set_text(widget, text)
    }

    /// Enables or disables a button.
    pub fn set_enabled(&self, widget: &Widget, enabled: bool) -> Result<(), Error> {
        self.inner.set_enabled(widget, enabled)
    }

    /// Tints a label's text (`red, green, blue`, each `0.0..=1.0`). A no-op
    /// on a button, whose color follows the platform's control style.
    pub fn set_text_color(&self, widget: &Widget, rgb: (f32, f32, f32)) -> Result<(), Error> {
        self.inner.set_text_color(widget, rgb)
    }

    /// Sets the panel's own background color, corner radius, and a hairline
    /// border. `background`/`border` are sRGB `(r, g, b)`, each `0.0..=1.0`;
    /// `border_width` in points (`0.0` omits the border). Call once, after
    /// [`Self::new`] — this is the panel's own look, not any one control's.
    pub fn style(
        &self,
        background: (f32, f32, f32),
        corner_radius: f64,
        border: (f32, f32, f32),
        border_width: f64,
    ) -> Result<(), Error> {
        self.inner
            .style(background, corner_radius, border, border_width)
    }

    /// Tints a button's bezel (a no-op, harmlessly, on a label). `emphasis`
    /// draws it filled with a light title, matching a primary action;
    /// otherwise it is a quieter outline in that color, for a secondary one.
    pub fn set_button_tint(
        &self,
        widget: &Widget,
        rgb: (f32, f32, f32),
        emphasis: bool,
    ) -> Result<(), Error> {
        self.inner.set_button_tint(widget, rgb, emphasis)
    }

    /// Registers the callback every button on this panel reports its press
    /// to, carrying the tag it was created with. Replaces any callback
    /// registered earlier — there is one action sink per panel, not one per
    /// button.
    pub fn on_action<F>(&self, callback: F) -> Result<(), Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        self.inner.on_action(callback)
    }
}

impl Drop for PanelWindow {
    fn drop(&mut self) {
        let _ = self.inner.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_options_creation() {
        let opts = PanelOptions::new(100.0, 200.0, 300.0, 400.0);
        assert_eq!(opts.x, 100.0);
        assert_eq!(opts.y, 200.0);
        assert_eq!(opts.width, 300.0);
        assert_eq!(opts.height, 400.0);
    }

    #[test]
    fn panel_options_clone() {
        let opts1 = PanelOptions::new(50.0, 60.0, 70.0, 80.0);
        let opts2 = opts1;
        assert_eq!(opts1.x, opts2.x);
        assert_eq!(opts1.y, opts2.y);
        assert_eq!(opts1.width, opts2.width);
        assert_eq!(opts1.height, opts2.height);
    }
}
