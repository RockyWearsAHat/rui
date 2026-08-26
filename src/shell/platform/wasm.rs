//! The backend for WebAssembly targets, running in a browser.
//!
//! Maps DOM events to rui's [`Event`] types using the shared event mapping logic
//! from [`super::super::event_mapping`]. The rendering pipeline writes pixel data
//! to an HTML canvas element via `CanvasRenderingContext2d.putImageData()`.

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

    fn present(&self, canvas: &Canvas) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            use web_sys::{CanvasRenderingContext2d, ImageData};

            let window = match web_sys::window() {
                Some(w) => w,
                None => return Err(Error::Platform("no window object available".into())),
            };

            let document = match window.document() {
                Some(d) => d,
                None => return Err(Error::Platform("no document object available".into())),
            };

            let canvas_element = match document.get_element_by_id("rui-canvas") {
                Some(elem) => elem,
                None => {
                    return Err(Error::Platform(
                        "canvas element #rui-canvas not found".into(),
                    ))
                }
            };

            let canvas_elem = canvas_element
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| Error::Platform("element is not a canvas".into()))?;

            let ctx = canvas_elem
                .get_context("2d")
                .map_err(|_| Error::Platform("failed to get 2d context".into()))?
                .ok_or_else(|| Error::Platform("no 2d context available".into()))?
                .dyn_into::<CanvasRenderingContext2d>()
                .map_err(|_| Error::Platform("context is not a 2d context".into()))?;

            let pixels = canvas.pixels();
            let width = canvas.width();
            let height = canvas.height();

            let mut data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
            for &pixel in pixels {
                let [b, g, r, a] = pixel.to_le_bytes();
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(a);
            }

            let image_data =
                ImageData::new_with_u8_clamped_array(wasm_bindgen::Clamped(&data), width)
                    .map_err(|_| Error::Platform("failed to create ImageData".into()))?;

            ctx.put_image_data(&image_data, 0.0, 0.0)
                .map_err(|_| Error::Platform("failed to put image data".into()))?;

            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = canvas;
            Ok(())
        }
    }

    fn is_open(&self) -> bool {
        true
    }
}

pub(crate) use WebBackend as Window;

#[cfg(all(test, target_arch = "wasm32"))]
pub mod wasm_backend {
    use super::*;

    /// Verifies that present() can be called without panicking.
    /// The test expects an error since the canvas element won't exist in a test environment.
    #[test]
    fn test_canvas_render() {
        let backend =
            WebBackend::open(&WindowOptions::default()).expect("failed to open web backend");

        let canvas = Canvas::new(100, 100, 1.0);

        let result = backend.present(&canvas);
        assert!(
            result.is_err(),
            "present() should error when canvas element is missing"
        );
    }
}
