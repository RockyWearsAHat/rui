//! Linux panel window stub (not yet implemented).
//!
//! This is a placeholder for a future Linux implementation.
//! Panel windows on Linux would use X11 or Wayland APIs to create a borderless,
//! non-activating window with the correct window type and decorations.

use super::PanelOptions;
use crate::Error;

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
}
