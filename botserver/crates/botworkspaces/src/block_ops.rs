use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockOperation {
    pub operation_type: BlockOperationType,
    pub block_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub position: Option<usize>,
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
    Duplicate,
}

pub fn apply_block_operations(blocks: &mut Vec<Block>, operations: Vec<BlockOperation>) {
    for op in operations {
        match op.operation_type {
            BlockOperationType::Insert => {
                if let Some(block) = op.block {
                    let position = op.position.unwrap_or(blocks.len());
                    if position <= blocks.len() {
                        blocks.insert(position, block);
                    }
                }
            }
            BlockOperationType::Update => {
                if let Some(block_id) = op.block_id {
                    if let Some(block) = find_block_mut(blocks, block_id) {
                        if let Some(content) = op.content {
                            block.content = content;
                        }
                        if let Some(props) = op.properties {
                            block.properties = props;
                        }
                        block.updated_at = Utc::now();
                    }
                }
            }
            BlockOperationType::Delete => {
                if let Some(block_id) = op.block_id {
                    remove_block(blocks, block_id);
                }
            }
            BlockOperationType::Move => {
                if let Some(block_id) = op.block_id {
                    if let Some(position) = op.position {
                        if let Some(block) = remove_block(blocks, block_id) {
                            let insert_pos = position.min(blocks.len());
                            blocks.insert(insert_pos, block);
                        }
                    }
                }
            }
            BlockOperationType::Duplicate => {
                if let Some(block_id) = op.block_id {
                    if let Some(block) = find_block(blocks, block_id) {
                        let mut new_block = block.clone();
                        new_block.id = Uuid::new_v4();
                        new_block.created_at = Utc::now();
                        new_block.updated_at = Utc::now();
                        let position = op.position.unwrap_or(blocks.len());
                        blocks.insert(position.min(blocks.len()), new_block);
                    }
                }
            }
        }
    }
}

pub(crate) fn find_block(blocks: &[Block], block_id: Uuid) -> Option<&Block> {
    for block in blocks {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block(&block.children, block_id) {
            return Some(found);
        }
    }
    None
}

pub(crate) fn find_block_mut(blocks: &mut [Block], block_id: Uuid) -> Option<&mut Block> {
    for block in blocks.iter_mut() {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block_mut(&mut block.children, block_id) {
            return Some(found);
        }
    }
    None
}

pub(crate) fn remove_block(blocks: &mut Vec<Block>, block_id: Uuid) -> Option<Block> {
    if let Some(pos) = blocks.iter().position(|b| b.id == block_id) {
        return Some(blocks.remove(pos));
    }

    for block in blocks.iter_mut() {
        if let Some(removed) = remove_block(&mut block.children, block_id) {
            return Some(removed);
        }
    }

    None
}

