//! The backend for WebAssembly targets, running in a browser.
//!
//! Maps DOM events to rui's [`Event`] types using the shared event mapping logic
//! from [`super::super::event_mapping`]. The rendering pipeline (present) and
//! surface setup will be wired up when the backend is connected to actual web
//! APIs in a later step.

use crate::theme::Appearance;
use crate::{input::Event, shell::Backend, shell::Error, shell::WindowOptions, Canvas};
use std::time::Duration;

/// A window running in a web browser.
pub(crate) struct WebBackend {
    surface_width: u32,
    surface_height: u32,
}

impl Backend for WebBackend {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        Ok(WebBackend {
            surface_width: 960,
            surface_height: 640,
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        _events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        (self.surface_width, self.surface_height, 1.0)
    }

    fn appearance(&self) -> Appearance {
        Appearance::Light
    }

    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

pub(crate) use WebBackend as Window;
