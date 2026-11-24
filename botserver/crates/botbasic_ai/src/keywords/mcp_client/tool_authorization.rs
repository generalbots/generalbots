//! #2662 — enforcement of the risk/permission metadata declared by MCP tools.
//!
//! `McpTool` has always declared `required_permissions`, `risk_level`,
//! `is_destructive` and `requires_approval`, and `invoke_tool` ignored all of
//! them. This module makes that metadata authoritative: the decision is a pure
//! function (unit-testable without a database or an HTTP server) and
//! `invoke_tool` refuses to dispatch when it denies.
//!
//! Fail-closed rules:
//!   * a tool with no declared metadata is never invoked;
//!   * `High`/`Critical` tools need the admin role;
//!   * every declared capability grant must be held by the caller;
//!   * explicit approval / destructive consent is never bypassed by role.

use super::*;

/// Matches `botserver::security::user_role::ROLE_ADMIN`. Duplicated here because
/// a library crate cannot depend on the botserver binary crate.
pub(crate) const ADMIN_ROLE: &str = "admin";

/// Effective authorization requirements for one tool invocation. Server-wide
/// metadata is folded in so the CSV `risk_level`/`requires_approval` columns are
/// not silently discarded (see `mcp_directory::get_csv_path`).
#[derive(Debug, Clone)]
pub(crate) struct ToolRequirements<'a> {
    pub risk_level: ToolRiskLevel,
    pub is_destructive: bool,
    pub requires_approval: bool,
    pub required_permissions: &'a [String],
}

/// Who is calling. `Default` is deliberately fail-closed.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ToolCallerContext<'a> {
    pub role: &'a str,
    pub granted_permissions: &'a [String],
    pub approval_granted: bool,
    pub destructive_consent_granted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolDenyReason {
    /// No tool definition / metadata was found for the requested name.
    ToolMetadataMissing,
    /// High or critical risk tool invoked by a non-admin.
    AdminRoleRequired,
    /// Destructive tool invoked without explicit destructive consent.
    DestructiveConsentRequired,
    /// Tool declared `requires_approval` but no approval was granted.
    ApprovalRequired,
    /// A declared capability grant is missing from the caller.
    MissingPermission(String),
}

impl ToolDenyReason {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::ToolMetadataMissing => "TOOL_METADATA_MISSING",
            Self::AdminRoleRequired => "TOOL_ADMIN_ROLE_REQUIRED",
            Self::DestructiveConsentRequired => "TOOL_DESTRUCTIVE_CONSENT_REQUIRED",
            Self::ApprovalRequired => "TOOL_APPROVAL_REQUIRED",
            Self::MissingPermission(_) => "TOOL_PERMISSION_REQUIRED",
        }
    }

    pub(crate) fn message(&self) -> String {
        match self {
            Self::ToolMetadataMissing => {
                "tool has no declared authorization metadata; refusing to invoke".to_string()
            }
            Self::AdminRoleRequired => {
                "this tool is high risk and requires the admin role".to_string()
            }
            Self::DestructiveConsentRequired => {
                "this tool is destructive and requires explicit consent".to_string()
            }
            Self::ApprovalRequired => {
                "this tool requires explicit approval before invocation".to_string()
            }
            Self::MissingPermission(permission) => {
                format!("caller is missing required permission '{permission}'")
            }
        }
    }
}

/// Fold the server-wide risk floor / consent flag into the tool's own metadata.
pub(crate) fn effective_requirements<'a>(
    tool: &'a McpTool,
    server: &McpServer,
) -> ToolRequirements<'a> {
    ToolRequirements {
        risk_level: if tool.risk_level >= server.risk_level {
            tool.risk_level
        } else {
            server.risk_level
        },
        is_destructive: tool.is_destructive,
        requires_approval: tool.requires_approval || server.requires_approval,
        required_permissions: &tool.required_permissions,
    }
}

