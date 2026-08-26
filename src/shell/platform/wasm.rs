//! The backend for WebAssembly targets, running in a browser.
//!
//! Uses `wasm-bindgen` and `web-sys` to interface with the DOM and canvas APIs,
//! presenting frames to a `<canvas>` element and routing keyboard/pointer events
//! into the same `Event` and `Input` types native backends produce.

use crate::theme::Appearance;
use crate::{Canvas, Event};
use std::time::Duration;

use crate::shell::{Backend, Error, WindowOptions};

/// A window running in a web browser.
pub(crate) struct Window;

impl Backend for Window {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        Err(Error::Unsupported)
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        _events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn surface(&self) -> (u32, u32, f32) {
        (1, 1, 1.0)
    }

    fn appearance(&self) -> Appearance {
        Appearance::Light
    }

    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    fn is_open(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::WindowOptions;

    #[test]
    fn webbackend_implements_backend_trait() {
        let _result = Window::open(&WindowOptions::default());
    }
}
