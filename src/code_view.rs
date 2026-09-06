//! Code view component for syntax-highlighted source code.

use crate::element::El;
use crate::style::{Align, Tone};
use crate::syntax::{tokenize, Language, Token, TokenType};
use crate::widgets::{code, col, micro, row};
use std::ops::Range;

/// What colour each kind of token is drawn in.
#[derive(Clone, Copy)]
pub struct CodeStyle {
    /// Colour for keywords.
    pub keyword: Tone,
    /// Colour for strings.
    pub string: Tone,
    /// Colour for comments.
    pub comment: Tone,
    /// Colour for numbers.
    pub number: Tone,
    /// Colour for type names.
    pub type_name: Tone,
    /// Colour for function names.
    pub function: Tone,
    /// Colour for other tokens.
    pub other: Tone,
}

impl Default for CodeStyle {
    fn default() -> Self {
        CodeStyle {
            keyword: Tone::Accent,
            string: Tone::Text,
            comment: Tone::Muted,
            number: Tone::Text,
            type_name: Tone::Text,
            function: Tone::Text,
            other: Tone::Text,
        }
    }
}

/// A block of source: numbered, monospaced, and scrolled rather than wrapped.
pub struct CodeView {
    source: String,
    language: Option<Language>,
    first_line: usize,
    show_numbers: bool,
    highlight_range: Option<Range<usize>>,
    line_marks: Vec<(usize, Tone)>,
    style: CodeStyle,
}

/// Create a code view from source code.
pub fn code_view(source: &str) -> CodeView {
    CodeView {
        source: source.to_string(),
        language: None,
        first_line: 1,
        show_numbers: true,
        highlight_range: None,
        line_marks: Vec::new(),
        style: CodeStyle::default(),
    }
}

impl CodeView {
    /// Set the language for syntax highlighting.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Set the first line number. Default is 1.
    pub fn first_line(mut self, number: usize) -> Self {
        self.first_line = number;
        self
    }

    /// Show or hide the line number gutter. Default is true.
    pub fn numbers(mut self, on: bool) -> Self {
        self.show_numbers = on;
        self
    }

    /// Highlight these lines (0-based, relative to the source).
    pub fn highlight(mut self, lines: Range<usize>) -> Self {
        self.highlight_range = Some(lines);
        self
    }

    /// Set background colours for lines (0-based). These override highlight.
    pub fn line_marks(mut self, marks: Vec<(usize, Tone)>) -> Self {
        self.line_marks = marks;
        self
    }

    /// Override the token colours.
    pub fn style(mut self, style: CodeStyle) -> Self {
        self.style = style;
        self
    }

    /// Build the element.
    pub fn build<S: 'static>(self) -> El<S> {
        let lines: Vec<&str> = if self.source.is_empty() {
            vec![]
        } else {
            let lines_vec: Vec<&str> = self.source.lines().collect();
            if lines_vec.len() > 20000 {
                lines_vec.into_iter().take(20000).collect()
            } else {
                lines_vec
            }
        };

        let mut line_elements = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            // Expand tabs to 4 spaces
            let expanded_line = line.replace('\t', "    ");

            // Tokenize the line
            let tokens = if let Some(lang) = self.language {
                tokenize(&expanded_line, lang)
            } else {
                vec![Token {
                    text: expanded_line.clone(),
                    ty: TokenType::Other,
                }]
            };

            // Get the line number
            let line_number = self.first_line + line_idx;

            // Check if this line has a line mark
            let line_mark = self.line_marks.iter().find(|(idx, _)| *idx == line_idx);

            // Check if this line should be highlighted
            let is_highlighted = self
                .highlight_range
                .as_ref()
                .map(|r| r.contains(&line_idx))
                .unwrap_or(false);

            // Build the line element
            let line_el = if let Some((_, mark_tone)) = line_mark {
                // Line mark beats highlight
                build_line_with_mark(
                    line_number,
                    &tokens,
                    self.show_numbers,
                    *mark_tone,
                    self.style,
                )
            } else if is_highlighted {
                build_line_with_highlight(
                    line_number,
                    &tokens,
                    self.show_numbers,
                    Tone::Selection,
                    self.style,
                )
            } else {
                build_line(line_number, &tokens, self.show_numbers, self.style)
            };

            line_elements.push(line_el);
        }

        // Add truncation message if needed
        if self.source.lines().count() > 20000 {
            line_elements.push(
                micro("… truncated")
                    .h(18.0)
                    .color(Tone::Muted)
                    .key("truncated-line"),
            );
        }

        // Wrap in scrollable containers
        let vertical_scroll = col(line_elements).scroll();
        row((vertical_scroll,)).scroll()
    }
}

fn build_line<S: 'static>(
    line_number: usize,
    tokens: &[Token],
    show_numbers: bool,
    style: CodeStyle,
) -> El<S> {
    let mut children = Vec::new();

    // Add gutter if needed
    if show_numbers {
        let gutter = micro(line_number.to_string())
            .w(52.0)
            .text_align(Align::End)
            .color(Tone::Muted)
            .pad_x(8.0);
        children.push(gutter);
    }

    // Add tokens as spans
    let mut spans = Vec::new();
    for token in tokens {
        let color = token_color(token.ty, style);
        spans.push(code(token.text.clone()).color(color).mono());
    }

    children.push(row(spans));

    row(children).h(18.0).key(format!("line-{}", line_number))
}

fn build_line_with_highlight<S: 'static>(
    line_number: usize,
    tokens: &[Token],
    show_numbers: bool,
    highlight_tone: Tone,
    style: CodeStyle,
) -> El<S> {
    let mut children = Vec::new();

    // Add gutter if needed
    if show_numbers {
        let gutter = micro(line_number.to_string())
            .w(52.0)
            .text_align(Align::End)
            .color(Tone::Muted)
            .pad_x(8.0);
        children.push(gutter);
    }

    // Add tokens as spans with highlight
    let mut spans = Vec::new();
    for token in tokens {
        let color = token_color(token.ty, style);
        spans.push(code(token.text.clone()).color(color).mono());
    }

    children.push(row(spans).fill(highlight_tone));

    row(children)
        .h(18.0)
        .key(format!("line-{}", line_number))
        .fill(highlight_tone)
}

fn build_line_with_mark<S: 'static>(
    line_number: usize,
    tokens: &[Token],
    show_numbers: bool,
    mark_tone: Tone,
    style: CodeStyle,
) -> El<S> {
    let mut children = Vec::new();

    // Add gutter if needed
    if show_numbers {
        let gutter = micro(line_number.to_string())
            .w(52.0)
            .text_align(Align::End)
            .color(Tone::Muted)
            .pad_x(8.0);
        children.push(gutter);
    }

    // Add tokens as spans with mark
    let mut spans = Vec::new();
    for token in tokens {
        let color = token_color(token.ty, style);
        spans.push(code(token.text.clone()).color(color).mono());
    }

    children.push(row(spans).fill(mark_tone));

    row(children)
        .h(18.0)
        .key(format!("line-{}", line_number))
        .fill(mark_tone)
}

fn token_color(ty: TokenType, style: CodeStyle) -> Tone {
    match ty {
        TokenType::Keyword => style.keyword,
        TokenType::String => style.string,
        TokenType::Comment => style.comment,
        TokenType::Number => style.number,
        TokenType::Type => style.type_name,
        TokenType::Function => style.function,
        TokenType::Other => style.other,
    }
}

/// Detect language from file extension.
pub fn language_for_path(path: &str) -> Option<Language> {
    crate::syntax::language_for_path(path)
}
