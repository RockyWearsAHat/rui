//! Markdown renderer — hand-written line-oriented parser with no dependencies.
#![allow(clippy::while_let_on_iterator, clippy::redundant_pattern_matching)]

use crate::code_view::code_view;
use crate::element::El;
use crate::style::{Length, Tone};
use crate::syntax::language_for_path;
use crate::table::{column, table, table_row};
use crate::widgets::{col, divider, heading, link, micro, panel, row, text, title};

/// A markdown document, rendered. Links are drawn but do nothing.
pub fn markdown<S: 'static>(source: &str) -> El<S> {
    markdown_with(source, |_: &mut S, _: String| {})
}

/// The same, with `on_link` called with a link's href when it is clicked.
pub fn markdown_with<S: 'static>(
    source: &str,
    on_link: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    let blocks = parse_blocks(source);
    let elements: Vec<El<S>> = blocks
        .into_iter()
        .map(|block| render_block(block, on_link))
        .collect();
    col(elements).gap(12.0)
}

#[derive(Debug, Clone)]
enum Block {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    Paragraph(String),
    Code {
        language: String,
        code: String,
    },
    BulletList(Vec<String>),
    NumberedList(Vec<String>),
    Table {
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Divider,
    Quote(String),
}

fn parse_blocks(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Skip empty lines
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        // Headings
        if let Some(rest) = line.strip_prefix("# ") {
            blocks.push(Block::Heading1(rest.to_string()));
            i += 1;
        } else if let Some(rest) = line.strip_prefix("## ") {
            blocks.push(Block::Heading2(rest.to_string()));
            i += 1;
        } else if let Some(rest) = line.strip_prefix("### ") {
            blocks.push(Block::Heading3(rest.to_string()));
            i += 1;
        }
        // Code fences
        else if line.trim().starts_with("```") {
            let info = line.trim_start_matches('`').trim().to_string();
            let mut code = String::new();
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            blocks.push(Block::Code {
                language: info,
                code,
            });
        }
        // Bullet lists
        else if line.trim_start().starts_with("- ")
            || line.trim_start().starts_with("* ")
            || line.trim_start().starts_with("+ ")
        {
            let mut items = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim_start();
                if let Some(item) = l
                    .strip_prefix("- ")
                    .or_else(|| l.strip_prefix("* "))
                    .or_else(|| l.strip_prefix("+ "))
                {
                    items.push(item.to_string());
                    i += 1;
                } else if l.is_empty() {
                    i += 1;
                    if i < lines.len() && !lines[i].trim_start().starts_with(['-', '*', '+']) {
                        break;
                    }
                } else {
                    break;
                }
            }
            blocks.push(Block::BulletList(items));
        }
        // Numbered lists
        else if is_numbered_list_start(line) {
            let mut items = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim_start();
                if let Some(item) = extract_numbered_item(l) {
                    items.push(item);
                    i += 1;
                } else if l.is_empty() {
                    i += 1;
                    if i < lines.len() && !is_numbered_list_start(lines[i].trim_start()) {
                        break;
                    }
                } else {
                    break;
                }
            }
            blocks.push(Block::NumberedList(items));
        }
        // Tables
        else if line.contains('|') {
            let table_result = parse_table(&lines, i);
            if let Some((table_block, new_i)) = table_result {
                blocks.push(table_block);
                i = new_i;
            } else {
                let mut para_text = line.to_string();
                i += 1;
                while i < lines.len() && !lines[i].trim().is_empty() {
                    if is_block_start(lines[i]) {
                        break;
                    }
                    para_text.push('\n');
                    para_text.push_str(lines[i]);
                    i += 1;
                }
                blocks.push(Block::Paragraph(para_text));
            }
        }
        // Dividers
        else if is_divider(line) {
            blocks.push(Block::Divider);
            i += 1;
        }
        // Block quotes
        else if line.trim_start().starts_with("> ") {
            let mut quote = String::new();
            while i < lines.len() && lines[i].trim_start().starts_with("> ") {
                let content = &lines[i].trim_start()[2..];
                if !quote.is_empty() {
                    quote.push('\n');
                }
                quote.push_str(content);
                i += 1;
            }
            blocks.push(Block::Quote(quote));
        }
        // Paragraphs
        else {
            let mut para_text = line.to_string();
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty() {
                if is_block_start(lines[i]) {
                    break;
                }
                para_text.push('\n');
                para_text.push_str(lines[i]);
                i += 1;
            }
            blocks.push(Block::Paragraph(para_text));
        }
    }

    blocks
}

