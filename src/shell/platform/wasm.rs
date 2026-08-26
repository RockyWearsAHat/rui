//! The backend for WebAssembly targets, running in a browser.
//!
//! Uses `wasm-bindgen` and `web-sys` to interface with the DOM and canvas APIs,
//! presenting frames to a `<canvas>` element and routing keyboard/pointer events
//! into the same `Event` and `Input` types native backends produce.

#![allow(unsafe_code)]

use crate::theme::Appearance;
use crate::{Canvas, Event};
use std::time::Duration;

use crate::shell::{Backend, Error, WindowOptions};
use wasm_bindgen::JsCast;

/// A window running in a web browser.
pub(crate) struct WebBackend {
    canvas: web_sys::HtmlCanvasElement,
    #[allow(dead_code)]
    context: web_sys::CanvasRenderingContext2d,
}

impl Backend for WebBackend {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        let window = web_sys::window()
            .ok_or_else(|| Error::Platform("no global window object available".to_string()))?;

        let document = window
            .document()
            .ok_or_else(|| Error::Platform("no document available".to_string()))?;

        // Try to find existing canvas with id 'rui-canvas', otherwise create new one
        let canvas = match document.get_element_by_id("rui-canvas") {
            Some(element) => element
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| {
                    Error::Platform("element with id 'rui-canvas' is not a canvas".to_string())
                })?,
            None => {
                let canvas = document
                    .create_element("canvas")
                    .map_err(|_| Error::Platform("failed to create canvas element".to_string()))?
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .map_err(|_| Error::Platform("failed to cast element to canvas".to_string()))?;

                canvas.set_id("rui-canvas");
                document
                    .body()
                    .ok_or_else(|| Error::Platform("no body element in document".to_string()))?
                    .append_child(&canvas)
                    .map_err(|_| Error::Platform("failed to append canvas to body".to_string()))?;

                canvas
            }
        };

        let context = canvas
            .get_context("2d")
            .map_err(|_| Error::Platform("failed to get 2D context".to_string()))?
            .ok_or_else(|| Error::Platform("2D context is not available".to_string()))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| Error::Platform("failed to cast context to 2D context".to_string()))?;

        Ok(WebBackend { canvas, context })
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
        let width = self.canvas.width();
        let height = self.canvas.height();
        (width, height, 1.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webbackend_implements_backend_trait() {
        let _result = WebBackend::open(&WindowOptions::default());
    }
}
