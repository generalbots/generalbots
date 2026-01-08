use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Block, BlockContent, BlockProperties, BlockType, CalloutContent, ChecklistContent,
    ChecklistItem, CodeContent, EmbedContent, EmbedType, GbComponentContent, GbComponentType,
    MediaContent, RichText, TableCell, TableContent, TableRow, TextAnnotations, TextSegment,
    ToggleContent, WorkspaceIcon,
};

pub struct BlockBuilder {
    block_type: BlockType,
    content: BlockContent,
    properties: BlockProperties,
    children: Vec<Block>,
    created_by: Uuid,
}

impl BlockBuilder {
    pub fn new(block_type: BlockType, created_by: Uuid) -> Self {
        Self {
            block_type,
            content: BlockContent::Empty,
            properties: BlockProperties::default(),
            children: Vec::new(),
            created_by,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.content = BlockContent::Text(RichText {
            segments: vec![TextSegment {
                text: text.to_string(),
                annotations: TextAnnotations::default(),
                link: None,
                mention: None,
            }],
        });
        self
    }

    pub fn with_rich_text(mut self, rich_text: RichText) -> Self {
        self.content = BlockContent::Text(rich_text);
        self
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.properties.color = Some(color.to_string());
        self
    }

    pub fn with_background(mut self, color: &str) -> Self {
        self.properties.background_color = Some(color.to_string());
        self
    }

    pub fn with_indent(mut self, level: u8) -> Self {
        self.properties.indent_level = level;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.properties.collapsed = collapsed;
        self
    }

    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.children = children;
        self
    }

    pub fn build(self) -> Block {
        let now = Utc::now();
        Block {
            id: Uuid::new_v4(),
            block_type: self.block_type,
            content: self.content,
            properties: self.properties,
            children: self.children,
            created_at: now,
            updated_at: now,
            created_by: self.created_by,
        }
    }
}

pub fn create_paragraph(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Paragraph, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading1(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading1, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading2(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading2, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading3(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading3, created_by)
        .with_text(text)
        .build()
}

pub fn create_bulleted_list_item(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::BulletedList, created_by)
        .with_text(text)
        .build()
}

pub fn create_numbered_list_item(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::NumberedList, created_by)
        .with_text(text)
        .build()
}