fn is_numbered_list_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
        if let Some(after) = rest.strip_prefix(|c: char| c.is_ascii_digit()) {
            if let Some(after) = after.strip_prefix(|c: char| c.is_ascii_digit()) {
                if after.strip_prefix(". ").is_some() {
                    return true;
                }
            } else if let Some(_) = after.strip_prefix(". ") {
                return true;
            }
        } else if let Some(_) = rest.strip_prefix(". ") {
            return true;
        }
    }
    false
}

fn extract_numbered_item(line: &str) -> Option<String> {
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            continue;
        } else if c == '.' {
            if let Some(' ') = chars.next() {
                return Some(chars.collect());
            }
            return None;
        } else {
            return None;
        }
    }
    None
}

fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("```")
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || is_numbered_list_start(line)
        || trimmed.contains('|')
        || is_divider(line)
        || trimmed.starts_with("> ")
}

fn is_divider(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---" || trimmed == "***"
}

fn parse_table(lines: &[&str], start_idx: usize) -> Option<(Block, usize)> {
    if start_idx >= lines.len() {
        return None;
    }

    let header_line = lines[start_idx];
    if !header_line.contains('|') {
        return None;
    }

    let header = parse_table_row(header_line);
    if header.is_empty() {
        return None;
    }

    let mut i = start_idx + 1;

    if i < lines.len() {
        let sep_line = lines[i];
        if is_table_separator(sep_line) {
            i += 1;
        } else {
            return None;
        }
    } else {
        return None;
    }

    let mut rows = Vec::new();
    while i < lines.len() {
        let l = lines[i].trim();
        if l.is_empty() {
            break;
        }
        if !l.contains('|') {
            break;
        }
        rows.push(parse_table_row(lines[i]));
        i += 1;
    }

    Some((Block::Table { header, rows }, i))
}

fn parse_table_row(line: &str) -> Vec<String> {
    line.split('|')
        .skip(1)
        .map(|cell| cell.trim().to_string())
        .filter(|cell| !cell.is_empty())
        .collect()
}

fn is_table_separator(line: &str) -> bool {
    let cells: Vec<&str> = line.split('|').collect();
    let mut has_valid = false;
    for cell in cells {
        let trimmed = cell.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.chars().all(|c| c == '-' || c == ':') {
            return false;
        }
        has_valid = true;
    }
    has_valid
}

fn render_block<S: 'static>(
    block: Block,
    on_link: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    match block {
        Block::Heading1(heading_text) => col((title(heading_text).bold(), divider())).gap(8.0),
        Block::Heading2(heading_text) => col((heading(heading_text).bold(), divider())).gap(8.0),
        Block::Heading3(heading_text) => heading(heading_text).bold(),
        Block::Paragraph(para_text) => parse_and_render_inline(para_text, on_link),
        Block::Code { language, code } => {
            let lang = if language.is_empty() {
                None
            } else {
                language_for_path(&language)
            };
            let cv = code_view(&code);
            let cv = if let Some(lang) = lang {
                cv.language(lang)
            } else {
                cv
            };
            panel(cv.numbers(false).build())
        }
        Block::BulletList(items) => {
            let elements: Vec<El<S>> = items
                .into_iter()
                .map(|item| {
                    row((
                        text("•").color(Tone::Muted).w(16.0),
                        parse_and_render_inline(item, on_link),
                    ))
                })
                .collect();
            col(elements).gap(8.0)
        }
        Block::NumberedList(items) => {
            let elements: Vec<El<S>> = items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    row((
                        text(format!("{}.", idx + 1)).color(Tone::Muted).w(16.0),
                        parse_and_render_inline(item, on_link),
                    ))
                })
                .collect();
            col(elements).gap(8.0)
        }
        Block::Table { header, rows } => {
            let mut cols = Vec::new();
            for i in 0..header.len() {
                let key: &'static str = Box::leak(format!("col{}", i).into_boxed_str());
                cols.push(column(key, Length::Fill(1.0)));
            }

            let mut table_rows = Vec::new();
            for (row_idx, row_cells) in rows.into_iter().enumerate() {
                let cells: Vec<El<S>> = row_cells
                    .into_iter()
                    .map(|cell| parse_and_render_inline(cell, on_link))
                    .collect();
                let row_key: &'static str = Box::leak(format!("row{}", row_idx).into_boxed_str());
                table_rows.push(table_row(row_key, cells));
            }

            let header_cells: Vec<El<S>> = header.into_iter().map(|h| text(h).bold()).collect();

            table(&cols, Some(header_cells), table_rows)
        }
        Block::Divider => divider(),
        Block::Quote(quote_text) => parse_and_render_inline(quote_text, on_link)
            .color(Tone::Muted)
            .border(2.0, Tone::Border)
            .pad(12.0),
    }
}

