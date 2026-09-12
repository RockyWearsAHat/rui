//! Windows panel window stub (not yet implemented).
//!
//! This is a placeholder for a future Windows implementation.
//! Panel windows on Windows would use Win32 API to create a borderless,
//! non-activating window with the correct window class and styles.

use super::PanelOptions;
use crate::Error;

/// Windows-specific widget handle (stub).
pub struct Widget;

/// Windows-specific panel window (stub).
pub struct PanelWindowInner;

impl PanelWindowInner {
    /// Create a new panel window (not yet implemented on Windows).
    pub fn new(_options: PanelOptions) -> Result<Self, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Show the panel window.
    pub fn show(&self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Hide the panel window.
    pub fn hide(&self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Move the panel to a new position.
    pub fn set_position(&self, _x: f64, _y: f64) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Resize the panel window.
    pub fn set_size(&self, _width: f64, _height: f64) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> Result<bool, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Close and release the panel.
    pub fn close(&mut self) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Register a dismissal callback (not yet implemented on Windows).
    pub fn on_dismiss(&self, _callback: Box<dyn Fn() + Send + Sync>) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Add a label (not yet implemented on Windows).
    pub fn add_label(
        &self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _text: &str,
    ) -> Result<Widget, Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Add a button (not yet implemented on Windows).
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
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Change a widget's text (not yet implemented on Windows).
    pub fn set_text(&self, _widget: &Widget, _text: &str) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Enable or disable a widget (not yet implemented on Windows).
    pub fn set_enabled(&self, _widget: &Widget, _enabled: bool) -> Result<(), Error> {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }

    /// Register the button-action callback (not yet implemented on Windows).
    pub fn on_action<F>(&self, _callback: F) -> Result<(), Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        Err(Error::Platform(
            "Panel windows are not yet implemented on Windows".into(),
        ))
    }
}