pub fn create_checklist(items: Vec<(&str, bool)>, created_by: Uuid) -> Block {
    let checklist_items: Vec<ChecklistItem> = items
        .into_iter()
        .map(|(text, checked)| ChecklistItem {
            id: Uuid::new_v4(),
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            checked,
            assignee: None,
            due_date: None,
        })
        .collect();

    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Checklist,
        content: BlockContent::Checklist(ChecklistContent {
            items: checklist_items,
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_toggle(title: &str, expanded: bool, children: Vec<Block>, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Toggle,
        content: BlockContent::Toggle(ToggleContent {
            title: RichText {
                segments: vec![TextSegment {
                    text: title.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            expanded,
        }),
        properties: BlockProperties::default(),
        children,
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_quote(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Quote, created_by)
        .with_text(text)
        .build()
}

pub fn create_callout(icon: &str, text: &str, background: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Callout,
        content: BlockContent::Callout(CalloutContent {
            icon: WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: icon.to_string(),
            },
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            background_color: background.to_string(),
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_divider(created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Divider,
        content: BlockContent::Empty,
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_code(code: &str, language: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Code,
        content: BlockContent::Code(CodeContent {
            code: code.to_string(),
            language: language.to_string(),
            caption: None,
            wrap: false,
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_table(rows: usize, cols: usize, has_header: bool, created_by: Uuid) -> Block {
    let table_rows: Vec<TableRow> = (0..rows)
        .map(|_| TableRow {
            id: Uuid::new_v4(),
            cells: (0..cols)
                .map(|_| TableCell {
                    content: RichText { segments: Vec::new() },
                    background_color: None,
                })
                .collect(),
        })
        .collect();

    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Table,
        content: BlockContent::Table(TableContent {
            rows: table_rows,
            has_header_row: has_header,
            has_header_column: false,
            column_widths: vec![200; cols],
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_image(url: &str, caption: Option<&str>, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Image,
        content: BlockContent::Media(MediaContent {
            url: url.to_string(),
            caption: caption.map(|c| RichText {
                segments: vec![TextSegment {
                    text: c.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            }),
            alt_text: None,
            width: None,
            height: None,
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_embed(url: &str, embed_type: EmbedType, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Embed,
        content: BlockContent::Embed(EmbedContent {
            url: url.to_string(),
            embed_type,
            caption: None,
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_gb_component(
    component_type: GbComponentType,
    bot_id: Option<Uuid>,
    created_by: Uuid,
) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::GbComponent,
        content: BlockContent::GbComponent(GbComponentContent {
            component_type,
            bot_id,
            config: std::collections::HashMap::new(),
        }),
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockOperation {
    pub operation_type: BlockOperationType,
    pub block_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub index: Option<usize>,
    pub block: Option<Block>,
    pub properties: Option<BlockProperties>,
    pub content: Option<BlockContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockOperationType {
    Insert,
    Update,
    Delete,
    Move,
    UpdateProperties,
    UpdateContent,
}

pub fn apply_block_operation(blocks: &mut Vec<Block>, operation: BlockOperation) -> Result<(), String> {
    match operation.operation_type {
        BlockOperationType::Insert => {
            let block = operation.block.ok_or("Block required for insert")?;
            let index = operation.index.unwrap_or(blocks.len());
            if index > blocks.len() {
                blocks.push(block);
            } else {
                blocks.insert(index, block);
            }
        }
        BlockOperationType::Update => {
            let new_block = operation.block.ok_or("Block required for update")?;
            if let Some(block) = blocks.iter_mut().find(|b| b.id == operation.block_id) {
                *block = new_block;
            } else {
                return Err("Block not found".to_string());
            }
        }
        BlockOperationType::Delete => {
            blocks.retain(|b| b.id != operation.block_id);
        }
        BlockOperationType::Move => {
            let index = operation.index.ok_or("Index required for move")?;
            if let Some(pos) = blocks.iter().position(|b| b.id == operation.block_id) {
                let block = blocks.remove(pos);
                let new_index = if index > pos { index - 1 } else { index };
                if new_index >= blocks.len() {
                    blocks.push(block);
                } else {
                    blocks.insert(new_index, block);
                }
            } else {
                return Err("Block not found".to_string());
            }
        }
        BlockOperationType::UpdateProperties => {
            let props = operation.properties.ok_or("Properties required")?;
            if let Some(block) = blocks.iter_mut().find(|b| b.id == operation.block_id) {
                block.properties = props;
                block.updated_at = Utc::now();
            } else {
                return Err("Block not found".to_string());
            }
        }
        BlockOperationType::UpdateContent => {
            let content = operation.content.ok_or("Content required")?;
            if let Some(block) = blocks.iter_mut().find(|b| b.id == operation.block_id) {
                block.content = content;
                block.updated_at = Utc::now();
            } else {
                return Err("Block not found".to_string());
            }
        }
    }
    Ok(())
}

pub fn blocks_to_plain_text(blocks: &[Block]) -> String {
    let mut result = String::new();
    for block in blocks {
        if let BlockContent::Text(rich_text) = &block.content {
            for segment in &rich_text.segments {
                result.push_str(&segment.text);
            }
            result.push('\n');
        }
        if !block.children.is_empty() {
            result.push_str(&blocks_to_plain_text(&block.children));
        }
    }
    result
}

pub fn blocks_to_markdown(blocks: &[Block], indent: usize) -> String {
    let mut result = String::new();
    let prefix = "  ".repeat(indent);

    for block in blocks {
        match block.block_type {
            BlockType::Heading1 => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}# {}\n\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::Heading2 => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}## {}\n\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::Heading3 => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}### {}\n\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::Paragraph => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}{}\n\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::BulletedList => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}- {}\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::NumberedList => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}1. {}\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::Quote => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}> {}\n\n", prefix, rich_text_to_string(rt)));
                }
            }
            BlockType::Code => {
                if let BlockContent::Code(code) = &block.content {
                    result.push_str(&format!("{}```{}\n{}\n{}```\n\n", prefix, code.language, code.code, prefix));
                }
            }
            BlockType::Divider => {
                result.push_str(&format!("{}---\n\n", prefix));
            }
            BlockType::Checklist => {
                if let BlockContent::Checklist(cl) = &block.content {
                    for item in &cl.items {
                        let checkbox = if item.checked { "[x]" } else { "[ ]" };
                        result.push_str(&format!("{}- {} {}\n", prefix, checkbox, rich_text_to_string(&item.text)));
                    }
                    result.push('\n');
                }
            }
            _ => {
                if let BlockContent::Text(rt) = &block.content {
                    result.push_str(&format!("{}{}\n", prefix, rich_text_to_string(rt)));
                }
            }
        }

        if !block.children.is_empty() {
            result.push_str(&blocks_to_markdown(&block.children, indent + 1));
        }
    }

    result
}

fn rich_text_to_string(rich_text: &RichText) -> String {
    rich_text.segments.iter().map(|s| s.text.as_str()).collect()
}

pub fn find_block_by_id(blocks: &[Block], block_id: Uuid) -> Option<&Block> {
    for block in blocks {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block_by_id(&block.children, block_id) {
            return Some(found);
        }
    }
    None
}

pub fn find_block_by_id_mut(blocks: &mut [Block], block_id: Uuid) -> Option<&mut Block> {
    for block in blocks {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block_by_id_mut(&mut block.children, block_id) {
            return Some(found);
        }
    }
    None
}
