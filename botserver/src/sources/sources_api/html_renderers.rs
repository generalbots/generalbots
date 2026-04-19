use super::types::{McpServersCatalog, PromptData, TemplateData};
use log::error;

pub fn get_prompts_data(category: &str) -> Vec<PromptData> {
    let all_prompts = vec![
        PromptData {
            id: "summarize".to_string(),
            title: "Summarize Text".to_string(),
            description: "Create concise summaries of long documents".to_string(),
            category: "writing".to_string(),
            icon: "📝".to_string(),
        },
        PromptData {
            id: "code-review".to_string(),
            title: "Code Review".to_string(),
            description: "Analyze code for bugs and improvements".to_string(),
            category: "coding".to_string(),
            icon: "🔍".to_string(),
        },
        PromptData {
            id: "data-analysis".to_string(),
            title: "Data Analysis".to_string(),
            description: "Extract insights from data sets".to_string(),
            category: "analysis".to_string(),
            icon: "📊".to_string(),
        },
        PromptData {
            id: "creative-writing".to_string(),
            title: "Creative Writing".to_string(),
            description: "Generate stories and creative content".to_string(),
            category: "creative".to_string(),
            icon: "🎨".to_string(),
        },
        PromptData {
            id: "email-draft".to_string(),
            title: "Email Draft".to_string(),
            description: "Compose professional emails".to_string(),
            category: "business".to_string(),
            icon: "📧".to_string(),
        },
    ];

    if category == "all" {
        all_prompts
    } else {
        all_prompts
            .into_iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

pub fn get_templates_data() -> Vec<TemplateData> {
    vec![
        TemplateData {
            name: "Customer Support Bot".to_string(),
            description: "Handle customer inquiries automatically".to_string(),
            category: "Support".to_string(),
            icon: "🎧".to_string(),
        },
        TemplateData {
            name: "FAQ Bot".to_string(),
            description: "Answer frequently asked questions".to_string(),
            category: "Support".to_string(),
            icon: "❓".to_string(),
        },
        TemplateData {
            name: "Lead Generation Bot".to_string(),
            description: "Qualify leads and collect information".to_string(),
            category: "Sales".to_string(),
            icon: "🎯".to_string(),
        },
    ]
}

pub fn load_mcp_servers_catalog() -> Option<McpServersCatalog> {
    let catalog_path = std::path::Path::new("./3rdparty/mcp_servers.json");
    if catalog_path.exists() {
        match std::fs::read_to_string(catalog_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(catalog) => Some(catalog),
                Err(e) => {
                    error!("Failed to parse mcp_servers.json: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to read mcp_servers.json: {}", e);
                None
            }
        }
    } else {
        None
    }
}

pub fn get_category_icon(category: &str) -> &'static str {
    match category {
        "Database" => "🗄️",
        "Analytics" => "📊",
        "Search" => "🔍",
        "Vector Database" => "🧮",
        "Deployment" => "🚀",
        "Data Catalog" => "📚",
        "Productivity" => "✅",
        "AI/ML" => "🤖",
        "Storage" => "💾",
        "DevOps" => "⚙️",
        "Process Mining" => "⛏️",
        "Development" => "💻",
        "Communication" => "💬",
        "Customer Support" => "🎧",
        "Finance" => "💰",
        "Enterprise" => "🏢",
        "HR" => "👥",
        "Security" => "🔒",
        "Documentation" => "📖",
        "Integration" => "🔗",
        "API" => "🔌",
        "Payments" => "💳",
        "Maps" => "🗺️",
        "Web Development" => "🌐",
        "Scheduling" => "📅",
        "Document Management" => "📁",
        "Contact Management" => "📇",
        "URL Shortener" => "🔗",
        "Manufacturing" => "🏭",
        _ => "📦",
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
