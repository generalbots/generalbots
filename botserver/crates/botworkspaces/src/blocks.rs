use super::*;

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
        self.content = BlockContent::Text {
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
        };
        self
    }

    pub fn with_rich_text(mut self, rich_text: RichText) -> Self {
        self.content = BlockContent::Text { text: rich_text };
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

    pub fn with_indent(mut self, level: u32) -> Self {
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
        content: BlockContent::Checklist {
            items: checklist_items,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_toggle(title: &str, expanded: bool, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Toggle,
        content: BlockContent::Toggle {
            title: RichText {
                segments: vec![TextSegment {
                    text: title.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            expanded,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
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
        content: BlockContent::Callout {
            icon: Some(icon.to_string()),
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
        },
        properties: BlockProperties {
            background_color: Some(background.to_string()),
            ..Default::default()
        },
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
        content: BlockContent::Code {
            code: code.to_string(),
            language: Some(language.to_string()),
        },
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
        content: BlockContent::Media {
            url: url.to_string(),
            caption: caption.map(|s| s.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_video(url: &str, caption: Option<&str>, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Video,
        content: BlockContent::Media {
            url: url.to_string(),
            caption: caption.map(|s| s.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_embed(url: &str, embed_type: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Embed,
        content: BlockContent::Embed {
            url: url.to_string(),
            embed_type: Some(embed_type.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_table(rows: usize, cols: usize, created_by: Uuid) -> Block {
    let now = Utc::now();
    let table_rows: Vec<TableRow> = (0..rows)
        .map(|_| TableRow {
            id: Uuid::new_v4(),
            cells: (0..cols)
                .map(|_| TableCell {
                    content: RichText { segments: vec![] },
                    background_color: None,
                })
                .collect(),
        })
        .collect();

    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Table,
        content: BlockContent::Table { rows: table_rows },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_gb_component(
    component_type: &str,
    config: serde_json::Value,
    created_by: Uuid,
) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::GbComponent,
        content: BlockContent::GbComponent {
            component_type: component_type.to_string(),
            config,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}
