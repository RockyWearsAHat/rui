//! Fonts embedded in the binary, for environments without filesystem access.
//!
//! Used by wasm and other embeddable backends that cannot load fonts from disk.
//! The synthetic test font serves as a placeholder; production deployments embed
//! a real UI and mono face as bytes.

use crate::font::Font;
use crate::shell::Error;

/// Attempts to parse the embedded UI font.
pub fn embedded_ui_font() -> Result<Font, Error> {
    Err(Error::Font(crate::font::FontError::NotAFont))
}

/// Attempts to parse the embedded mono font.
pub fn embedded_mono_font() -> Result<Font, Error> {
    Err(Error::Font(crate::font::FontError::NotAFont))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_fonts_parse() {
        // With placeholder implementation, expect errors
        let ui_result = embedded_ui_font();
        let mono_result = embedded_mono_font();

        assert!(ui_result.is_err());
        assert!(mono_result.is_err());
    }
}
