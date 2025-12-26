use chrono::{DateTime, Utc};
use rhai::{Array, Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEntity {
    pub id: Uuid,

    pub bot_id: Uuid,

    pub entity_type: String,

    pub entity_name: String,

    pub aliases: Vec<String>,

    pub properties: serde_json::Value,

    pub confidence: f64,

    pub source: EntitySource,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntitySource {
    Manual,
    Extracted,
    Imported,
    Inferred,
}

impl Default for EntitySource {
    fn default() -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgRelationship {
    pub id: Uuid,

    pub bot_id: Uuid,

    pub from_entity_id: Uuid,

    pub to_entity_id: Uuid,

    pub relationship_type: String,

    pub properties: serde_json::Value,

    pub confidence: f64,

    pub bidirectional: bool,

    pub source: EntitySource,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,

    pub canonical_name: String,

    pub entity_type: String,

    pub start_pos: usize,

    pub end_pos: usize,

    pub confidence: f64,

    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    pub from_entity: String,

    pub to_entity: String,

    pub relationship_type: String,

    pub confidence: f64,

    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub entities: Vec<ExtractedEntity>,

    pub relationships: Vec<ExtractedRelationship>,

    pub metadata: ExtractionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub model: String,

    pub processing_time_ms: u64,

    pub tokens_processed: usize,

    pub text_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    pub entities: Vec<KgEntity>,

    pub relationships: Vec<KgRelationship>,

    pub explanation: String,

    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraphConfig {
    pub enabled: bool,

    pub backend: String,

    pub extract_entities: bool,

    pub extraction_model: String,

    pub max_entities: usize,

    pub max_relationships: usize,

    pub min_confidence: f64,

    pub entity_types: Vec<String>,
}

impl Default for KnowledgeGraphConfig {
    fn default() -> Self {
        Self {
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

#[derive(Debug)]
pub struct KnowledgeGraphManager {
    config: KnowledgeGraphConfig,
}

impl KnowledgeGraphManager {
    pub fn new(config: KnowledgeGraphConfig) -> Self {
        Self { config }
    }

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
        Self::new(config)
    }

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

    pub fn generate_query_prompt(&self, query: &str, context: &str) -> String {
        format!(
            r"Answer this question using the knowledge graph context.

QUESTION: {query}

KNOWLEDGE GRAPH CONTEXT:
{context}

Provide a natural language answer based on the entities and relationships.
If the information is not available, say so clearly.
"
        )
    }

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
                tokens_processed: text_length / 4,
                text_length,
            },
        })
    }

    pub fn should_extract(&self) -> bool {
        self.config.enabled && self.config.extract_entities
    }

    pub fn is_valid_entity_type(&self, entity_type: &str) -> bool {
        self.config
            .entity_types
            .iter()
            .any(|t| t.eq_ignore_ascii_case(entity_type))
    }
}

fn extract_json(response: &str) -> Result<String, String> {
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start + 7..].find("```") {
            return Ok(response[start + 7..start + 7 + end].trim().to_string());
        }
    }

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

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err("No JSON found in response".to_string())
}

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

pub fn register_knowledge_graph_keywords(engine: &mut Engine) {
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

pub const KNOWLEDGE_GRAPH_SCHEMA: &str = r"
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
";

pub mod sql {
    pub const INSERT_ENTITY: &str = r"
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
    ";

    pub const INSERT_RELATIONSHIP: &str = r"
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
    ";

    pub const GET_ENTITY_BY_NAME: &str = r"
        SELECT * FROM kg_entities
        WHERE bot_id = $1
        AND (
            LOWER(entity_name) = LOWER($2)
            OR aliases @> $3::jsonb
        )
        LIMIT 1
    ";

    pub const GET_ENTITY_BY_ID: &str = r"
        SELECT * FROM kg_entities WHERE id = $1
    ";

    pub const SEARCH_ENTITIES: &str = r"
        SELECT * FROM kg_entities
        WHERE bot_id = $1
        AND (
            to_tsvector('english', entity_name) @@ plainto_tsquery('english', $2)
            OR LOWER(entity_name) LIKE LOWER($3)
        )
        ORDER BY confidence DESC
        LIMIT $4
    ";

    pub const GET_ENTITIES_BY_TYPE: &str = r"
        SELECT * FROM kg_entities
        WHERE bot_id = $1 AND entity_type = $2
        ORDER BY entity_name
        LIMIT $3
    ";

    pub const GET_RELATED_ENTITIES: &str = r"
        SELECT e.*, r.relationship_type, r.confidence as rel_confidence
        FROM kg_entities e
        JOIN kg_relationships r ON (
            (r.from_entity_id = $1 AND r.to_entity_id = e.id)
            OR (r.bidirectional AND r.to_entity_id = $1 AND r.from_entity_id = e.id)
        )
        WHERE r.bot_id = $2
        ORDER BY r.confidence DESC
        LIMIT $3
    ";

    pub const GET_RELATED_BY_TYPE: &str = r"
        SELECT e.*, r.relationship_type, r.confidence as rel_confidence
        FROM kg_entities e
        JOIN kg_relationships r ON (
            (r.from_entity_id = $1 AND r.to_entity_id = e.id)
            OR (r.bidirectional AND r.to_entity_id = $1 AND r.from_entity_id = e.id)
        )
        WHERE r.bot_id = $2 AND r.relationship_type = $3
        ORDER BY r.confidence DESC
        LIMIT $4
    ";

    pub const GET_RELATIONSHIP: &str = r"
        SELECT * FROM kg_relationships
        WHERE bot_id = $1
        AND from_entity_id = $2
        AND to_entity_id = $3
        AND relationship_type = $4
    ";

    pub const GET_ALL_RELATIONSHIPS_FOR_ENTITY: &str = r"
        SELECT r.*,
               e1.entity_name as from_name, e1.entity_type as from_type,
               e2.entity_name as to_name, e2.entity_type as to_type
        FROM kg_relationships r
        JOIN kg_entities e1 ON r.from_entity_id = e1.id
        JOIN kg_entities e2 ON r.to_entity_id = e2.id
        WHERE r.bot_id = $1
        AND (r.from_entity_id = $2 OR r.to_entity_id = $2)
        ORDER BY r.confidence DESC
    ";

    pub const DELETE_ENTITY: &str = r"
        DELETE FROM kg_entities WHERE id = $1 AND bot_id = $2
    ";

    pub const DELETE_RELATIONSHIP: &str = r"
        DELETE FROM kg_relationships WHERE id = $1 AND bot_id = $2
    ";

    pub const COUNT_ENTITIES: &str = r"
        SELECT COUNT(*) FROM kg_entities WHERE bot_id = $1
    ";

    pub const COUNT_RELATIONSHIPS: &str = r"
        SELECT COUNT(*) FROM kg_relationships WHERE bot_id = $1
    ";

    pub const GET_ENTITY_TYPES: &str = r"
        SELECT DISTINCT entity_type, COUNT(*) as count
        FROM kg_entities
        WHERE bot_id = $1
        GROUP BY entity_type
        ORDER BY count DESC
    ";

    pub const GET_RELATIONSHIP_TYPES: &str = r"
        SELECT DISTINCT relationship_type, COUNT(*) as count
        FROM kg_relationships
        WHERE bot_id = $1
        GROUP BY relationship_type
        ORDER BY count DESC
    ";

    pub const FIND_PATH: &str = r"
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
    ";
}

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
