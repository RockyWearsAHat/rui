//! wasm32 stub for panel windows.
//!
//! Panel windows are an OS-level floating-window feature (tray dropdowns);
//! there is no equivalent in a browser tab. Every constructor returns a
//! platform-unsupported error so wasm32 consumers of `rui` compile cleanly
//! and can decide for themselves whether to skip panel-window features.

use super::PanelOptions;
use crate::Error;

fn unsupported() -> Error {
    Error::Platform("panel windows are not supported on wasm32 (browser targets)".into())
}

#[derive(Debug, Clone, Copy)]
pub struct Widget;

pub struct PanelWindowInner;

impl PanelWindowInner {
    pub fn new(_options: PanelOptions) -> Result<Self, Error> {
        Err(unsupported())
    }

    pub fn show(&self) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn hide(&self) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn set_position(&self, _x: f64, _y: f64) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn set_size(&self, _width: f64, _height: f64) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn is_visible(&self) -> Result<bool, Error> {
        Err(unsupported())
    }

    pub fn on_dismiss(&self, _callback: Box<dyn Fn() + Send + Sync>) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn add_label(
        &self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _text: &str,
    ) -> Result<Widget, Error> {
        Err(unsupported())
    }

    pub fn add_button(
        &self,
        _x: f64,
        _y: f64,
        _width: f64,
        _height: f64,
        _title: &str,
        _tag: i64,
    ) -> Result<Widget, Error> {
        Err(unsupported())
    }

    pub fn set_text(&self, _widget: &Widget, _text: &str) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn set_enabled(&self, _widget: &Widget, _enabled: bool) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn set_text_color(&self, _widget: &Widget, _rgb: (f32, f32, f32)) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn style(
        &self,
        _background: (f32, f32, f32),
        _corner_radius: f64,
        _border: (f32, f32, f32),
        _border_width: f64,
    ) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn set_button_tint(
        &self,
        _widget: &Widget,
        _rgb: (f32, f32, f32),
        _emphasis: bool,
    ) -> Result<(), Error> {
        Err(unsupported())
    }

    pub fn on_action<F>(&self, _callback: F) -> Result<(), Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        Err(unsupported())
    }

    pub fn close(&self) -> Result<(), Error> {
        Err(unsupported())
    }
}
