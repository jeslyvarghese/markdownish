use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Parsed inline content element
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, url: String, title: Option<String> },
    Image { alt: String, url: String, title: Option<String> },
    SoftBreak,
    HardBreak,
}

/// Table column alignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnAlign {
    None,
    Left,
    Center,
    Right,
}

/// Top-level block element
#[derive(Debug, Clone)]
pub enum Block {
    Heading { level: u8, content: Vec<Inline>, anchor: String },
    Paragraph(Vec<Inline>),
    CodeBlock { language: Option<String>, code: String },
    BlockQuote(Vec<Block>),
    BulletList(Vec<ListItem>),
    OrderedList { start: u64, items: Vec<ListItem> },
    HorizontalRule,
    Table {
        alignments: Vec<ColumnAlign>,
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Vec<Block>,
}

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn parse(input: &str) -> Vec<Block> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        let events: Vec<Event> = Parser::new_ext(input, options).collect();
        let mut idx = 0;
        parse_blocks(&events, &mut idx)
    }
}

/// Convert heading text to a GitHub-style anchor slug.
pub fn heading_to_anchor(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .flat_map(|c| c.to_lowercase())
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn parse_blocks(events: &[Event], idx: &mut usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    while *idx < events.len() {
        match &events[*idx] {
            Event::Start(Tag::Heading { level, .. }) => {
                let level_num = heading_level_to_u8(level);
                *idx += 1;
                let content = parse_inlines_until(events, idx, TagEnd::Heading((*level).into()));
                let anchor = heading_to_anchor(&inlines_to_text(&content));
                blocks.push(Block::Heading { level: level_num, content, anchor });
            }
            Event::Start(Tag::Paragraph) => {
                *idx += 1;
                let content = parse_inlines_until(events, idx, TagEnd::Paragraph);
                if !content.is_empty() {
                    blocks.push(Block::Paragraph(content));
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let s = lang.to_string();
                        if s.is_empty() { None } else { Some(s) }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
                *idx += 1;
                let mut code = String::new();
                while *idx < events.len() {
                    match &events[*idx] {
                        Event::Text(t) => { code.push_str(t); *idx += 1; }
                        Event::End(TagEnd::CodeBlock) => { *idx += 1; break; }
                        _ => { *idx += 1; }
                    }
                }
                blocks.push(Block::CodeBlock { language, code });
            }
            Event::Start(Tag::BlockQuote(_)) => {
                *idx += 1;
                let inner = parse_blocks_until_end(events, idx, |e| {
                    matches!(e, Event::End(TagEnd::BlockQuote(_)))
                });
                blocks.push(Block::BlockQuote(inner));
            }
            Event::Start(Tag::List(start_num)) => {
                let start = *start_num;
                *idx += 1;
                let items = parse_list_items(events, idx);
                match start {
                    Some(n) => blocks.push(Block::OrderedList { start: n, items }),
                    None => blocks.push(Block::BulletList(items)),
                }
            }
            Event::Rule => {
                blocks.push(Block::HorizontalRule);
                *idx += 1;
            }
            Event::Start(Tag::Table(aligns)) => {
                let alignments: Vec<ColumnAlign> = aligns.iter().map(|a| match a {
                    pulldown_cmark::Alignment::Left   => ColumnAlign::Left,
                    pulldown_cmark::Alignment::Center => ColumnAlign::Center,
                    pulldown_cmark::Alignment::Right  => ColumnAlign::Right,
                    pulldown_cmark::Alignment::None   => ColumnAlign::None,
                }).collect();
                *idx += 1;
                let (headers, rows) = parse_table_contents(events, idx);
                blocks.push(Block::Table { alignments, headers, rows });
            }
            Event::End(_) => break,
            _ => { *idx += 1; }
        }
    }
    blocks
}

fn parse_table_contents(
    events: &[Event],
    idx: &mut usize,
) -> (Vec<Vec<Inline>>, Vec<Vec<Vec<Inline>>>) {
    let mut headers: Vec<Vec<Inline>> = Vec::new();
    let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();

    while *idx < events.len() {
        match &events[*idx] {
            Event::Start(Tag::TableHead) => {
                *idx += 1;
                headers = parse_table_row_cells(events, idx, TagEnd::TableHead);
            }
            Event::Start(Tag::TableRow) => {
                *idx += 1;
                let row = parse_table_row_cells(events, idx, TagEnd::TableRow);
                rows.push(row);
            }
            Event::End(TagEnd::Table) => { *idx += 1; break; }
            _ => { *idx += 1; }
        }
    }
    (headers, rows)
}

fn parse_table_row_cells(events: &[Event], idx: &mut usize, end: TagEnd) -> Vec<Vec<Inline>> {
    let mut cells: Vec<Vec<Inline>> = Vec::new();
    while *idx < events.len() {
        match &events[*idx] {
            Event::End(tag) if tag_end_matches(tag, &end) => { *idx += 1; break; }
            Event::Start(Tag::TableCell) => {
                *idx += 1;
                let cell = parse_inlines_until(events, idx, TagEnd::TableCell);
                cells.push(cell);
            }
            _ => { *idx += 1; }
        }
    }
    cells
}

fn is_inline_event(e: &Event) -> bool {
    matches!(
        e,
        Event::Text(_)
            | Event::Code(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Start(
                Tag::Strong
                    | Tag::Emphasis
                    | Tag::Strikethrough
                    | Tag::Link { .. }
                    | Tag::Image { .. }
            )
    )
}

fn parse_blocks_until_end(
    events: &[Event],
    idx: &mut usize,
    end_cond: impl Fn(&Event) -> bool,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    while *idx < events.len() {
        if end_cond(&events[*idx]) {
            *idx += 1;
            break;
        }
        match &events[*idx] {
            Event::Start(Tag::Paragraph) => {
                *idx += 1;
                let content = parse_inlines_until(events, idx, TagEnd::Paragraph);
                if !content.is_empty() {
                    blocks.push(Block::Paragraph(content));
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let level_num = heading_level_to_u8(level);
                let end = TagEnd::Heading((*level).into());
                *idx += 1;
                let content = parse_inlines_until(events, idx, end);
                let anchor = heading_to_anchor(&inlines_to_text(&content));
                blocks.push(Block::Heading { level: level_num, content, anchor });
            }
            Event::Start(Tag::List(start_num)) => {
                let start = *start_num;
                *idx += 1;
                let items = parse_list_items(events, idx);
                match start {
                    Some(n) => blocks.push(Block::OrderedList { start: n, items }),
                    None => blocks.push(Block::BulletList(items)),
                }
            }
            e if is_inline_event(e) => {
                let inlines = collect_bare_inlines(events, idx, &end_cond);
                if !inlines.is_empty() {
                    blocks.push(Block::Paragraph(inlines));
                }
            }
            _ => { *idx += 1; }
        }
    }
    blocks
}

fn collect_bare_inlines(
    events: &[Event],
    idx: &mut usize,
    end_cond: &dyn Fn(&Event) -> bool,
) -> Vec<Inline> {
    let stop = |e: &Event| -> bool {
        if end_cond(e) { return true; }
        matches!(
            e,
            Event::Start(
                Tag::Paragraph
                    | Tag::Heading { .. }
                    | Tag::CodeBlock(_)
                    | Tag::BlockQuote(_)
                    | Tag::List(_)
                    | Tag::Item
            ) | Event::End(TagEnd::Item | TagEnd::BlockQuote(_) | TagEnd::List(_))
            | Event::Rule
        )
    };

    let mut inlines = Vec::new();
    while *idx < events.len() && !stop(&events[*idx]) {
        collect_inline_event(events, idx, &mut inlines);
    }
    inlines
}

fn collect_inline_event(events: &[Event], idx: &mut usize, inlines: &mut Vec<Inline>) {
    match &events[*idx] {
        Event::Text(t) => { inlines.push(Inline::Text(t.to_string())); *idx += 1; }
        Event::Code(c) => { inlines.push(Inline::Code(c.to_string())); *idx += 1; }
        Event::SoftBreak => { inlines.push(Inline::SoftBreak); *idx += 1; }
        Event::HardBreak => { inlines.push(Inline::HardBreak); *idx += 1; }
        Event::Start(Tag::Strong) => {
            *idx += 1;
            let inner = parse_inlines_inner(events, idx, &TagEnd::Strong);
            inlines.push(Inline::Bold(inner));
        }
        Event::Start(Tag::Emphasis) => {
            *idx += 1;
            let inner = parse_inlines_inner(events, idx, &TagEnd::Emphasis);
            inlines.push(Inline::Italic(inner));
        }
        Event::Start(Tag::Strikethrough) => {
            *idx += 1;
            let inner = parse_inlines_inner(events, idx, &TagEnd::Strikethrough);
            inlines.push(Inline::Strikethrough(inner));
        }
        Event::Start(Tag::Link { dest_url, title, .. }) => {
            let url = dest_url.to_string();
            let title_opt = if title.is_empty() { None } else { Some(title.to_string()) };
            *idx += 1;
            let text = parse_inlines_inner(events, idx, &TagEnd::Link);
            inlines.push(Inline::Link { text, url, title: title_opt });
        }
        Event::Start(Tag::Image { dest_url, title, .. }) => {
            let url = dest_url.to_string();
            let title_opt = if title.is_empty() { None } else { Some(title.to_string()) };
            *idx += 1;
            let mut alt = String::new();
            while *idx < events.len() {
                match &events[*idx] {
                    Event::Text(t) => { alt.push_str(t); *idx += 1; }
                    Event::End(TagEnd::Image) => { *idx += 1; break; }
                    _ => { *idx += 1; }
                }
            }
            inlines.push(Inline::Image { alt, url, title: title_opt });
        }
        _ => { *idx += 1; }
    }
}

fn parse_list_items(events: &[Event], idx: &mut usize) -> Vec<ListItem> {
    let mut items = Vec::new();
    while *idx < events.len() {
        match &events[*idx] {
            Event::Start(Tag::Item) => {
                *idx += 1;
                let content = parse_blocks_until_end(events, idx, |e| {
                    matches!(e, Event::End(TagEnd::Item))
                });
                items.push(ListItem { content });
            }
            Event::End(TagEnd::List(_)) => { *idx += 1; break; }
            _ => { *idx += 1; }
        }
    }
    items
}

fn parse_inlines_until(events: &[Event], idx: &mut usize, end: TagEnd) -> Vec<Inline> {
    parse_inlines_inner(events, idx, &end)
}

fn parse_inlines_inner(events: &[Event], idx: &mut usize, end: &TagEnd) -> Vec<Inline> {
    let mut inlines = Vec::new();
    while *idx < events.len() {
        match &events[*idx] {
            Event::End(tag) if tag_end_matches(tag, end) => {
                *idx += 1;
                break;
            }
            _ => collect_inline_event(events, idx, &mut inlines),
        }
    }
    inlines
}

fn tag_end_matches(a: &TagEnd, b: &TagEnd) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

fn heading_level_to_u8(level: &HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub fn inlines_to_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => out.push_str(t),
            Inline::Bold(inner)
            | Inline::Italic(inner)
            | Inline::Strikethrough(inner) => {
                out.push_str(&inlines_to_text(inner));
            }
            Inline::Code(c) => out.push_str(c),
            Inline::Link { text, .. } => out.push_str(&inlines_to_text(text)),
            Inline::Image { alt, .. } => out.push_str(alt),
            Inline::SoftBreak => out.push(' '),
            Inline::HardBreak => out.push('\n'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_first_block(md: &str) -> Block {
        let blocks = MarkdownParser::parse(md);
        assert!(!blocks.is_empty(), "Expected at least one block from: {:?}", md);
        blocks.into_iter().next().unwrap()
    }

    #[test]
    fn test_parse_h1() {
        let block = parse_first_block("# Hello World");
        match block {
            Block::Heading { level, content, anchor } => {
                assert_eq!(level, 1);
                assert_eq!(inlines_to_text(&content), "Hello World");
                assert_eq!(anchor, "hello-world");
            }
            _ => panic!("Expected heading, got {:?}", block),
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let block = parse_first_block("Hello, world!");
        match block {
            Block::Paragraph(content) => {
                assert_eq!(inlines_to_text(&content), "Hello, world!");
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_table() {
        let md = "| Name | Age |\n|---|---|\n| Alice | 30 |\n| Bob | 25 |";
        let blocks = MarkdownParser::parse(md);
        let table = blocks.into_iter().find(|b| matches!(b, Block::Table { .. }));
        assert!(table.is_some());
        match table.unwrap() {
            Block::Table { headers, rows, .. } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(inlines_to_text(&headers[0]), "Name");
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected Table"),
        }
    }
}