/// The pure risk-decision predicate. No I/O, no hidden state: every input that
/// can change the answer is an explicit argument, so it is fully unit-testable.
pub(crate) fn decide_tool_access(
    requirements: &ToolRequirements<'_>,
    caller: &ToolCallerContext<'_>,
) -> Result<(), ToolDenyReason> {
    let is_admin = caller.role.eq_ignore_ascii_case(ADMIN_ROLE);

    // High-risk tools are admin-only. An empty/unknown role is non-admin.
    if requirements.risk_level >= ToolRiskLevel::High && !is_admin {
        return Err(ToolDenyReason::AdminRoleRequired);
    }

    // Consent is per-invocation and is never bypassed by role.
    if requirements.is_destructive && !caller.destructive_consent_granted {
        return Err(ToolDenyReason::DestructiveConsentRequired);
    }
    if requirements.requires_approval && !caller.approval_granted {
        return Err(ToolDenyReason::ApprovalRequired);
    }

    // Every declared capability grant must be present. Admin role satisfies the
    // risk gate above only; it does not implicitly grant capabilities.
    for permission in requirements.required_permissions {
        if !caller.granted_permissions.iter().any(|p| p == permission) {
            return Err(ToolDenyReason::MissingPermission(permission.clone()));
        }
    }

    Ok(())
}

