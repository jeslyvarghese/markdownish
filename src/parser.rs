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
/// e.g. "My Heading!" → "my-heading"
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
                // Header cells are directly inside TableHead (no inner TableRow)
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

/// Parse table cells until the given end tag (TableHead or TableRow).
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
            // Tight list items emit bare inline events with no <p> wrapper —
            // collect them into a synthetic paragraph.
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

/// Collect consecutive inline-level events as a paragraph.
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

/// Parse a single inline event and append result to `inlines`.
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

/// Parse inline events until the given end tag is encountered.
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

/// Discriminant-based comparison — handles all TagEnd variants correctly.
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

/// Flatten inlines to plain text.
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
    fn test_parse_h2_through_h6() {
        for n in 2..=6u8 {
            let md = format!("{} Heading {}", "#".repeat(n as usize), n);
            let block = parse_first_block(&md);
            match block {
                Block::Heading { level, .. } => assert_eq!(level, n),
                _ => panic!("Expected heading for H{}", n),
            }
        }
    }

    #[test]
    fn test_heading_anchor_slug() {
        assert_eq!(heading_to_anchor("Hello World"), "hello-world");
        assert_eq!(heading_to_anchor("My Section!"), "my-section");
        assert_eq!(heading_to_anchor("  spaces  "), "spaces");
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
    fn test_parse_bold_text() {
        let block = parse_first_block("**bold text**");
        match block {
            Block::Paragraph(inlines) => {
                assert!(matches!(&inlines[0], Inline::Bold(_)));
                if let Inline::Bold(inner) = &inlines[0] {
                    assert_eq!(inlines_to_text(inner), "bold text");
                }
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_italic_text() {
        let block = parse_first_block("*italic text*");
        match block {
            Block::Paragraph(inlines) => {
                assert!(matches!(&inlines[0], Inline::Italic(_)));
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_strikethrough() {
        let block = parse_first_block("~~struck~~");
        match block {
            Block::Paragraph(inlines) => {
                assert!(matches!(&inlines[0], Inline::Strikethrough(_)));
                if let Inline::Strikethrough(inner) = &inlines[0] {
                    assert_eq!(inlines_to_text(inner), "struck");
                }
            }
            _ => panic!("Expected paragraph with strikethrough"),
        }
    }

    #[test]
    fn test_parse_inline_code() {
        let block = parse_first_block("Use `foo()` function");
        match block {
            Block::Paragraph(inlines) => {
                let has_code = inlines.iter().any(|i| {
                    matches!(i, Inline::Code(c) if c == "foo()")
                });
                assert!(has_code);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_code_block_with_language() {
        let md = "```rust\nfn main() {}\n```";
        let block = parse_first_block(md);
        match block {
            Block::CodeBlock { language, code } => {
                assert_eq!(language, Some("rust".to_string()));
                assert!(code.contains("fn main()"));
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn test_parse_code_block_no_language() {
        let md = "```\nsome code\n```";
        let block = parse_first_block(md);
        match block {
            Block::CodeBlock { language, .. } => {
                assert_eq!(language, None);
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let blocks = MarkdownParser::parse("---");
        let has_hr = blocks.iter().any(|b| matches!(b, Block::HorizontalRule));
        assert!(has_hr);
    }

    #[test]
    fn test_parse_bullet_list() {
        let md = "- item one\n- item two\n- item three";
        let block = parse_first_block(md);
        match block {
            Block::BulletList(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected bullet list"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let md = "1. first\n2. second\n3. third";
        let block = parse_first_block(md);
        match block {
            Block::OrderedList { start, items } => {
                assert_eq!(start, 1);
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected ordered list"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let md = "> This is a quote";
        let block = parse_first_block(md);
        assert!(matches!(block, Block::BlockQuote(_)));
    }

    #[test]
    fn test_parse_link() {
        let md = "[click here](https://example.com)";
        let block = parse_first_block(md);
        match block {
            Block::Paragraph(inlines) => {
                let has_link = inlines.iter().any(|i| {
                    matches!(i, Inline::Link { url, .. } if url == "https://example.com")
                });
                assert!(has_link);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_anchor_link() {
        let md = "[Go to section](#my-section)";
        let block = parse_first_block(md);
        match block {
            Block::Paragraph(inlines) => {
                let has_anchor = inlines.iter().any(|i| {
                    matches!(i, Inline::Link { url, .. } if url == "#my-section")
                });
                assert!(has_anchor);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let md = "# Title\n\nParagraph text.\n\n---\n\n- item";
        let blocks = MarkdownParser::parse(md);
        assert!(blocks.len() >= 3);
    }

    #[test]
    fn test_inlines_to_text_strips_formatting() {
        let inlines = vec![
            Inline::Text("Hello ".to_string()),
            Inline::Bold(vec![Inline::Text("world".to_string())]),
            Inline::Text("!".to_string()),
        ];
        assert_eq!(inlines_to_text(&inlines), "Hello world!");
    }

    #[test]
    fn test_tight_list_items_have_content() {
        let md = "- alpha\n- beta\n- gamma";
        let block = parse_first_block(md);
        match block {
            Block::BulletList(items) => {
                assert_eq!(items.len(), 3);
                for item in &items {
                    let text: String = item.content.iter().map(|b| match b {
                        Block::Paragraph(inlines) => inlines_to_text(inlines),
                        _ => String::new(),
                    }).collect();
                    assert!(!text.is_empty(), "list item should have text content");
                }
            }
            _ => panic!("Expected bullet list"),
        }
    }

    #[test]
    fn test_ordered_list_items_have_content() {
        let md = "1. one\n2. two\n3. three";
        let block = parse_first_block(md);
        match block {
            Block::OrderedList { items, .. } => {
                for item in &items {
                    let text: String = item.content.iter().map(|b| match b {
                        Block::Paragraph(inlines) => inlines_to_text(inlines),
                        _ => String::new(),
                    }).collect();
                    assert!(!text.is_empty());
                }
            }
            _ => panic!("Expected ordered list"),
        }
    }

    #[test]
    fn test_empty_input() {
        let blocks = MarkdownParser::parse("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_nested_emphasis() {
        let md = "***bold italic***";
        let block = parse_first_block(md);
        match block {
            Block::Paragraph(inlines) => {
                let text = inlines_to_text(&inlines);
                assert_eq!(text, "bold italic");
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_inlines_to_text_handles_strikethrough() {
        let inlines = vec![Inline::Strikethrough(vec![Inline::Text("del".to_string())])];
        assert_eq!(inlines_to_text(&inlines), "del");
    }

    #[test]
    fn test_parse_table() {
        let md = "| Name | Age |\n|---|---|\n| Alice | 30 |\n| Bob | 25 |";
        let blocks = MarkdownParser::parse(md);
        let table = blocks.into_iter().find(|b| matches!(b, Block::Table { .. }));
        assert!(table.is_some(), "Expected a Table block");
        match table.unwrap() {
            Block::Table { headers, rows, .. } => {
                assert_eq!(headers.len(), 2, "Expected 2 header columns");
                assert_eq!(inlines_to_text(&headers[0]), "Name");
                assert_eq!(inlines_to_text(&headers[1]), "Age");
                assert_eq!(rows.len(), 2, "Expected 2 data rows");
                assert_eq!(inlines_to_text(&rows[0][0]), "Alice");
                assert_eq!(inlines_to_text(&rows[1][0]), "Bob");
            }
            _ => panic!("Expected Table"),
        }
    }

    #[test]
    fn test_parse_table_with_alignment() {
        let md = "| Left | Center | Right |\n|:---|:---:|---:|\n| a | b | c |";
        let blocks = MarkdownParser::parse(md);
        let table = blocks.into_iter().find(|b| matches!(b, Block::Table { .. }));
        assert!(table.is_some());
        match table.unwrap() {
            Block::Table { alignments, .. } => {
                assert_eq!(alignments[0], ColumnAlign::Left);
                assert_eq!(alignments[1], ColumnAlign::Center);
                assert_eq!(alignments[2], ColumnAlign::Right);
            }
            _ => panic!("Expected Table"),
        }
    }
}
