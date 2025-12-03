//! Knowledge Graph - Entity Relationship Management
//!
//! This module provides knowledge graph capabilities for tracking relationships
//! between entities mentioned in conversations. It enables:
//!
//! - Entity extraction from text
//! - Relationship mapping between entities
//! - Graph queries for complex questions
//! - Integration with RAG for context enrichment
//!
//! ## BASIC Keywords
//!
//! ```basic
//! ' Extract entities from text
//! EXTRACT ENTITIES FROM text INTO KNOWLEDGE GRAPH
//!
//! ' Query the knowledge graph
//! results = QUERY GRAPH "people who work on Project Alpha"
//!
//! ' Add entity manually
//! ADD ENTITY "John Smith" TYPE "person" WITH {"department": "Sales"}
//!
//! ' Add relationship
//! ADD RELATIONSHIP "John Smith" -> "works_on" -> "Project Alpha"
//!
//! ' Get entity details
//! entity = GET ENTITY "John Smith"
//!
//! ' Find related entities
//! related = GET RELATED "Project Alpha" BY "works_on"
//!
//! ' Delete entity
//! DELETE ENTITY "John Smith"
//! ```
//!
//! ## Config.csv Properties
//!
//! ```csv
//! name,value
//! knowledge-graph-enabled,true
//! knowledge-graph-backend,postgresql
//! knowledge-graph-extract-entities,true
//! knowledge-graph-extraction-model,quality
//! knowledge-graph-max-entities,10000
//! knowledge-graph-max-relationships,50000
//! ```

use chrono::{DateTime, Utc};
use rhai::{Array, Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// Entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEntity {
    /// Unique identifier
    pub id: Uuid,
    /// Bot ID this entity belongs to
    pub bot_id: Uuid,
    /// Entity type (person, organization, project, product, etc.)
    pub entity_type: String,
    /// Entity name (canonical form)
    pub entity_name: String,
    /// Alternative names/aliases
    pub aliases: Vec<String>,
    /// Entity properties
    pub properties: serde_json::Value,
    /// Confidence score (0-1) if extracted automatically
    pub confidence: f64,
    /// Source of the entity (manual, extracted, imported)
    pub source: EntitySource,
    /// When the entity was created
    pub created_at: DateTime<Utc>,
    /// When the entity was last updated
    pub updated_at: DateTime<Utc>,
}

/// Source of entity creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntitySource {
    Manual,
    Extracted,
    Imported,
    Inferred,
}

impl Default for EntitySource {
    fn default() -> Self {
        EntitySource::Manual
    }
}

/// Relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgRelationship {
    /// Unique identifier
    pub id: Uuid,
    /// Bot ID this relationship belongs to
    pub bot_id: Uuid,
    /// Source entity ID
    pub from_entity_id: Uuid,
    /// Target entity ID
    pub to_entity_id: Uuid,
    /// Relationship type (works_on, reports_to, owns, etc.)
    pub relationship_type: String,
    /// Relationship properties (strength, since, etc.)
    pub properties: serde_json::Value,
    /// Confidence score (0-1) if extracted automatically
    pub confidence: f64,
    /// Whether this is a bidirectional relationship
    pub bidirectional: bool,
    /// Source of the relationship
    pub source: EntitySource,
    /// When the relationship was created
    pub created_at: DateTime<Utc>,
}

/// Entity extraction result from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity name as found in text
    pub name: String,
    /// Normalized/canonical name
    pub canonical_name: String,
    /// Entity type
    pub entity_type: String,
    /// Start position in text
    pub start_pos: usize,
    /// End position in text
    pub end_pos: usize,
    /// Confidence score
    pub confidence: f64,
    /// Additional properties extracted
    pub properties: serde_json::Value,
}

/// Extracted relationship from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    /// Source entity name
    pub from_entity: String,
    /// Target entity name
    pub to_entity: String,
    /// Relationship type
    pub relationship_type: String,
    /// Confidence score
    pub confidence: f64,
    /// Supporting text snippet
    pub evidence: String,
}

/// Knowledge graph extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Extracted relationships
    pub relationships: Vec<ExtractedRelationship>,
    /// Processing metadata
    pub metadata: ExtractionMetadata,
}

