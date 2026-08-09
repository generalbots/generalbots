//! Capability registry for the Vibe platform (Issue #801).
//!
//! Derives the runtime capability surface from the actual tool registry:
//! tools are grouped by category and use case, so the LLM and the UI can
//! answer "what can a run in this use case do?" with the ground truth
//! instead of a hardcoded prompt list. Capabilities are pure data built
//! from [`ToolDescriptor`]s, keeping this module free of runtime state.

use crate::tool_executor::{ToolCategory, ToolDescriptor};
use crate::types::VibeUseCase;
use serde::{Deserialize, Serialize};

/// A group of tools sharing a category and a use-case scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Stable identifier, e.g. "deployment/software_development".
    pub id: String,
    /// Human-readable category title.
    pub title: String,
    /// What the capability enables the agent to do.
    pub description: String,
    /// Category the tools belong to.
    pub category: ToolCategory,
    /// Use cases this capability serves.
    pub use_cases: Vec<VibeUseCase>,
    /// Tool names included in this capability.
    pub tools: Vec<String>,
    /// True when at least one tool requires human approval.
    pub requires_approval: bool,
}

/// Total number of use cases, used to enumerate capability scopes.
pub const ALL_USE_CASES: [VibeUseCase; 3] = [
    VibeUseCase::SoftwareDevelopment,
    VibeUseCase::CustomerSupport,
    VibeUseCase::FinancialAnalysis,
];

/// Builds the full capability set from the registered tool descriptors.
/// Tools without explicit use-case restrictions belong to every use case.
pub fn build_capabilities(tools: &[ToolDescriptor]) -> Vec<Capability> {
    let mut capabilities = Vec::new();
    for category in category_order() {
        let category_tools: Vec<&ToolDescriptor> = tools
            .iter()
            .filter(|t| t.category == category)
            .collect();
        if category_tools.is_empty() {
            continue;
        }
        for use_case in ALL_USE_CASES {
            let scoped: Vec<&ToolDescriptor> = category_tools
                .iter()
                .copied()
                .filter(|t| {
                    t.schema.allowed_use_cases.is_empty()
                        || t.schema.allowed_use_cases.contains(&use_case)
                })
                .collect();
            if scoped.is_empty() {
                continue;
            }
            let tools: Vec<String> = scoped.iter().map(|t| t.schema.name.clone()).collect();
            let requires_approval = scoped
                .iter()
                .any(|t| t.schema.requires_approval);
            capabilities.push(Capability {
                id: format!("{}/{}", category.as_str(), use_case_str(use_case)),
                title: category_title(category).to_string(),
                description: category_description(category).to_string(),
                category,
                use_cases: vec![use_case],
                tools,
                requires_approval,
            });
        }
    }
    capabilities.sort_by(|a, b| a.id.cmp(&b.id));
    capabilities
}

/// Keeps only capabilities that serve the requested use case.
pub fn capabilities_for(capabilities: &[Capability], use_case: VibeUseCase) -> Vec<Capability> {
    capabilities
        .iter()
        .filter(|c| c.use_cases.contains(&use_case))
        .cloned()
        .collect()
}

fn use_case_str(use_case: VibeUseCase) -> &'static str {
    match use_case {
        VibeUseCase::SoftwareDevelopment => "software_development",
        VibeUseCase::CustomerSupport => "customer_support",
        VibeUseCase::FinancialAnalysis => "financial_analysis",
    }
}

fn category_order() -> [ToolCategory; 6] {
    [
        ToolCategory::Autotask,
        ToolCategory::Deployment,
        ToolCategory::Crm,
        ToolCategory::Sources,
        ToolCategory::File,
        ToolCategory::Analysis,
    ]
}

fn category_title(category: ToolCategory) -> &'static str {
    match category {
        ToolCategory::Autotask => "Task automation",
        ToolCategory::Deployment => "Deployment",
        ToolCategory::Crm => "Customer relationship",
        ToolCategory::Sources => "Data sources",
        ToolCategory::File => "File operations",
        ToolCategory::Analysis => "Analysis",
    }
}

