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
    fn test_embedded_fonts_parse_and_load_glyphs() {
        let ui_font = embedded_ui_font().expect("UI font should parse");
        let mono_font = embedded_mono_font().expect("Mono font should parse");

        // Verify UI font can load at least one glyph
        let ui_glyph_id = ui_font.glyph_for('A');
        let ui_rendered = ui_font.render(ui_glyph_id, 16.0, 0.0);
        assert!(
            !ui_rendered.mask.is_empty(),
            "UI font should render glyph 'A'"
        );

        // Verify mono font can load at least one glyph
        let mono_glyph_id = mono_font.glyph_for('A');
        let mono_rendered = mono_font.render(mono_glyph_id, 16.0, 0.0);
        assert!(
            !mono_rendered.mask.is_empty(),
            "Mono font should render glyph 'A'"
        );
    }
}
