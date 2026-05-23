use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub id: Uuid,
    pub name: String,
    pub columns: Vec<KanbanColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanColumn {
    pub id: Uuid,
    pub name: String,
    pub wip_limit: Option<usize>,
    pub cards: Vec<KanbanCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanCard {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub assigned_to: Option<String>,
    pub priority: String,
    pub tags: Vec<String>,
    pub column_id: Uuid,
}

impl KanbanBoard {
    pub fn new(name: &str, column_names: &[&str]) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            columns: column_names.iter().map(|n| KanbanColumn {
                id: Uuid::new_v4(),
                name: n.to_string(),
                wip_limit: None,
                cards: Vec::new(),
            }).collect(),
        }
    }

    pub fn default_attendance() -> Self {
        Self::new("Attendance", &["pending", "in_progress", "waiting_client", "resolved"])
    }

    pub fn find_column(&self, column_id: &Uuid) -> Option<&KanbanColumn> {
        self.columns.iter().find(|c| &c.id == column_id)
    }

    pub fn find_column_mut(&mut self, column_id: &Uuid) -> Option<&mut KanbanColumn> {
        self.columns.iter_mut().find(|c| &c.id == column_id)
    }

    pub fn add_card(&mut self, title: &str, description: &str, column_id: &Uuid) -> Option<&KanbanCard> {
        let column = self.find_column_mut(column_id)?;

        if let Some(limit) = column.wip_limit {
            if column.cards.len() >= limit {
                return None;
            }
        }

        let card = KanbanCard {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            assigned_to: None,
            priority: "medium".to_string(),
            tags: Vec::new(),
            column_id: *column_id,
        };
        column.cards.push(card);
        column.cards.last()
    }

    pub fn move_card(&mut self, card_id: &Uuid, target_column_id: &Uuid) -> bool {
        let card = self.find_card_mut(card_id);
        let card = match card {
            Some(c) => c,
            None => return false,
        };

        if &card.column_id == target_column_id {
            return true;
        }

        if let Some(target) = self.find_column_mut(target_column_id) {
            if let Some(limit) = target.wip_limit {
                if target.cards.len() >= limit {
                    return false;
                }
            }
        }

        card.column_id = *target_column_id;
        true
    }

    fn find_card_mut(&mut self, card_id: &Uuid) -> Option<&mut KanbanCard> {
        for col in &mut self.columns {
            if let Some(card) = col.cards.iter_mut().find(|c| &c.id == card_id) {
                return Some(card);
            }
        }
        None
    }

    pub fn delete_card(&mut self, card_id: &Uuid) -> bool {
        for col in &mut self.columns {
            if let Some(pos) = col.cards.iter().position(|c| &c.id == card_id) {
                col.cards.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn cards_by_assignee(&self, agent_id: &str) -> Vec<&KanbanCard> {
        self.columns.iter()
            .flat_map(|c| c.cards.iter())
            .filter(|c| c.assigned_to.as_deref() == Some(agent_id))
            .collect()
    }

    pub fn set_column_wip_limit(&mut self, column_id: &Uuid, limit: usize) -> bool {
        if let Some(col) = self.find_column_mut(column_id) {
            col.wip_limit = Some(limit);
            true
        } else {
            false
        }
    }
}

impl KanbanCard {
    pub fn assign(&mut self, agent_id: &str) {
        self.assigned_to = Some(agent_id.to_string());
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new() {
        let b = KanbanBoard::default_attendance();
        assert_eq!(b.columns.len(), 4);
    }

    #[test]
    fn test_add_and_move_card() {
        let mut b = KanbanBoard::default_attendance();
        let col_id = b.columns[0].id;
        let card = b.add_card("Test Ticket", "Description", &col_id);
        assert!(card.is_some());
        assert_eq!(b.columns[0].cards.len(), 1);

        let card_id = b.columns[0].cards[0].id;
        let target_id = b.columns[1].id;
        assert!(b.move_card(&card_id, &target_id));
        assert_eq!(b.columns[0].cards.len(), 0);
        assert_eq!(b.columns[1].cards.len(), 1);
    }

    #[test]
    fn test_wip_limit() {
        let mut b = KanbanBoard::default_attendance();
        let col_id = b.columns[0].id;
        b.set_column_wip_limit(&col_id, 2);
        assert!(b.add_card("A", "A", &col_id).is_some());
        assert!(b.add_card("B", "B", &col_id).is_some());
        assert!(b.add_card("C", "C", &col_id).is_none());
    }

    #[test]
    fn test_delete_card() {
        let mut b = KanbanBoard::default_attendance();
        let col_id = b.columns[0].id;
        b.add_card("To Delete", "Desc", &col_id);
        let card_id = b.columns[0].cards[0].id;
        assert!(b.delete_card(&card_id));
        assert_eq!(b.columns[0].cards.len(), 0);
    }
}