/// Metadata about the extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    /// Model used for extraction
    pub model: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of tokens processed
    pub tokens_processed: usize,
    /// Source text length
    pub text_length: usize,
}

/// Query result from the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    /// Matching entities
    pub entities: Vec<KgEntity>,
    /// Relationships between matched entities
    pub relationships: Vec<KgRelationship>,
    /// Query explanation
    pub explanation: String,
    /// Confidence in the result
    pub confidence: f64,
}

/// Configuration for knowledge graph
#[derive(Debug, Clone)]
pub struct KnowledgeGraphConfig {
    /// Whether knowledge graph is enabled
    pub enabled: bool,
    /// Backend storage (postgresql, neo4j, etc.)
    pub backend: String,
    /// Whether to auto-extract entities from conversations
    pub extract_entities: bool,
    /// Model to use for entity extraction
    pub extraction_model: String,
    /// Maximum entities per bot
    pub max_entities: usize,
    /// Maximum relationships per bot
    pub max_relationships: usize,
    /// Minimum confidence threshold for extraction
    pub min_confidence: f64,
    /// Entity types to extract
    pub entity_types: Vec<String>,
}

impl Default for KnowledgeGraphConfig {
    fn default() -> Self {
        KnowledgeGraphConfig {
            enabled: true,
            backend: "postgresql".to_string(),
            extract_entities: true,
            extraction_model: "quality".to_string(),
            max_entities: 10000,
            max_relationships: 50000,
            min_confidence: 0.7,
            entity_types: vec![
                "person".to_string(),
                "organization".to_string(),
                "project".to_string(),
                "product".to_string(),
                "location".to_string(),
                "event".to_string(),
                "concept".to_string(),
            ],
        }
    }
}

/// Knowledge Graph Manager
#[derive(Debug)]
pub struct KnowledgeGraphManager {
    config: KnowledgeGraphConfig,
}

impl KnowledgeGraphManager {
    /// Create a new knowledge graph manager
    pub fn new(config: KnowledgeGraphConfig) -> Self {
        KnowledgeGraphManager { config }
    }