/// Look up the tool, combine metadata and decide. Used by `invoke_tool` before
/// any HTTP/stdio dispatch happens.
pub(crate) fn authorize_tool_invocation(
    server: &McpServer,
    request: &McpRequest,
) -> Result<(), ToolDenyReason> {
    let tool = match server.tools.iter().find(|t| t.name == request.tool) {
        Some(tool) => tool,
        None => return Err(ToolDenyReason::ToolMetadataMissing),
    };
    let requirements = effective_requirements(tool, server);
    let caller = ToolCallerContext {
        role: &request.context.role,
        granted_permissions: &request.context.granted_permissions,
        approval_granted: request.context.approval_granted,
        destructive_consent_granted: request.context.destructive_consent_granted,
    };
    decide_tool_access(&requirements, &caller)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req<'a>(
        risk: ToolRiskLevel,
        destructive: bool,
        approval: bool,
        permissions: &'a [String],
    ) -> ToolRequirements<'a> {
        ToolRequirements {
            risk_level: risk,
            is_destructive: destructive,
            requires_approval: approval,
            required_permissions: permissions,
        }
    }

    fn caller<'a>(
        role: &'a str,
        permissions: &'a [String],
        approval: bool,
        destructive: bool,
    ) -> ToolCallerContext<'a> {
        ToolCallerContext {
            role,
            granted_permissions: permissions,
            approval_granted: approval,
            destructive_consent_granted: destructive,
        }
    }

    #[test]
    fn safe_tool_allowed_for_user_without_consent() {
        let requirements = req(ToolRiskLevel::Safe, false, false, &[]);
        assert_eq!(decide_tool_access(&requirements, &caller("", &[], false, false)), Ok(()));
    }

    #[test]
    fn high_and_critical_tools_denied_for_non_admin() {
        for risk in [ToolRiskLevel::High, ToolRiskLevel::Critical] {
            let requirements = req(risk, false, false, &[]);
            assert_eq!(
                decide_tool_access(&requirements, &caller("user", &[], false, false)),
                Err(ToolDenyReason::AdminRoleRequired)
            );
        }
    }

    #[test]
    fn high_risk_tool_allowed_for_admin() {
        let requirements = req(ToolRiskLevel::High, false, false, &[]);
        assert_eq!(decide_tool_access(&requirements, &caller("admin", &[], false, false)), Ok(()));
    }

    #[test]
    fn approval_required_tool_denied_without_approval() {
        let requirements = req(ToolRiskLevel::Low, false, true, &[]);
        assert_eq!(
            decide_tool_access(&requirements, &caller("admin", &[], false, false)),
            Err(ToolDenyReason::ApprovalRequired)
        );
    }

    #[test]
    fn approval_required_tool_allowed_with_approval() {
        let requirements = req(ToolRiskLevel::Low, false, true, &[]);
        assert_eq!(decide_tool_access(&requirements, &caller("user", &[], true, false)), Ok(()));
    }

    #[test]
    fn destructive_tool_denied_without_destructive_consent() {
        let requirements = req(ToolRiskLevel::Low, true, false, &[]);
        assert_eq!(
            decide_tool_access(&requirements, &caller("admin", &[], true, false)),
            Err(ToolDenyReason::DestructiveConsentRequired)
        );
    }

    #[test]
    fn destructive_tool_allowed_with_destructive_consent() {
        let requirements = req(ToolRiskLevel::Low, true, false, &[]);
        assert_eq!(decide_tool_access(&requirements, &caller("user", &[], false, true)), Ok(()));
    }

    #[test]
    fn missing_permission_denied_even_for_admin() {
        let required = vec!["filesystem.write".to_string()];
        let requirements = req(ToolRiskLevel::Safe, false, false, &required);
        assert_eq!(
            decide_tool_access(&requirements, &caller("admin", &[], false, false)),
            Err(ToolDenyReason::MissingPermission("filesystem.write".to_string()))
        );
    }

    #[test]
    fn granted_permission_allows_invocation() {
        let required = vec!["filesystem.write".to_string()];
        let granted = vec!["filesystem.write".to_string()];
        let requirements = req(ToolRiskLevel::Safe, false, false, &required);
        assert_eq!(decide_tool_access(&requirements, &caller("user", &granted, false, false)), Ok(()));
    }

    fn tool(name: &str, risk: ToolRiskLevel, approval: bool) -> McpTool {
        McpTool {
            name: name.to_string(),
            description: String::new(),
            input_schema: serde_json::json!({}),
            output_schema: None,
            required_permissions: Vec::new(),
            risk_level: risk,
            is_destructive: false,
            requires_approval: approval,
            ui: None,
            visibility: Vec::new(),
            rate_limit: None,
            timeout_seconds: None,
        }
    }

    fn server(tools: Vec<McpTool>, risk: ToolRiskLevel, approval: bool) -> McpServer {
        McpServer {
            id: "s".to_string(),
            name: "srv".to_string(),
            description: String::new(),
            server_type: McpServerType::Custom("custom".to_string()),
            connection: McpConnection {
                connection_type: ConnectionType::Http,
                url: String::new(),
                port: None,
                timeout_seconds: 30,
                max_retries: 0,
                retry_backoff_ms: 0,
                keep_alive: false,
                tls_config: None,
            },
            auth: McpAuth::default(),
            tools,
            resources: Vec::new(),
            capabilities: McpCapabilities::default(),
            status: McpServerStatus::Active,
            bot_id: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_health_check: None,
            health_status: HealthStatus::default(),
            risk_level: risk,
            requires_approval: approval,
        }
    }

    fn request(tool_name: &str, role: &str, approval: bool) -> McpRequest {
        McpRequest {
            id: "r".to_string(),
            server: "srv".to_string(),
            tool: tool_name.to_string(),
            arguments: serde_json::json!({}),
            context: McpRequestContext {
                session_id: String::new(),
                bot_id: String::new(),
                user_id: String::new(),
                task_id: None,
                step_id: None,
                correlation_id: None,
                role: role.to_string(),
                granted_permissions: Vec::new(),
                approval_granted: approval,
                destructive_consent_granted: false,
            },
            timeout_seconds: None,
        }
    }

    #[test]
    fn missing_tool_metadata_fails_closed() {
        let srv = server(Vec::new(), ToolRiskLevel::Safe, false);
        let request = request("unknown", "admin", true);
        assert_eq!(
            authorize_tool_invocation(&srv, &request),
            Err(ToolDenyReason::ToolMetadataMissing)
        );
    }

    #[test]
    fn server_risk_floor_escalates_tool_metadata() {
        let tool = tool("read", ToolRiskLevel::Low, false);
        let effective = effective_requirements(&tool, &server(Vec::new(), ToolRiskLevel::High, false));
        assert_eq!(effective.risk_level, ToolRiskLevel::High);
    }

    #[test]
    fn server_approval_flag_is_honored() {
        let tool = tool("read", ToolRiskLevel::Safe, false);
        let srv = server(vec![tool], ToolRiskLevel::Safe, true);
        let request = request("read", "user", false);
        assert_eq!(
            authorize_tool_invocation(&srv, &request),
            Err(ToolDenyReason::ApprovalRequired)
        );
    }
}
