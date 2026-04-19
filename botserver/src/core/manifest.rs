use serde::{Deserialize, Serialize};
use crate::core::features::COMPILED_FEATURES;
use crate::core::product::PRODUCT_CONFIG;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub version: String,
    pub features: FeatureManifest,
    pub apps: Vec<AppInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureManifest {
    pub compiled: Vec<String>,
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub enabled: bool,
    pub compiled: bool,
    pub category: String,
}

impl WorkspaceManifest {
    pub fn new() -> Self {
        let compiled: Vec<String> = COMPILED_FEATURES.iter().map(|s| s.to_string()).collect();
        
        let enabled: Vec<String> = PRODUCT_CONFIG
            .read()
            .ok()
            .as_ref()
            .map(|c| c.get_enabled_apps())
            .unwrap_or_default()
            .into_iter()
            .filter(|app| compiled.contains(app))
            .collect();
        
        let disabled: Vec<String> = compiled
            .iter()
            .filter(|app| !enabled.contains(app))
            .cloned()
            .collect();
        
        let apps = Self::build_app_info(&compiled, &enabled);
        
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            features: FeatureManifest {
                compiled,
                enabled,
                disabled,
            },
            apps,
        }
    }
    
    fn build_app_info(compiled: &[String], enabled: &[String]) -> Vec<AppInfo> {
        let mut apps = Vec::new();
        
        // Communication apps
        for app in ["chat", "mail", "calendar", "meet", "people", "social"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "communication".to_string(),
                });
            }
        }
        
        // Productivity apps
        for app in ["drive", "tasks", "docs", "paper", "sheet", "slides"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "productivity".to_string(),
                });
            }
        }
        
        // Project management
        for app in ["project", "goals", "workspace", "tickets"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "project_management".to_string(),
                });
            }
        }
        
        // Business apps
        for app in ["billing", "products", "analytics"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "business".to_string(),
                });
            }
        }
        
        // Media apps
        for app in ["video", "player"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "media".to_string(),
                });
            }
        }
        
        // Learning apps
        for app in ["canvas", "learn", "research"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "learning".to_string(),
                });
            }
        }
        
        // Developer apps
        for app in ["sources", "dashboards", "designer", "editor", "tools"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "developer".to_string(),
                });
            }
        }
        
        // System apps
        for app in ["automation", "cache", "directory", "admin", "monitoring", "attendant"] {
            if compiled.contains(&app.to_string()) {
                apps.push(AppInfo {
                    name: app.to_string(),
                    enabled: enabled.contains(&app.to_string()),
                    compiled: true,
                    category: "system".to_string(),
                });
            }
        }
        
        apps
    }
}

impl Default for WorkspaceManifest {
    fn default() -> Self {
        Self::new()
    }
}
