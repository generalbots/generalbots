//! Dataset schema for LLM evaluation. Each row is a single test case: a
//! prompt that should be sent to the LLM and a contract describing what a
//! good answer looks like.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFormat {
    Jsonl,
    Csv,
    Yaml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub id: Uuid,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub context: Option<String>,
    pub tags: Vec<String>,
    pub contract: Contract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub description: Option<String>,
    pub format: DatasetFormat,
    pub entries: Vec<DatasetEntry>,
}

impl Dataset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            format: DatasetFormat::Jsonl,
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: DatasetEntry) {
        self.entries.push(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<DatasetEntry> {
        self.entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub json_schema: Option<serde_json::Value>,
    pub max_tokens: Option<u32>,
    pub min_tokens: Option<u32>,
    pub language: Option<String>,
}

impl Contract {
    pub fn must_contain_only(phrases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            must_contain: phrases.into_iter().map(Into::into).collect(),
            must_not_contain: Vec::new(),
            json_schema: None,
            max_tokens: None,
            min_tokens: None,
            language: None,
        }
    }

    pub fn forbid(phrases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            must_contain: Vec::new(),
            must_not_contain: phrases.into_iter().map(Into::into).collect(),
            json_schema: None,
            max_tokens: None,
            min_tokens: None,
            language: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_push_and_filter() {
        let mut ds = Dataset::new("sample");
        ds.push(DatasetEntry {
            id: Uuid::new_v4(),
            prompt: "hello".into(),
            system_prompt: None,
            context: None,
            tags: vec!["greeting".into()],
            contract: Contract::must_contain_only(["olá"]),
        });
        ds.push(DatasetEntry {
            id: Uuid::new_v4(),
            prompt: "what is 2+2".into(),
            system_prompt: None,
            context: None,
            tags: vec!["math".into()],
            contract: Contract::must_contain_only(["4"]),
        });
        let greeting = ds.filter_by_tag("greeting");
        assert_eq!(greeting.len(), 1);
    }
}