    /// Create from config map
    pub fn from_config(config_map: &HashMap<String, String>) -> Self {
        let config = KnowledgeGraphConfig {
            enabled: config_map
                .get("knowledge-graph-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            backend: config_map
                .get("knowledge-graph-backend")
                .cloned()
                .unwrap_or_else(|| "postgresql".to_string()),
            extract_entities: config_map
                .get("knowledge-graph-extract-entities")
                .map(|v| v == "true")
                .unwrap_or(true),
            extraction_model: config_map
                .get("knowledge-graph-extraction-model")
                .cloned()
                .unwrap_or_else(|| "quality".to_string()),
            max_entities: config_map
                .get("knowledge-graph-max-entities")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10000),
            max_relationships: config_map
                .get("knowledge-graph-max-relationships")
                .and_then(|v| v.parse().ok())
                .unwrap_or(50000),
            min_confidence: config_map
                .get("knowledge-graph-min-confidence")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            entity_types: config_map
                .get("knowledge-graph-entity-types")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| KnowledgeGraphConfig::default().entity_types),
        };
        KnowledgeGraphManager::new(config)
    }

    /// Generate entity extraction prompt
    pub fn generate_extraction_prompt(&self, text: &str) -> String {
        let entity_types = self.config.entity_types.join(", ");

        format!(
            r#"Extract entities and relationships from the following text.

ENTITY TYPES TO EXTRACT: {entity_types}

TEXT:
{text}

Respond with valid JSON only:
{{
    "entities": [
        {{
            "name": "exact name as in text",
            "canonical_name": "normalized name",
            "entity_type": "person|organization|project|product|location|event|concept",
            "confidence": 0.95,
            "properties": {{"key": "value"}}
        }}
    ],
    "relationships": [
        {{
            "from_entity": "entity name",
            "to_entity": "entity name",
            "relationship_type": "works_on|reports_to|owns|part_of|located_in|related_to",
            "confidence": 0.9,
            "evidence": "text snippet supporting this relationship"
        }}
    ]
}}"#
        )
    }

    /// Generate graph query prompt
    pub fn generate_query_prompt(&self, query: &str, context: &str) -> String {
        format!(
            r#"Answer this question using the knowledge graph context.

QUESTION: {query}

KNOWLEDGE GRAPH CONTEXT:
{context}

Provide a natural language answer based on the entities and relationships.
If the information is not available, say so clearly.
"#
        )
    }

    /// Parse extraction response from LLM
    pub fn parse_extraction_response(
        &self,
        response: &str,
        text_length: usize,
        processing_time_ms: u64,
    ) -> Result<ExtractionResult, String> {
        let json_str = extract_json(response)?;

        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let entities: Vec<ExtractedEntity> = parsed["entities"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(ExtractedEntity {
                            name: v["name"].as_str()?.to_string(),
                            canonical_name: v["canonical_name"]
                                .as_str()
                                .unwrap_or(v["name"].as_str()?)
                                .to_string(),
                            entity_type: v["entity_type"].as_str()?.to_string(),
                            start_pos: 0,
                            end_pos: 0,
                            confidence: v["confidence"].as_f64().unwrap_or(0.8),
                            properties: v["properties"].clone(),
                        })
                    })
                    .filter(|e| e.confidence >= self.config.min_confidence)
                    .collect()
            })
            .unwrap_or_default();

        let relationships: Vec<ExtractedRelationship> = parsed["relationships"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        Some(ExtractedRelationship {
                            from_entity: v["from_entity"].as_str()?.to_string(),
                            to_entity: v["to_entity"].as_str()?.to_string(),
                            relationship_type: v["relationship_type"].as_str()?.to_string(),
                            confidence: v["confidence"].as_f64().unwrap_or(0.8),
                            evidence: v["evidence"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .filter(|r| r.confidence >= self.config.min_confidence)
                    .collect()
            })
            .unwrap_or_default();

        Ok(ExtractionResult {
            entities,
            relationships,
            metadata: ExtractionMetadata {
                model: self.config.extraction_model.clone(),
                processing_time_ms,
                tokens_processed: text_length / 4, // Rough estimate
                text_length,
            },
        })
    }

    /// Check if extraction should run
    pub fn should_extract(&self) -> bool {
        self.config.enabled && self.config.extract_entities
    }

    /// Validate entity type
    pub fn is_valid_entity_type(&self, entity_type: &str) -> bool {
        self.config
            .entity_types
            .iter()
            .any(|t| t.eq_ignore_ascii_case(entity_type))
    }
}

/// Extract JSON from LLM response
fn extract_json(response: &str) -> Result<String, String> {
    // Try to find JSON in code blocks first
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start + 7..].find("```") {
            return Ok(response[start + 7..start + 7 + end].trim().to_string());
        }
    }

    // Try to find JSON in generic code blocks
    if let Some(start) = response.find("```") {
        let after_start = start + 3;
        let json_start = response[after_start..]
            .find('\n')
            .map(|i| after_start + i + 1)
            .unwrap_or(after_start);
        if let Some(end) = response[json_start..].find("```") {
            return Ok(response[json_start..json_start + end].trim().to_string());
        }
    }

    // Try to find raw JSON
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err("No JSON found in response".to_string())
}

/// Convert KgEntity to Rhai Dynamic
impl KgEntity {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("id".into(), self.id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert("entity_type".into(), self.entity_type.clone().into());
        map.insert("entity_name".into(), self.entity_name.clone().into());

        let aliases: Array = self
            .aliases
            .iter()
            .map(|a| Dynamic::from(a.clone()))
            .collect();
        map.insert("aliases".into(), aliases.into());

        map.insert("properties".into(), json_to_dynamic(&self.properties));
        map.insert("confidence".into(), self.confidence.into());
        map.insert(
            "source".into(),
            format!("{:?}", self.source).to_lowercase().into(),
        );
        map.insert("created_at".into(), self.created_at.to_rfc3339().into());
        map.insert("updated_at".into(), self.updated_at.to_rfc3339().into());

        Dynamic::from(map)
    }
}