fn category_description(category: ToolCategory) -> &'static str {
    match category {
        ToolCategory::Autotask => "Classifies intent, compiles and executes multi-step plans",
        ToolCategory::Deployment => "Deploys applications, binds domains and provisions TLS",
        ToolCategory::Crm => "Manages contacts, deals and customer records",
        ToolCategory::Sources => "Connects to external data sources",
        ToolCategory::File => "Reads, writes and organizes files",
        ToolCategory::Analysis => "Runs analysis and insight generation over data",
    }
}

impl ToolCategory {
    /// Machine-readable category name used in capability IDs.
    fn as_str(&self) -> &'static str {
        match self {
            Self::Autotask => "autotask",
            Self::Deployment => "deployment",
            Self::Crm => "crm",
            Self::Sources => "sources",
            Self::File => "file",
            Self::Analysis => "analysis",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_executor::ToolSchema;

    fn descriptor(
        name: &str,
        category: ToolCategory,
        use_cases: Vec<VibeUseCase>,
        requires_approval: bool,
    ) -> ToolDescriptor {
        ToolDescriptor {
            schema: ToolSchema {
                name: name.into(),
                description: name.into(),
                parameters: serde_json::json!({"type": "object", "properties": {}}),
                requires_approval,
                allowed_use_cases: use_cases,
            },
            category,
        }
    }

    #[test]
    fn groups_tools_by_category_and_use_case() {
        let tools = vec![
            descriptor("deploy_app", ToolCategory::Deployment, vec![VibeUseCase::SoftwareDevelopment], true),
            descriptor("global_file", ToolCategory::File, vec![], false),
        ];
        let caps = build_capabilities(&tools);
        assert!(caps.iter().any(|c| c.id == "deployment/software_development"));
        assert!(!caps.iter().any(|c| c.id == "deployment/customer_support"));
        let file_caps: Vec<_> = caps.iter().filter(|c| c.category == ToolCategory::File).collect();
        assert_eq!(file_caps.len(), 3, "unrestricted category appears in every use case");
        let sd = caps.iter().find(|c| c.id == "deployment/software_development").expect("sd deployment");
        assert!(sd.requires_approval);
        assert_eq!(sd.tools, vec!["deploy_app".to_string()]);
        assert_eq!(sd.title, "Deployment");
        assert!(!sd.description.is_empty());
    }

    #[test]
    fn filters_by_use_case() {
        let tools = vec![
            descriptor("deploy_app", ToolCategory::Deployment, vec![VibeUseCase::SoftwareDevelopment], false),
            descriptor("forecast", ToolCategory::Analysis, vec![VibeUseCase::FinancialAnalysis], false),
        ];
        let caps = build_capabilities(&tools);
        let fa = capabilities_for(&caps, VibeUseCase::FinancialAnalysis);
        assert_eq!(fa.len(), 1);
        assert_eq!(fa[0].id, "analysis/financial_analysis");
        let cs = capabilities_for(&caps, VibeUseCase::CustomerSupport);
        assert!(cs.is_empty());
    }

    #[test]
    fn no_approval_flag_when_no_tool_requires_it() {
        let tools = vec![descriptor("agent", ToolCategory::Autotask, vec![VibeUseCase::CustomerSupport], false)];
        let caps = build_capabilities(&tools);
        assert!(!caps[0].requires_approval);
    }

    #[test]
    fn capabilities_are_sorted_and_deterministic() {
        let tools = vec![
            descriptor("a", ToolCategory::Deployment, vec![VibeUseCase::SoftwareDevelopment], false),
            descriptor("b", ToolCategory::Autotask, vec![VibeUseCase::CustomerSupport], false),
        ];
        let first = build_capabilities(&tools);
        let second = build_capabilities(&tools);
        let ids: Vec<String> = first.iter().map(|c| c.id.clone()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
        assert_eq!(first, second);
    }

    #[test]
    fn serialize_round_trip() {
        let caps = build_capabilities(&[descriptor("f", ToolCategory::File, vec![], false)]);
        let value = serde_json::to_value(&caps[0]).expect("serialize");
        let decoded: Capability = serde_json::from_value(value).expect("deserialize");
        assert_eq!(decoded, caps[0]);
    }
}