fn parse_and_render_inline<S: 'static>(
    source_text: String,
    on_link: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    let spans = parse_inline(source_text, on_link);
    row(spans).wrap()
}

fn parse_inline<S: 'static>(
    source_text: String,
    on_link: impl Fn(&mut S, String) + Copy + 'static,
) -> Vec<El<S>> {
    let mut result = Vec::new();
    let mut chars = source_text.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            '`' => {
                if !current.is_empty() {
                    result.push(text(current.clone()));
                    current.clear();
                }
                let mut code_text = String::new();
                while let Some(c) = chars.next() {
                    if c == '`' {
                        break;
                    }
                    code_text.push(c);
                }
                result.push(text(code_text));
            }
            '[' => {
                if !current.is_empty() {
                    result.push(text(current.clone()));
                    current.clear();
                }
                let mut link_text = String::new();
                let mut found_bracket = false;
                while let Some(c) = chars.next() {
                    if c == ']' {
                        found_bracket = true;
                        break;
                    }
                    link_text.push(c);
                }

                if found_bracket {
                    if let Some('(') = chars.peek() {
                        chars.next();
                        let mut href = String::new();
                        while let Some(c) = chars.next() {
                            if c == ')' {
                                break;
                            }
                            href.push(c);
                        }
                        let href_clone = href.clone();
                        result.push(
                            link(link_text)
                                .on_click(move |s: &mut S| on_link(s, href_clone.clone())),
                        );
                    } else {
                        result.push(text(format!("[{}]", link_text)));
                    }
                } else {
                    result.push(text(format!("[{}", link_text)));
                }
            }
            '!' => {
                if chars.peek() == Some(&'[') {
                    if !current.is_empty() {
                        result.push(text(current.clone()));
                        current.clear();
                    }
                    chars.next();
                    let mut alt = String::new();
                    while let Some(c) = chars.next() {
                        if c == ']' {
                            break;
                        }
                        alt.push(c);
                    }

                    if let Some('(') = chars.peek() {
                        chars.next();
                        let mut _src = String::new();
                        while let Some(c) = chars.next() {
                            if c == ')' {
                                break;
                            }
                            _src.push(c);
                        }
                        let placeholder = format!("[image: {}]", alt);
                        result.push(micro(placeholder).color(Tone::Muted));
                    } else {
                        result.push(text(format!("![{}]", alt)));
                    }
                } else {
                    current.push(c);
                }
            }
            '*' => {
                if chars.peek() == Some(&'*') {
                    if !current.is_empty() {
                        result.push(text(current.clone()));
                        current.clear();
                    }
                    chars.next();
                    let mut bold_text = String::new();
                    while let Some(c) = chars.next() {
                        if c == '*' && chars.peek() == Some(&'*') {
                            chars.next();
                            break;
                        }
                        bold_text.push(c);
                    }
                    result.push(text(bold_text).bold());
                } else {
                    if !current.is_empty() {
                        result.push(text(current.clone()));
                        current.clear();
                    }
                    let mut em_text = String::new();
                    while let Some(c) = chars.next() {
                        if c == '*' {
                            break;
                        }
                        em_text.push(c);
                    }
                    result.push(text(em_text).color(Tone::Text).tracking(0.3));
                }
            }
            '_' => {
                if chars.peek() == Some(&'_') {
                    if !current.is_empty() {
                        result.push(text(current.clone()));
                        current.clear();
                    }
                    chars.next();
                    let mut bold_text = String::new();
                    while let Some(c) = chars.next() {
                        if c == '_' && chars.peek() == Some(&'_') {
                            chars.next();
                            break;
                        }
                        bold_text.push(c);
                    }
                    result.push(text(bold_text).bold());
                } else {
                    if !current.is_empty() {
                        result.push(text(current.clone()));
                        current.clear();
                    }
                    let mut em_text = String::new();
                    while let Some(c) = chars.next() {
                        if c == '_' {
                            break;
                        }
                        em_text.push(c);
                    }
                    result.push(text(em_text).color(Tone::Text).tracking(0.3));
                }
            }
            '<' => {
                if !current.is_empty() {
                    result.push(text(current.clone()));
                    current.clear();
                }
                while let Some(c) = chars.next() {
                    if c == '>' {
                        break;
                    }
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        result.push(text(current));
    }

    if result.is_empty() {
        result.push(text(""));
    }

    result
}
