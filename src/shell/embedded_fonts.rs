//! Fonts embedded in the binary, for environments without filesystem access.
//!
//! Used by wasm and other embeddable backends that cannot load fonts from disk.

use crate::font::Font;
use crate::shell::Error;

/// Parses the embedded UI font (DejaVuSans).
pub fn embedded_ui_font() -> Result<Font, Error> {
    let bytes = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
    Font::parse(bytes.to_vec()).map_err(Error::Font)
}

/// Parses the embedded mono font (DejaVuSansMono).
pub fn embedded_mono_font() -> Result<Font, Error> {
    let bytes = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");
    Font::parse(bytes.to_vec()).map_err(Error::Font)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_fonts_parse() {
        let ui_result = embedded_ui_font();
        let mono_result = embedded_mono_font();

        assert!(ui_result.is_ok());
        assert!(mono_result.is_ok());
    }
}
