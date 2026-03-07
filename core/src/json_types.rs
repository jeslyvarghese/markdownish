use serde::Serialize;

use crate::parser::{Block, ColumnAlign, Inline, ListItem};

/// Root document passed to Swift as JSON.
#[derive(Serialize)]
pub struct Document {
    pub version: u32,
    pub blocks: Vec<JsonBlock>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonBlock {
    Heading {
        level: u8,
        anchor: String,
        inlines: Vec<JsonInline>,
    },
    Paragraph {
        inlines: Vec<JsonInline>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    BlockQuote {
        blocks: Vec<JsonBlock>,
    },
    BulletList {
        items: Vec<JsonListItem>,
    },
    OrderedList {
        start: u64,
        items: Vec<JsonListItem>,
    },
    HorizontalRule,
    Table {
        alignments: Vec<JsonAlign>,
        headers: Vec<Vec<JsonInline>>,
        rows: Vec<Vec<Vec<JsonInline>>>,
    },
}

#[derive(Serialize)]
pub struct JsonListItem {
    pub content: Vec<JsonBlock>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonAlign {
    None,
    Left,
    Center,
    Right,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonInline {
    Text { content: String },
    Bold { children: Vec<JsonInline> },
    Italic { children: Vec<JsonInline> },
    Strikethrough { children: Vec<JsonInline> },
    Code { content: String },
    Link { url: String, title: Option<String>, children: Vec<JsonInline> },
    Image { url: String, alt: String, title: Option<String> },
    SoftBreak,
    HardBreak,
}

// --- Conversions ---

impl From<Inline> for JsonInline {
    fn from(inline: Inline) -> Self {
        match inline {
            Inline::Text(s) => JsonInline::Text { content: s },
            Inline::Bold(inner) => JsonInline::Bold { children: inner.into_iter().map(Into::into).collect() },
            Inline::Italic(inner) => JsonInline::Italic { children: inner.into_iter().map(Into::into).collect() },
            Inline::Strikethrough(inner) => JsonInline::Strikethrough { children: inner.into_iter().map(Into::into).collect() },
            Inline::Code(s) => JsonInline::Code { content: s },
            Inline::Link { text, url, title } => JsonInline::Link {
                url,
                title,
                children: text.into_iter().map(Into::into).collect(),
            },
            Inline::Image { alt, url, title } => JsonInline::Image { url, alt, title },
            Inline::SoftBreak => JsonInline::SoftBreak,
            Inline::HardBreak => JsonInline::HardBreak,
        }
    }
}

impl From<ColumnAlign> for JsonAlign {
    fn from(align: ColumnAlign) -> Self {
        match align {
            ColumnAlign::None   => JsonAlign::None,
            ColumnAlign::Left   => JsonAlign::Left,
            ColumnAlign::Center => JsonAlign::Center,
            ColumnAlign::Right  => JsonAlign::Right,
        }
    }
}

impl From<ListItem> for JsonListItem {
    fn from(item: ListItem) -> Self {
        JsonListItem { content: item.content.into_iter().map(Into::into).collect() }
    }
}

impl From<Block> for JsonBlock {
    fn from(block: Block) -> Self {
        match block {
            Block::Heading { level, content, anchor } => JsonBlock::Heading {
                level,
                anchor,
                inlines: content.into_iter().map(Into::into).collect(),
            },
            Block::Paragraph(inlines) => JsonBlock::Paragraph {
                inlines: inlines.into_iter().map(Into::into).collect(),
            },
            Block::CodeBlock { language, code } => JsonBlock::CodeBlock { language, code },
            Block::BlockQuote(blocks) => JsonBlock::BlockQuote {
                blocks: blocks.into_iter().map(Into::into).collect(),
            },
            Block::BulletList(items) => JsonBlock::BulletList {
                items: items.into_iter().map(Into::into).collect(),
            },
            Block::OrderedList { start, items } => JsonBlock::OrderedList {
                start,
                items: items.into_iter().map(Into::into).collect(),
            },
            Block::HorizontalRule => JsonBlock::HorizontalRule,
            Block::Table { alignments, headers, rows } => JsonBlock::Table {
                alignments: alignments.into_iter().map(Into::into).collect(),
                headers: headers.into_iter()
                    .map(|row| row.into_iter().map(Into::into).collect())
                    .collect(),
                rows: rows.into_iter()
                    .map(|row| row.into_iter()
                        .map(|cell| cell.into_iter().map(Into::into).collect())
                        .collect())
                    .collect(),
            },
        }
    }
}

impl Document {
    pub fn from_blocks(blocks: Vec<Block>) -> Self {
        Document {
            version: 1,
            blocks: blocks.into_iter().map(Into::into).collect(),
        }
    }
}
