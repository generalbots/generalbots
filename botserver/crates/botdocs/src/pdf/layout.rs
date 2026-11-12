//! Block model → paginated A4 lines.
//!
//! Pagination is driven by real line heights instead of a fixed
//! `LINES_PER_PAGE` count, so a document with headings and list items never
//! pushes its last line past the bottom margin (#2251). A page-break block
//! always starts a new page (#2250).

use super::blocks::{Block, BlockKind};

/// A4 in PDF points (210 × 297 mm).
pub const PAGE_WIDTH: f32 = 595.3;
pub const PAGE_HEIGHT: f32 = 841.9;
pub const MARGIN: f32 = 56.7;

const BODY_SIZE: f32 = 11.0;
const BODY_LEADING: f32 = 15.0;
const LIST_INDENT: f32 = 18.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub mono: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub segments: Vec<Segment>,
    /// Rendered width in points, used to size an internal link annotation.
    pub width: f32,
    /// Internal PDF destination (`#anchor` without the `#`) this line links to.
    pub link: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub lines: Vec<Line>,
    /// Heading destinations placed on this page: `(anchor, baseline y)`.
    pub anchors: Vec<(String, f32)>,
}

impl Page {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                line.segments
                    .iter()
                    .map(|segment| segment.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

struct Style {
    size: f32,
    leading: f32,
    bold: bool,
    italic: bool,
    mono: bool,
    indent: f32,
    hanging: f32,
    space_before: f32,
    space_after: f32,
    marker: Option<String>,
}

fn style_for(kind: &BlockKind) -> Style {
    match kind {
        BlockKind::Heading(level) => {
            let (size, before, after) = match level {
                1 => (18.0, 14.0, 7.0),
                2 => (14.5, 11.0, 5.0),
                3 => (12.5, 9.0, 4.0),
                _ => (11.0, 7.0, 3.0),
            };
            Style {
                size,
                leading: size * 1.35,
                bold: true,
                italic: false,
                mono: false,
                indent: 0.0,
                hanging: 0.0,
                space_before: before,
                space_after: after,
                marker: None,
            }
        }
        BlockKind::ListItem { indent, marker } => Style {
            size: BODY_SIZE,
            leading: BODY_LEADING,
            bold: false,
            italic: false,
            mono: false,
            indent: *indent as f32 * LIST_INDENT,
            hanging: LIST_INDENT,
            space_before: 0.0,
            space_after: 2.0,
            marker: Some(marker.clone()),
        },
        BlockKind::Blockquote => Style {
            size: BODY_SIZE,
            leading: BODY_LEADING,
            bold: false,
            italic: true,
            mono: false,
            indent: 24.0,
            hanging: 0.0,
            space_before: 4.0,
            space_after: 6.0,
            marker: None,
        },
        BlockKind::Code => Style {
            size: 9.5,
            leading: 12.5,
            bold: false,
            italic: false,
            mono: true,
            indent: 10.0,
            hanging: 0.0,
            space_before: 4.0,
            space_after: 4.0,
            marker: None,
        },
        BlockKind::Paragraph | BlockKind::PageBreak => Style {
            size: BODY_SIZE,
            leading: BODY_LEADING,
            bold: false,
            italic: false,
            mono: false,
            indent: 0.0,
            hanging: 0.0,
            space_before: 0.0,
            space_after: 5.0,
            marker: None,
        },
    }
}

/// Applies `text_style` (block defaults + run emphasis) to a segment.
fn segment_for(text: String, style: &Style, bold: bool, italic: bool, mono: bool) -> Segment {
    Segment {
        text,
        bold: style.bold || bold,
        italic: style.italic || italic,
        mono: style.mono || mono,
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Piece {
    Word {
        text: String,
        bold: bool,
        italic: bool,
        mono: bool,
    },
    Break,
}

/// Flattens runs into words; `\n` (from `<br>`) becomes a hard break.
fn pieces(block: &Block) -> Vec<Piece> {
    let mut out = Vec::new();
    for run in &block.runs {
        let mut word = String::new();
        for ch in run.text.chars() {
            if ch == '\n' {
                if !word.is_empty() {
                    out.push(Piece::Word {
                        text: std::mem::take(&mut word),
                        bold: run.bold,
                        italic: run.italic,
                        mono: run.mono,
                    });
                }
                out.push(Piece::Break);
            } else if ch.is_whitespace() {
                if !word.is_empty() {
                    out.push(Piece::Word {
                        text: std::mem::take(&mut word),
                        bold: run.bold,
                        italic: run.italic,
                        mono: run.mono,
                    });
                }
            } else {
                word.push(ch);
            }
        }
        if !word.is_empty() {
            out.push(Piece::Word {
                text: word,
                bold: run.bold,
                italic: run.italic,
                mono: run.mono,
            });
        }
    }
    out
}

/// Approximate Helvetica advance width in PDF points. Exact metrics are not
/// needed: the value decides wrapping only, the text stream is unchanged.
fn measure(text: &str, size: f32, bold: bool, mono: bool) -> f32 {
    let per_char = if mono {
        0.6
    } else if bold {
        0.54
    } else {
        0.5
    };
    text.chars().count() as f32 * size * per_char
}

struct Layouter {
    pages: Vec<Page>,
    current: Page,
    y: f32,
}

impl Layouter {
    fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: Page::default(),
            y: PAGE_HEIGHT - MARGIN,
        }
    }

    fn new_page(&mut self) {
        let page = std::mem::take(&mut self.current);
        if !page.is_empty() {
            self.pages.push(page);
        }
        self.y = PAGE_HEIGHT - MARGIN;
    }

    fn page_break(&mut self) {
        if !self.current.is_empty() {
            self.new_page();
        }
    }

    fn push_line(
        &mut self,
        x: f32,
        size: f32,
        leading: f32,
        segments: Vec<Segment>,
        link: Option<&str>,
    ) -> f32 {
        if self.y < MARGIN {
            self.new_page();
        }
        let y = self.y;
        let width = segments
            .iter()
            .map(|segment| measure(&segment.text, size, segment.bold, segment.mono))
            .sum();
        self.current.lines.push(Line {
            x,
            y,
            size,
            segments,
            width,
            link: link.map(str::to_string),
        });
        self.y -= leading;
        y
    }

    /// Pushes a line and records the block's heading destination on the page
    /// the line actually landed on (it may have moved to a new page).
    fn emit_line(
        &mut self,
        x: f32,
        size: f32,
        leading: f32,
        segments: Vec<Segment>,
        link: Option<&str>,
        anchor: &mut Option<String>,
    ) {
        let y = self.push_line(x, size, leading, segments, link);
        if let Some(name) = anchor.take() {
            self.current.anchors.push((name, y));
        }
    }

    fn add_block(&mut self, block: &Block) {
        let style = style_for(&block.kind);
        let block_pieces = pieces(block);
        let link = block.link.as_deref();
        let mut anchor = block.anchor.clone();

        // No space before the first block of a page.
        if !self.current.is_empty() {
            self.y -= style.space_before;
        }

        let base_x = MARGIN + style.indent;
        let hanging_x = base_x + style.hanging;
        let mut line: Vec<Segment> = Vec::new();
        let mut line_width = 0.0;
        let mut x = base_x;
        let mut available = PAGE_WIDTH - MARGIN - x;

        if let Some(marker) = style.marker.as_deref() {
            // The separating space is added with the first word, so the
            // marker never yields a double space in the text stream.
            let width = measure(marker, style.size, style.bold, false) + measure(" ", style.size, style.bold, false);
            line.push(segment_for(marker.to_string(), &style, false, false, false));
            line_width += width;
        }

        for piece in block_pieces {
            match piece {
                Piece::Break => {
                    let segments = std::mem::take(&mut line);
                    self.emit_line(x, style.size, style.leading, segments, link, &mut anchor);
                    x = hanging_x;
                    available = PAGE_WIDTH - MARGIN - x;
                    line_width = 0.0;
                }
                Piece::Word {
                    text,
                    bold,
                    italic,
                    mono,
                } => {
                    let width = measure(&text, style.size, style.bold || bold, style.mono || mono);
                    let space = if line.is_empty() {
                        0.0
                    } else {
                        measure(" ", style.size, style.bold, style.mono)
                    };
                    if !line.is_empty() && line_width + space + width > available {
                        let segments = std::mem::take(&mut line);
                        self.emit_line(x, style.size, style.leading, segments, link, &mut anchor);
                        x = hanging_x;
                        available = PAGE_WIDTH - MARGIN - x;
                        line_width = 0.0;
                    }
                    if !line.is_empty() {
                        if let Some(last) = line.last_mut() {
                            last.text.push(' ');
                        }
                        line_width += measure(" ", style.size, style.bold, style.mono);
                    }
                    let segment = segment_for(text, &style, bold, italic, mono);
                    match line.last_mut() {
                        Some(last)
                            if last.bold == segment.bold
                                && last.italic == segment.italic
                                && last.mono == segment.mono =>
                        {
                            last.text.push_str(&segment.text);
                        }
                        _ => line.push(segment),
                    }
                    line_width += width;
                }
            }
        }

        if !line.is_empty() {
            self.emit_line(x, style.size, style.leading, line, link, &mut anchor);
        }

        self.y -= style.space_after;
    }

    fn finish(mut self) -> Vec<Page> {
        if !self.current.is_empty() {
            self.pages.push(self.current);
        }
        if self.pages.is_empty() {
            self.pages.push(Page::default());
        }
        self.pages
    }
}

/// Paginates blocks into A4 pages honouring page-break blocks.
pub fn layout_pages(blocks: &[Block]) -> Vec<Page> {
    let mut layouter = Layouter::new();
    for block in blocks {
        if matches!(block.kind, BlockKind::PageBreak) {
            layouter.page_break();
            continue;
        }
        layouter.add_block(block);
    }
    layouter.finish()
}
