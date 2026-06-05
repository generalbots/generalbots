use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CIType {
    Server,
    Network,
    Database,
    Application,
    Middleware,
    Storage,
    VirtualMachine,
}

impl CIType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CIType::Server => "Server",
            CIType::Network => "Network",
            CIType::Database => "Database",
            CIType::Application => "Application",
            CIType::Middleware => "Middleware",
            CIType::Storage => "Storage",
            CIType::VirtualMachine => "VirtualMachine",
        }
    }

    pub fn from_str(s: &str) -> Option<CIType> {
        match s {
            "Server" => Some(CIType::Server),
            "Network" => Some(CIType::Network),
            "Database" => Some(CIType::Database),
            "Application" => Some(CIType::Application),
            "Middleware" => Some(CIType::Middleware),
            "Storage" => Some(CIType::Storage),
            "VirtualMachine" => Some(CIType::VirtualMachine),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationItem {
    pub id: Uuid,
    pub name: String,
    pub ci_type: CIType,
    pub version: String,
    pub status: String,
    pub location: Option<String>,
    pub owner: Option<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIRelationship {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCIRequest {
    pub name: String,
    pub ci_type: String,
    pub version: String,
    pub status: String,
    pub location: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCIRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationshipRequest {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
}

#[derive(Debug, Serialize)]
pub struct ImpactAnalysis {
    pub ci: ConfigurationItem,
    pub downstream_impact: Vec<ConfigurationItem>,
    pub upstream_dependencies: Vec<ConfigurationItem>,
}

type CiStorage = Arc<Mutex<HashMap<Uuid, ConfigurationItem>>>;
type RelStorage = Arc<Mutex<Vec<CIRelationship>>>;

#[derive(Clone)]
pub struct CmdbService {
    cis: CiStorage,
    relationships: RelStorage,
}

impl CmdbService {
    pub fn new() -> Self {
        CmdbService {
            cis: Arc::new(Mutex::new(HashMap::new())),
            relationships: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create_ci(&self, req: CreateCIRequest) -> Result<ConfigurationItem, String> {
        let ci_type = CIType::from_str(&req.ci_type)
            .ok_or_else(|| format!("Invalid CI type: {}", req.ci_type))?;
        let id = Uuid::new_v4();
        let now = Utc::now();
        let ci = ConfigurationItem {
            id,
            name: req.name,
            ci_type,
            version: req.version,
            status: req.status,
            location: req.location,
            owner: req.owner,
            attributes: req.attributes.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };
        let mut store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.insert(id, ci.clone());
        Ok(ci)
    }

    pub fn get_ci(&self, id: Uuid) -> Result<ConfigurationItem, String> {
        let store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.get(&id).cloned().ok_or_else(|| format!("CI not found: {id}"))
    }

    pub fn list_cis(&self) -> Result<Vec<ConfigurationItem>, String> {
        let store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut items: Vec<ConfigurationItem> = store.values().cloned().collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(items)
    }

    pub fn update_ci(&self, id: Uuid, req: UpdateCIRequest) -> Result<ConfigurationItem, String> {
        let mut store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        let ci = store.get_mut(&id).ok_or_else(|| format!("CI not found: {id}"))?;
        if let Some(name) = req.name {
            ci.name = name;
        }
        if let Some(version) = req.version {
            ci.version = version;
        }
        if let Some(status) = req.status {
            ci.status = status;
        }
        if let Some(location) = req.location {
            ci.location = Some(location);
        }
        if let Some(owner) = req.owner {
            ci.owner = Some(owner);
        }
        if let Some(attributes) = req.attributes {
            ci.attributes = attributes;
        }
        ci.updated_at = Utc::now();
        Ok(ci.clone())
    }

    pub fn delete_ci(&self, id: Uuid) -> Result<(), String> {
        let mut store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.remove(&id).ok_or_else(|| format!("CI not found: {id}"))?;
        let mut rels = self.relationships.lock().map_err(|e| format!("Lock error: {e}"))?;
        rels.retain(|r| r.source_id != id && r.target_id != id);
        Ok(())
    }

    pub fn create_relationship(&self, req: CreateRelationshipRequest) -> Result<CIRelationship, String> {
        let store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        if !store.contains_key(&req.source_id) {
            return Err(format!("Source CI not found: {}", req.source_id));
        }
        if !store.contains_key(&req.target_id) {
            return Err(format!("Target CI not found: {}", req.target_id));
        }
        drop(store);
        let rel = CIRelationship {
            source_id: req.source_id,
            target_id: req.target_id,
            relationship_type: req.relationship_type,
        };
        let mut rels = self.relationships.lock().map_err(|e| format!("Lock error: {e}"))?;
        rels.push(rel.clone());
        Ok(rel)
    }

    pub fn list_relationships(&self) -> Result<Vec<CIRelationship>, String> {
        let rels = self.relationships.lock().map_err(|e| format!("Lock error: {e}"))?;
        Ok(rels.clone())
    }

    pub fn get_relationships_for_ci(&self, ci_id: Uuid) -> Result<Vec<CIRelationship>, String> {
        let rels = self.relationships.lock().map_err(|e| format!("Lock error: {e}"))?;
        let filtered: Vec<CIRelationship> = rels
            .iter()
            .filter(|r| r.source_id == ci_id || r.target_id == ci_id)
            .cloned()
            .collect();
        Ok(filtered)
    }

    pub fn impact_analysis(&self, ci_id: Uuid) -> Result<ImpactAnalysis, String> {
        let store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        let ci = store.get(&ci_id).cloned()
            .ok_or_else(|| format!("CI not found: {ci_id}"))?;
        drop(store);
        let rels = self.relationships.lock().map_err(|e| format!("Lock error: {e}"))?;
        let downstream_ids: Vec<Uuid> = rels
            .iter()
            .filter(|r| r.source_id == ci_id)
            .map(|r| r.target_id)
            .collect();
        let upstream_ids: Vec<Uuid> = rels
            .iter()
            .filter(|r| r.target_id == ci_id)
            .map(|r| r.source_id)
            .collect();
        drop(rels);
        let store = self.cis.lock().map_err(|e| format!("Lock error: {e}"))?;
        let downstream_impact: Vec<ConfigurationItem> = downstream_ids
            .iter()
            .filter_map(|id| store.get(id).cloned())
            .collect();
        let upstream_dependencies: Vec<ConfigurationItem> = upstream_ids
            .iter()
            .filter_map(|id| store.get(id).cloned())
            .collect();
        Ok(ImpactAnalysis {
            ci,
            downstream_impact,
            upstream_dependencies,
        })
    }
}
