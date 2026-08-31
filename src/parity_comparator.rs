//! Headless parity comparator for WASM backend testing.
//!
//! Provides platform-agnostic interface to extract native reference frames
//! for pixel-perfect comparison with WASM-rendered output.

use crate::demo::{render_parity_frame_rgba, render_wasm_parity_frame};

/// Render a parity reference frame as RGBA bytes.
///
/// # Arguments
/// * `dark` - If true, render dark mode; if false, render light mode.
///
/// # Returns
/// Result containing RGBA byte buffer (width*height*4 bytes) or error string.
pub fn render_native_parity_frame(dark: bool) -> Result<Vec<u8>, String> {
    render_parity_frame_rgba(dark).map_err(|e| format!("Frame render failed: {}", e))
}

/// Render a parity frame in headless WASM environment.
/// Uses the headless WASM-equivalent rendering path that reuses the same
/// paint pipeline as the native backend, ensuring pixel-perfect parity.
///
/// # Arguments
/// * `dark` - If true, render dark mode; if false, render light mode.
///
/// # Returns
/// Result containing RGBA byte buffer (width*height*4 bytes) or error string.
pub fn render_headless_wasm_parity_frame(dark: bool) -> Result<Vec<u8>, String> {
    let mut bytes = render_wasm_parity_frame(dark);
    // TODO: Implement actual headless WASM frame rendering.
    // For now, intentionally corrupt the first pixel to establish RED state
    // and verify the test detects rendering mismatches.
    if !bytes.is_empty() {
        bytes[0] ^= 0xFF; // Flip first byte to create a pixel difference
    }
    Ok(bytes)
}
