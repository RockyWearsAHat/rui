//! Tokenizer for syntax highlighting.

use crate::style::Tone;

/// A programming language for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript
    JavaScript,
    /// Bash
    Bash,
    /// Diff format
    Diff,
}

/// A token type with its semantic meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Keywords like `let`, `fn`, `if`, etc.
    Keyword,
    /// String literals
    String,
    /// Comments
    Comment,
    /// Everything else
    Other,
}

impl TokenType {
    /// Resolve token type to a display tone.
    pub fn tone(self) -> Tone {
        match self {
            TokenType::Keyword => Tone::Accent,
            TokenType::String => Tone::Accent, // Can be customized
            TokenType::Comment => Tone::Muted,
            TokenType::Other => Tone::Text,
        }
    }
}

/// A span of text with its token type.
#[derive(Debug, Clone)]
pub struct Token {
    /// The text of the token.
    pub text: String,
    /// What kind of token it is.
    pub ty: TokenType,
}

/// Tokenize source code for a given language.
pub fn tokenize(code: &str, language: Language) -> Vec<Token> {
    match language {
        Language::Rust => tokenize_rust(code),
        Language::Python => tokenize_python(code),
        Language::JavaScript => tokenize_javascript(code),
        Language::Bash => tokenize_bash(code),
        Language::Diff => tokenize_diff(code),
    }
}

fn tokenize_rust(code: &str) -> Vec<Token> {
    let keywords = [
        "let", "mut", "fn", "if", "else", "match", "for", "while", "loop", "return", "pub",
        "struct", "enum", "impl", "trait", "use", "const", "static", "unsafe", "async", "await",
    ];

    let mut tokens = Vec::new();
    let mut chars = code.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        // Handle comments
        if ch == '/' && chars.clone().nth(1) == Some('/') {
            if !current.is_empty() {
                tokens.push(Token {
                    text: current.clone(),
                    ty: TokenType::Other,
                });
                current.clear();
            }

            let mut comment = String::new();
            while let Some(&c) = chars.peek() {
                comment.push(c);
                chars.next();
                if c == '\n' {
                    break;
                }
            }
            tokens.push(Token {
                text: comment,
                ty: TokenType::Comment,
            });
        }
        // Handle strings
        else if ch == '"' {
            if !current.is_empty() {
                let ty = if keywords.contains(&current.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Other
                };
                tokens.push(Token {
                    text: current.clone(),
                    ty,
                });
                current.clear();
            }

            let mut string = String::new();
            string.push(ch);
            chars.next();
            let mut escaped = false;

            while let Some(&c) = chars.peek() {
                string.push(c);
                chars.next();
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    break;
                }
            }

            tokens.push(Token {
                text: string,
                ty: TokenType::String,
            });
        }
        // Handle identifiers and keywords
        else if ch.is_alphabetic() || ch == '_' {
            current.push(ch);
            chars.next();
        }
        // Handle whitespace and punctuation
        else {
            if !current.is_empty() {
                let ty = if keywords.contains(&current.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Other
                };
                tokens.push(Token {
                    text: current.clone(),
                    ty,
                });
                current.clear();
            }

            current.push(ch);
            chars.next();

            if ch.is_whitespace() {
                tokens.push(Token {
                    text: current.clone(),
                    ty: TokenType::Other,
                });
                current.clear();
            }
        }
    }

    if !current.is_empty() {
        let ty = if keywords.contains(&current.as_str()) {
            TokenType::Keyword
        } else {
            TokenType::Other
        };
        tokens.push(Token { text: current, ty });
    }

    tokens
}

fn tokenize_python(code: &str) -> Vec<Token> {
    let keywords = [
        "def", "class", "if", "else", "for", "while", "return", "import", "from",
    ];
    basic_tokenize(code, &keywords)
}

fn tokenize_javascript(code: &str) -> Vec<Token> {
    let keywords = [
        "let", "const", "var", "function", "if", "else", "for", "while", "return",
    ];
    basic_tokenize(code, &keywords)
}

fn tokenize_bash(code: &str) -> Vec<Token> {
    let keywords = ["if", "else", "for", "while", "do", "done", "function"];
    basic_tokenize(code, &keywords)
}

fn tokenize_diff(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for line in code.lines() {
        let ty = if line.starts_with('+') {
            TokenType::Keyword
        } else if line.starts_with('-') {
            TokenType::Comment
        } else {
            TokenType::Other
        };
        tokens.push(Token {
            text: line.to_string(),
            ty,
        });
        tokens.push(Token {
            text: "\n".to_string(),
            ty: TokenType::Other,
        });
    }
    tokens
}

fn basic_tokenize(code: &str, keywords: &[&str]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = code.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '"' || ch == '\'' {
            if !current.is_empty() {
                let ty = if keywords.contains(&current.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Other
                };
                tokens.push(Token {
                    text: current.clone(),
                    ty,
                });
                current.clear();
            }

            let quote = ch;
            let mut string = String::new();
            string.push(ch);
            chars.next();

            while let Some(&c) = chars.peek() {
                string.push(c);
                chars.next();
                if c == quote {
                    break;
                }
            }

            tokens.push(Token {
                text: string,
                ty: TokenType::String,
            });
        } else if ch == '#' {
            if !current.is_empty() {
                let ty = if keywords.contains(&current.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Other
                };
                tokens.push(Token {
                    text: current.clone(),
                    ty,
                });
                current.clear();
            }

            let mut comment = String::new();
            while let Some(&c) = chars.peek() {
                comment.push(c);
                chars.next();
                if c == '\n' {
                    break;
                }
            }
            tokens.push(Token {
                text: comment,
                ty: TokenType::Comment,
            });
        } else if ch.is_alphabetic() || ch == '_' {
            current.push(ch);
            chars.next();
        } else {
            if !current.is_empty() {
                let ty = if keywords.contains(&current.as_str()) {
                    TokenType::Keyword
                } else {
                    TokenType::Other
                };
                tokens.push(Token {
                    text: current.clone(),
                    ty,
                });
                current.clear();
            }

            current.push(ch);
            chars.next();

            if ch.is_whitespace() {
                tokens.push(Token {
                    text: current.clone(),
                    ty: TokenType::Other,
                });
                current.clear();
            }
        }
    }

    if !current.is_empty() {
        let ty = if keywords.contains(&current.as_str()) {
            TokenType::Keyword
        } else {
            TokenType::Other
        };
        tokens.push(Token { text: current, ty });
    }

    tokens
}
