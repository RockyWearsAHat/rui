//! Linux panel window stub (not yet implemented).
//!
//! This is a placeholder for a future Linux implementation.
//! Panel windows on Linux would use X11 or Wayland APIs to create a borderless,
//! non-activating window with the correct window type and decorations.

use super::PanelOptions;
use crate::Error;

/// Linux-specific widget handle (stub).
pub struct Widget;

/// Linux-specific panel window (stub).
pub struct PanelWindowInner;

impl PanelWindowInner {
    /// Create a new panel window (not yet implemented on Linux).
    pub fn new(_options: PanelOptions) -> Result<Self, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Show the panel window.
    pub fn show(&self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Hide the panel window.
    pub fn hide(&self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Move the panel to a new position.
    pub fn set_position(&self, _x: f64, _y: f64) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Resize the panel window.
    pub fn set_size(&self, _width: f64, _height: f64) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> Result<bool, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Close and release the panel.
    pub fn close(&mut self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Register a dismissal callback (not yet implemented on Linux).
    pub fn on_dismiss(&self, _callback: Box<dyn Fn() + Send + Sync>) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Add a label (not yet implemented on Linux).
    pub fn add_label(
        &self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _text: &str,
    ) -> Result<Widget, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Add a button (not yet implemented on Linux).
    pub fn add_button(
        &self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _title: &str,
        _tag: i64,
    ) -> Result<Widget, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Change a widget's text (not yet implemented on Linux).
    pub fn set_text(&self, _widget: &Widget, _text: &str) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Enable or disable a widget (not yet implemented on Linux).
    pub fn set_enabled(&self, _widget: &Widget, _enabled: bool) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Tint a label's text (not yet implemented on Linux).
    pub fn set_text_color(&self, _widget: &Widget, _rgb: (f32, f32, f32)) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Set the panel's style (not yet implemented on Linux).
    pub fn style(
        &self,
        _background: (f32, f32, f32),
        _corner_radius: f64,
        _border: (f32, f32, f32),
        _border_width: f64,
    ) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Tint a button's bezel (not yet implemented on Linux).
    pub fn set_button_tint(
        &self,
        _widget: &Widget,
        _rgb: (f32, f32, f32),
        _emphasis: bool,
    ) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }

    /// Register the button-action callback (not yet implemented on Linux).
    pub fn on_action<F>(&self, _callback: F) -> Result<(), Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Linux".into(),
        ))
    }
}