/// Convert KgRelationship to Rhai Dynamic
impl KgRelationship {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("id".into(), self.id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert(
            "from_entity_id".into(),
            self.from_entity_id.to_string().into(),
        );
        map.insert("to_entity_id".into(), self.to_entity_id.to_string().into());
        map.insert(
            "relationship_type".into(),
            self.relationship_type.clone().into(),
        );
        map.insert("properties".into(), json_to_dynamic(&self.properties));
        map.insert("confidence".into(), self.confidence.into());
        map.insert("bidirectional".into(), self.bidirectional.into());
        map.insert(
            "source".into(),
            format!("{:?}", self.source).to_lowercase().into(),
        );
        map.insert("created_at".into(), self.created_at.to_rfc3339().into());

        Dynamic::from(map)
    }
}

/// Convert JSON value to Rhai Dynamic
fn json_to_dynamic(value: &serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let array: Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(array)
        }
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

/// Register knowledge graph keywords with Rhai engine
pub fn register_knowledge_graph_keywords(engine: &mut Engine) {
    // Helper functions for working with entities in scripts

    engine.register_fn("entity_name", |entity: Map| -> String {
        entity
            .get("entity_name")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    engine.register_fn("entity_type", |entity: Map| -> String {
        entity
            .get("entity_type")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    engine.register_fn("entity_properties", |entity: Map| -> Map {
        entity
            .get("properties")
            .and_then(|v| v.clone().try_cast::<Map>())
            .unwrap_or_default()
    });

    engine.register_fn("relationship_type", |rel: Map| -> String {
        rel.get("relationship_type")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    engine.register_fn("is_bidirectional", |rel: Map| -> bool {
        rel.get("bidirectional")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false)
    });

    info!("Knowledge graph keywords registered");
}

/// SQL for creating knowledge graph tables
pub const KNOWLEDGE_GRAPH_SCHEMA: &str = r#"
-- Knowledge graph entities
CREATE TABLE IF NOT EXISTS kg_entities (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_name VARCHAR(500) NOT NULL,
    aliases JSONB NOT NULL DEFAULT '[]',
    properties JSONB NOT NULL DEFAULT '{}',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, entity_type, entity_name)
);

-- Knowledge graph relationships
CREATE TABLE IF NOT EXISTS kg_relationships (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    from_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    bidirectional BOOLEAN NOT NULL DEFAULT false,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, from_entity_id, to_entity_id, relationship_type)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_kg_entities_bot_id ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_lower ON kg_entities(LOWER(entity_name));
CREATE INDEX IF NOT EXISTS idx_kg_entities_aliases ON kg_entities USING GIN(aliases);

CREATE INDEX IF NOT EXISTS idx_kg_relationships_bot_id ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_type ON kg_relationships(relationship_type);

-- Full-text search on entity names
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_fts ON kg_entities
    USING GIN(to_tsvector('english', entity_name));
"#;

/// SQL for knowledge graph operations
pub mod sql {
    pub const INSERT_ENTITY: &str = r#"
        INSERT INTO kg_entities (
            id, bot_id, entity_type, entity_name, aliases, properties,
            confidence, source, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )
        ON CONFLICT (bot_id, entity_type, entity_name)
        DO UPDATE SET
            aliases = kg_entities.aliases || $5,
            properties = kg_entities.properties || $6,
            confidence = GREATEST(kg_entities.confidence, $7),
            updated_at = $10
        RETURNING id
    "#;

    pub const INSERT_RELATIONSHIP: &str = r#"
        INSERT INTO kg_relationships (
            id, bot_id, from_entity_id, to_entity_id, relationship_type,
            properties, confidence, bidirectional, source, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
        )
        ON CONFLICT (bot_id, from_entity_id, to_entity_id, relationship_type)
        DO UPDATE SET
            properties = kg_relationships.properties || $6,
            confidence = GREATEST(kg_relationships.confidence, $7)
        RETURNING id
    "#;

    pub const GET_ENTITY_BY_NAME: &str = r#"
        SELECT * FROM kg_entities
        WHERE bot_id = $1
        AND (
            LOWER(entity_name) = LOWER($2)
            OR aliases @> $3::jsonb
        )
        LIMIT 1
    "#;

    pub const GET_ENTITY_BY_ID: &str = r#"
        SELECT * FROM kg_entities WHERE id = $1
    "#;

    pub const SEARCH_ENTITIES: &str = r#"
        SELECT * FROM kg_entities
        WHERE bot_id = $1
        AND (
            to_tsvector('english', entity_name) @@ plainto_tsquery('english', $2)
            OR LOWER(entity_name) LIKE LOWER($3)
        )
        ORDER BY confidence DESC
        LIMIT $4
    "#;

    pub const GET_ENTITIES_BY_TYPE: &str = r#"
        SELECT * FROM kg_entities
        WHERE bot_id = $1 AND entity_type = $2
        ORDER BY entity_name
        LIMIT $3
    "#;

    pub const GET_RELATED_ENTITIES: &str = r#"
        SELECT e.*, r.relationship_type, r.confidence as rel_confidence
        FROM kg_entities e
        JOIN kg_relationships r ON (
            (r.from_entity_id = $1 AND r.to_entity_id = e.id)
            OR (r.bidirectional AND r.to_entity_id = $1 AND r.from_entity_id = e.id)
        )
        WHERE r.bot_id = $2
        ORDER BY r.confidence DESC
        LIMIT $3
    "#;

    pub const GET_RELATED_BY_TYPE: &str = r#"
        SELECT e.*, r.relationship_type, r.confidence as rel_confidence
        FROM kg_entities e
        JOIN kg_relationships r ON (
            (r.from_entity_id = $1 AND r.to_entity_id = e.id)
            OR (r.bidirectional AND r.to_entity_id = $1 AND r.from_entity_id = e.id)
        )
        WHERE r.bot_id = $2 AND r.relationship_type = $3
        ORDER BY r.confidence DESC
        LIMIT $4
    "#;

    pub const GET_RELATIONSHIP: &str = r#"
        SELECT * FROM kg_relationships
        WHERE bot_id = $1
        AND from_entity_id = $2
        AND to_entity_id = $3
        AND relationship_type = $4
    "#;

    pub const GET_ALL_RELATIONSHIPS_FOR_ENTITY: &str = r#"
        SELECT r.*,
               e1.entity_name as from_name, e1.entity_type as from_type,
               e2.entity_name as to_name, e2.entity_type as to_type
        FROM kg_relationships r
        JOIN kg_entities e1 ON r.from_entity_id = e1.id
        JOIN kg_entities e2 ON r.to_entity_id = e2.id
        WHERE r.bot_id = $1
        AND (r.from_entity_id = $2 OR r.to_entity_id = $2)
        ORDER BY r.confidence DESC
    "#;

    pub const DELETE_ENTITY: &str = r#"
        DELETE FROM kg_entities WHERE id = $1 AND bot_id = $2
    "#;

    pub const DELETE_RELATIONSHIP: &str = r#"
        DELETE FROM kg_relationships WHERE id = $1 AND bot_id = $2
    "#;

    pub const COUNT_ENTITIES: &str = r#"
        SELECT COUNT(*) FROM kg_entities WHERE bot_id = $1
    "#;

    pub const COUNT_RELATIONSHIPS: &str = r#"
        SELECT COUNT(*) FROM kg_relationships WHERE bot_id = $1
    "#;

    pub const GET_ENTITY_TYPES: &str = r#"
        SELECT DISTINCT entity_type, COUNT(*) as count
        FROM kg_entities
        WHERE bot_id = $1
        GROUP BY entity_type
        ORDER BY count DESC
    "#;

    pub const GET_RELATIONSHIP_TYPES: &str = r#"
        SELECT DISTINCT relationship_type, COUNT(*) as count
        FROM kg_relationships
        WHERE bot_id = $1
        GROUP BY relationship_type
        ORDER BY count DESC
    "#;

    /// Graph traversal query (find path between two entities)
    pub const FIND_PATH: &str = r#"
        WITH RECURSIVE path_finder AS (
            -- Base case: start from source entity
            SELECT
                from_entity_id,
                to_entity_id,
                relationship_type,
                ARRAY[from_entity_id] as path,
                1 as depth
            FROM kg_relationships
            WHERE bot_id = $1 AND from_entity_id = $2

            UNION ALL

            -- Recursive case: follow relationships
            SELECT
                r.from_entity_id,
                r.to_entity_id,
                r.relationship_type,
                pf.path || r.from_entity_id,
                pf.depth + 1
            FROM kg_relationships r
            JOIN path_finder pf ON r.from_entity_id = pf.to_entity_id
            WHERE r.bot_id = $1
            AND NOT r.from_entity_id = ANY(pf.path)  -- Prevent cycles
            AND pf.depth < $4  -- Max depth
        )
        SELECT * FROM path_finder
        WHERE to_entity_id = $3
        ORDER BY depth
        LIMIT 1
    "#;
}

/// Common relationship types
pub mod relationship_types {
    pub const WORKS_ON: &str = "works_on";
    pub const REPORTS_TO: &str = "reports_to";
    pub const MANAGES: &str = "manages";
    pub const OWNS: &str = "owns";
    pub const PART_OF: &str = "part_of";
    pub const LOCATED_IN: &str = "located_in";
    pub const RELATED_TO: &str = "related_to";
    pub const CREATED_BY: &str = "created_by";
    pub const DEPENDS_ON: &str = "depends_on";
    pub const CONNECTED_TO: &str = "connected_to";
    pub const MEMBER_OF: &str = "member_of";
    pub const SUCCESSOR_OF: &str = "successor_of";
    pub const PREDECESSOR_OF: &str = "predecessor_of";
    pub const ALIAS_OF: &str = "alias_of";
}

/// Common entity types
pub mod entity_types {
    pub const PERSON: &str = "person";
    pub const ORGANIZATION: &str = "organization";
    pub const PROJECT: &str = "project";
    pub const PRODUCT: &str = "product";
    pub const LOCATION: &str = "location";
    pub const EVENT: &str = "event";
    pub const CONCEPT: &str = "concept";
    pub const DOCUMENT: &str = "document";
    pub const TEAM: &str = "team";
    pub const ROLE: &str = "role";
    pub const SKILL: &str = "skill";
    pub const TECHNOLOGY: &str = "technology";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KnowledgeGraphConfig::default();
        assert!(config.enabled);
        assert_eq!(config.backend, "postgresql");
        assert!(config.entity_types.contains(&"person".to_string()));
    }

    #[test]
    fn test_extraction_prompt() {
        let manager = KnowledgeGraphManager::new(KnowledgeGraphConfig::default());
        let prompt = manager.generate_extraction_prompt("John works at Acme Corp.");
        assert!(prompt.contains("John works at Acme Corp."));
        assert!(prompt.contains("ENTITY TYPES TO EXTRACT"));
    }

    #[test]
    fn test_parse_extraction_response() {
        let manager = KnowledgeGraphManager::new(KnowledgeGraphConfig::default());
        let response = r#"{
            "entities": [
                {
                    "name": "John",
                    "canonical_name": "John Smith",
                    "entity_type": "person",
                    "confidence": 0.9,
                    "properties": {}
                }
            ],
            "relationships": [
                {
                    "from_entity": "John",
                    "to_entity": "Acme Corp",
                    "relationship_type": "works_on",
                    "confidence": 0.85,
                    "evidence": "John works at Acme Corp"
                }
            ]
        }"#;

        let result = manager.parse_extraction_response(response, 100, 50);
        assert!(result.is_ok());
        let extraction = result.unwrap();
        assert_eq!(extraction.entities.len(), 1);
        assert_eq!(extraction.relationships.len(), 1);
    }

    #[test]
    fn test_entity_to_dynamic() {
        let entity = KgEntity {
            id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            entity_type: "person".to_string(),
            entity_name: "John Smith".to_string(),
            aliases: vec!["John".to_string()],
            properties: serde_json::json!({"department": "Sales"}),
            confidence: 0.95,
            source: EntitySource::Manual,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dynamic = entity.to_dynamic();
        assert!(dynamic.is::<Map>());
    }

    #[test]
    fn test_is_valid_entity_type() {
        let manager = KnowledgeGraphManager::new(KnowledgeGraphConfig::default());
        assert!(manager.is_valid_entity_type("person"));
        assert!(manager.is_valid_entity_type("PERSON"));
        assert!(manager.is_valid_entity_type("organization"));
        assert!(!manager.is_valid_entity_type("unknown_type"));
    }

    #[test]
    fn test_json_to_dynamic() {
        let json = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a", "b"]
        });

        let dynamic = json_to_dynamic(&json);
        assert!(dynamic.is::<Map>());
    }
}
