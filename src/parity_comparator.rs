//! Headless parity comparator for WASM backend testing.
//!
//! Provides platform-agnostic interface to extract native reference frames
//! for pixel-perfect comparison with WASM-rendered output.

use crate::demo::render_parity_frame_rgba;

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
/// Currently a stub returning mismatched bytes to establish RED test state.
/// This will be replaced with actual WASM-equivalent rendering in next phase.
///
/// # Arguments
/// * `dark` - If true, render dark mode; if false, render light mode.
///
/// # Returns
/// Result containing RGBA byte buffer (width*height*4 bytes) or error string.
pub fn render_headless_wasm_parity_frame(dark: bool) -> Result<Vec<u8>, String> {
    let mut bytes = render_parity_frame_rgba(dark)
        .map_err(|e| format!("Headless WASM frame render failed: {}", e))?;

    // Intentionally corrupt first pixel to establish RED state (test should fail)
    // This stub will be replaced with actual headless WASM rendering implementation
    if bytes.len() >= 4 {
        bytes[0] = bytes[0].wrapping_add(1);
    }

    Ok(bytes)
}
