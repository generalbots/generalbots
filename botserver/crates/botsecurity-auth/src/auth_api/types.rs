use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Comprehensive Permission enumeration representing 261 granular domain permissions
/// along with legacy ones for backward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // --- Legacy Permissions ---
    Read,
    Write,
    Delete,
    Admin,
    ManageUsers,
    ManageBots,
    ViewAnalytics,
    ManageSettings,
    ExecuteTasks,
    ViewLogs,
    ManageSecrets,
    AccessApi,
    ManageFiles,
    SendMessages,
    ViewConversations,
    ManageWebhooks,
    ManageIntegrations,
    // --- Granular Domain Permissions ---
    AdministrationCanManageOrganization,
    AdministrationCanManageMembers,
    AdministrationCanViewMembers,
    AdministrationCanManageSettings,
    AdministrationCanManageBilling,
    AdministrationCanViewBilling,
    AdministrationCanManageAuditLog,
    AdministrationCanViewAuditLog,
    AdministrationCanManageDns,
    AdministrationCanManageOnboarding,
    AdministrationCanManageRoles,
    AdministrationCanManageGroups,
    ComplianceCanViewDashboard,
    ComplianceCanManagePolicies,
    ComplianceCanViewReports,
    ComplianceCanExportReports,
    ComplianceCanManageDataRetention,
    ComplianceCanManageGdpr,
    ComplianceCanManageHipaa,
    ComplianceCanManageIso27001,
    ComplianceCanManageSoc2,
    SecurityCanManageUsers,
    SecurityCanManageSecrets,
    SecurityCanViewLogs,
    SecurityCanManageApiKeys,
    SecurityCanManageIpSafelist,
    SecurityCanManageMfa,
    SecurityCanManageSessions,
    SecurityCanManageEncryption,
    SecurityCanManageIntegrations,
    SecurityCanManagePasswordPolicy,
    SecurityCanConfigure,
    SecurityCanAdmin,
    MailCanRead,
    MailCanSend,
    MailCanDelete,
    MailCanManageFolders,
    MailCanManageFilters,
    MailCanManageTemplates,
    MailCanManageSignatures,
    MailCanManageAutoReply,
    MailCanManageForwarding,
    MailCanSendCampaigns,
    MailCanAdmin,
    MailCanConfigure,
    CalendarCanRead,
    CalendarCanCreate,
    CalendarCanUpdate,
    CalendarCanDelete,
    CalendarCanManageCalendars,
    CalendarCanShare,
    CalendarCanManageReminders,
    CalendarCanViewAvailability,
    DriveCanRead,
    DriveCanWrite,
    DriveCanDelete,
    DriveCanUpload,
    DriveCanDownload,
    DriveCanManageFolders,
    DriveCanShare,
    DriveCanManagePermissions,
    DriveCanAdmin,
    DriveCanManageVersions,
    DocumentsCanRead,
    DocumentsCanCreate,
    DocumentsCanUpdate,
    DocumentsCanDelete,
    DocumentsCanShare,
    DocumentsCanExport,
    DocumentsCanManageTemplates,
    DocumentsCanManageFolders,
    DocumentsCanComment,
    DocumentsCanTrackChanges,
    SpreadsheetsCanRead,
    SpreadsheetsCanCreate,
    SpreadsheetsCanUpdate,
    SpreadsheetsCanDelete,
    SpreadsheetsCanShare,
    SpreadsheetsCanExport,
    SpreadsheetsCanImport,
    SpreadsheetsCanManageFormulas,
    SpreadsheetsCanManageCharts,
    PresentationsCanRead,
    PresentationsCanCreate,
    PresentationsCanUpdate,
    PresentationsCanDelete,
    PresentationsCanPresent,
    PresentationsCanExport,
    MeetingsCanCreate,
    MeetingsCanJoin,
    MeetingsCanManageRooms,
    MeetingsCanRecord,
    MeetingsCanShareScreen,
    MeetingsCanManageParticipants,
    MeetingsCanManageSettings,
    MeetingsCanAdmin,
    ChatCanSendMessages,
    ChatCanReadMessages,
    ChatCanDeleteMessages,
    ChatCanViewConversations,
    ChatCanManageBots,
    ChatCanCreateBots,
    ChatCanEditBots,
    ChatCanDeleteBots,
    ChatCanPublishBots,
    ChatCanViewBots,
    ChatCanExecuteTools,
    ChatCanManageKnowledgeBase,
    ChatCanReadKnowledgeBase,
    ChatCanWriteKnowledgeBase,
    ChatCanAdminKnowledgeBase,
    ChatCanConfigureBots,
    TasksCanCreate,
    TasksCanRead,
    TasksCanUpdate,
    TasksCanDelete,
    TasksCanExecute,
    TasksCanAssign,
    TasksCanManageProjects,
    TasksCanManageWorkflows,
    TasksCanManageAutoTask,
    AiToolsCanManageLlm,
    AiToolsCanConfigureLlm,
    AiToolsCanManageModels,
    AiToolsCanManagePrompts,
    AiToolsCanDesignScripts,
    AiToolsCanEditScripts,
    AiToolsCanManageAutotask,
    AiToolsCanManageTraining,
    AiToolsCanManageVibe,
    AiToolsCanManageMcp,
    BusinessIntelligenceCanViewDashboard,
    BusinessIntelligenceCanViewReports,
    BusinessIntelligenceCanCreateReports,
    BusinessIntelligenceCanEditReports,
    BusinessIntelligenceCanDeleteReports,
    BusinessIntelligenceCanExportReports,
    BusinessIntelligenceCanViewMetrics,
    BusinessIntelligenceCanManageDashboards,
    BusinessIntelligenceCanTrace,
    BusinessIntelligenceCanMonitorPerformance,
    BusinessIntelligenceCanViewAnalytics,
    BusinessIntelligenceCanExportAnalytics,
    IntegrationsCanManageWebhooks,
    IntegrationsCanManageApiKeys,
    IntegrationsCanConnectSources,
    IntegrationsCanManageSocialMedia,
    IntegrationsCanManageWhatsApp,
    IntegrationsCanManageTelegram,
    IntegrationsCanManageMsTeams,
    IntegrationsCanManageInstagram,
    IntegrationsCanManageImap,
    IntegrationsCanManageGoogle,
    IntegrationsCanManageMicrosoft,
    IntegrationsCanManageChannels,
    AutomationCanCreateWorkflows,
    AutomationCanEditWorkflows,
    AutomationCanDeleteWorkflows,
    AutomationCanExecuteWorkflows,
    AutomationCanManageTriggers,
    AutomationCanManageSchedules,
    AutomationCanManageEventHandlers,
    CrmCanViewPipeline,
    CrmCanManageLeads,
    CrmCanManageContacts,
    CrmCanManageDeals,
    CrmCanViewReports,
    CrmCanExportReports,
    CrmCanManageForecast,
    CampaignsCanCreate,
    CampaignsCanEdit,
    CampaignsCanDelete,
    CampaignsCanExecute,
    CampaignsCanViewAnalytics,
    CampaignsCanManageSegments,
    ProductsCanViewCatalog,
    ProductsCanCreateProducts,
    ProductsCanEditProducts,
    ProductsCanDeleteProducts,
    ProductsCanCreateServices,
    ProductsCanEditServices,
    ProductsCanDeleteServices,
    ProductsCanManagePriceLists,
    TicketsCanCreate,
    TicketsCanRead,
    TicketsCanUpdate,
    TicketsCanDelete,
    TicketsCanAssign,
    TicketsCanResolve,
    TicketsCanManagePriorities,
    TicketsCanViewAnalytics,
    TicketsCanManageAttendant,
    PeopleCanViewDirectory,
    PeopleCanManageContacts,
    PeopleCanManageGroups,
    PeopleCanManageRoles,
    PeopleCanImportContacts,
    BrowserCanNavigate,
    BrowserCanBookmark,
    BrowserCanManageHistory,
    BrowserCanDownload,
    TerminalCanExecute,
    TerminalCanViewOutput,
    TerminalCanManageSessions,
    ResearchCanSearch,
    ResearchCanManageSources,
    ResearchCanExportResults,
    ResearchCanManageSessions,
    SocialCanPost,
    SocialCanSchedulePosts,
    SocialCanViewFeed,
    SocialCanManageAccounts,
    SocialCanViewAnalytics,
    VideoCanUpload,
    VideoCanPlay,
    VideoCanEdit,
    VideoCanDelete,
    VideoCanManageLibrary,
    CanvasCanCreate,
    CanvasCanEdit,
    CanvasCanView,
    CanvasCanDelete,
    CanvasCanExport,
    WorkspaceCanCreateSites,
    WorkspaceCanEditSites,
    WorkspaceCanDeleteSites,
    WorkspaceCanViewSites,
    WorkspaceCanManagePages,
    WorkspaceCanManageDatabases,
    GoalsCanCreate,
    GoalsCanEdit,
    GoalsCanDelete,
    GoalsCanView,
    GoalsCanTrackProgress,
    LearnCanViewCourses,
    LearnCanEnroll,
    LearnCanCreateCourses,
    LearnCanEditCourses,
    LearnCanDeleteCourses,
    LearnCanManageModules,
    CodeCanRead,
    CodeCanWrite,
    CodeCanDelete,
    CodeCanExecute,
    CodeCanManageGit,
    CodeCanCommit,
    CodeCanPush,
    CodeCanDeploy,
    DatabaseCanQuery,
    DatabaseCanReadTables,
    DatabaseCanWriteTables,
    DatabaseCanDeleteTables,
    DatabaseCanAdminTables,
    DatabaseCanManageMigrations,
    TemplatesCanView,
    TemplatesCanCreate,
    TemplatesCanEdit,
    TemplatesCanDelete,
    TemplatesCanApply,
    ListsCanView,
    ListsCanCreate,
    ListsCanEdit,
    ListsCanDelete,
    ListsCanExport,
    ListsCanImport,
}

impl Permission {
    /// Returns the standard string alias format: "domain:action:resource"
    pub fn as_alias(&self) -> &str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Admin => "admin",
            Self::ManageUsers => "admin:manage:users",
            Self::ManageBots => "bot:manage:*",
            Self::ViewAnalytics => "analytics:view:*",
            Self::ManageSettings => "admin:manage:settings",
            Self::ExecuteTasks => "tasks:execute:*",
            Self::ViewLogs => "security:view:logs",
            Self::ManageSecrets => "security:manage:secrets",
            Self::AccessApi => "api:access:*",
            Self::ManageFiles => "drive:manage:*",
            Self::SendMessages => "chat:send:messages",
            Self::ViewConversations => "chat:view:conversations",
            Self::ManageWebhooks => "integrations:manage:webhooks",
            Self::ManageIntegrations => "integrations:manage:*",
            Self::AdministrationCanManageOrganization => "admin:manage:organization",
            Self::AdministrationCanManageMembers => "admin:manage:members",
            Self::AdministrationCanViewMembers => "admin:view:members",
            Self::AdministrationCanManageSettings => "admin:manage:settings",
            Self::AdministrationCanManageBilling => "admin:manage:billing",
            Self::AdministrationCanViewBilling => "admin:view:billing",
            Self::AdministrationCanManageAuditLog => "admin:manage:auditlog",
            Self::AdministrationCanViewAuditLog => "admin:view:auditlog",
            Self::AdministrationCanManageDns => "admin:manage:dns",
            Self::AdministrationCanManageOnboarding => "admin:manage:onboarding",
            Self::AdministrationCanManageRoles => "admin:manage:roles",
            Self::AdministrationCanManageGroups => "admin:manage:groups",
            Self::ComplianceCanViewDashboard => "compliance:view:dashboard",
            Self::ComplianceCanManagePolicies => "compliance:manage:policies",
            Self::ComplianceCanViewReports => "compliance:view:reports",
            Self::ComplianceCanExportReports => "compliance:export:reports",
            Self::ComplianceCanManageDataRetention => "compliance:manage:dataretention",
            Self::ComplianceCanManageGdpr => "compliance:manage:gdpr",
            Self::ComplianceCanManageHipaa => "compliance:manage:hipaa",
            Self::ComplianceCanManageIso27001 => "compliance:manage:iso27001",
            Self::ComplianceCanManageSoc2 => "compliance:manage:soc2",
            Self::SecurityCanManageUsers => "security:manage:users",
            Self::SecurityCanManageSecrets => "security:manage:secrets",
            Self::SecurityCanViewLogs => "security:view:logs",
            Self::SecurityCanManageApiKeys => "security:manage:apikeys",
            Self::SecurityCanManageIpSafelist => "security:manage:ipsafelist",
            Self::SecurityCanManageMfa => "security:manage:mfa",
            Self::SecurityCanManageSessions => "security:manage:sessions",
            Self::SecurityCanManageEncryption => "security:manage:encryption",
            Self::SecurityCanManageIntegrations => "security:manage:integrations",
            Self::SecurityCanManagePasswordPolicy => "security:manage:passwordpolicy",
            Self::SecurityCanConfigure => "security:configure:*",
            Self::SecurityCanAdmin => "security:admin:*",
            Self::MailCanRead => "mail:read:*",
            Self::MailCanSend => "mail:send:*",
            Self::MailCanDelete => "mail:delete:*",
            Self::MailCanManageFolders => "mail:manage:folders",
            Self::MailCanManageFilters => "mail:manage:filters",
            Self::MailCanManageTemplates => "mail:manage:templates",
            Self::MailCanManageSignatures => "mail:manage:signatures",
            Self::MailCanManageAutoReply => "mail:manage:autoreply",
            Self::MailCanManageForwarding => "mail:manage:forwarding",
            Self::MailCanSendCampaigns => "mail:send:campaigns",
            Self::MailCanAdmin => "mail:admin:*",
            Self::MailCanConfigure => "mail:configure:*",
            Self::CalendarCanRead => "calendar:read:*",
            Self::CalendarCanCreate => "calendar:create:*",
            Self::CalendarCanUpdate => "calendar:update:*",
            Self::CalendarCanDelete => "calendar:delete:*",
            Self::CalendarCanManageCalendars => "calendar:manage:calendars",
            Self::CalendarCanShare => "calendar:share:*",
            Self::CalendarCanManageReminders => "calendar:manage:reminders",
            Self::CalendarCanViewAvailability => "calendar:view:availability",
            Self::DriveCanRead => "drive:read:*",
            Self::DriveCanWrite => "drive:write:*",
            Self::DriveCanDelete => "drive:delete:*",
            Self::DriveCanUpload => "drive:upload:*",
            Self::DriveCanDownload => "drive:download:*",
            Self::DriveCanManageFolders => "drive:manage:folders",
            Self::DriveCanShare => "drive:share:*",
            Self::DriveCanManagePermissions => "drive:manage:permissions",
            Self::DriveCanAdmin => "drive:admin:*",
            Self::DriveCanManageVersions => "drive:manage:versions",
            Self::DocumentsCanRead => "documents:read:*",
            Self::DocumentsCanCreate => "documents:create:*",
            Self::DocumentsCanUpdate => "documents:update:*",
            Self::DocumentsCanDelete => "documents:delete:*",
            Self::DocumentsCanShare => "documents:share:*",
            Self::DocumentsCanExport => "documents:export:*",
            Self::DocumentsCanManageTemplates => "documents:manage:templates",
            Self::DocumentsCanManageFolders => "documents:manage:folders",
            Self::DocumentsCanComment => "documents:comment:*",
            Self::DocumentsCanTrackChanges => "documents:track:changes",
            Self::SpreadsheetsCanRead => "spreadsheets:read:*",
            Self::SpreadsheetsCanCreate => "spreadsheets:create:*",
            Self::SpreadsheetsCanUpdate => "spreadsheets:update:*",
            Self::SpreadsheetsCanDelete => "spreadsheets:delete:*",
            Self::SpreadsheetsCanShare => "spreadsheets:share:*",
            Self::SpreadsheetsCanExport => "spreadsheets:export:*",
            Self::SpreadsheetsCanImport => "spreadsheets:import:*",
            Self::SpreadsheetsCanManageFormulas => "spreadsheets:manage:formulas",
            Self::SpreadsheetsCanManageCharts => "spreadsheets:manage:charts",
            Self::PresentationsCanRead => "presentations:read:*",
            Self::PresentationsCanCreate => "presentations:create:*",
            Self::PresentationsCanUpdate => "presentations:update:*",
            Self::PresentationsCanDelete => "presentations:delete:*",
            Self::PresentationsCanPresent => "presentations:present:*",
            Self::PresentationsCanExport => "presentations:export:*",
            Self::MeetingsCanCreate => "meetings:create:*",
            Self::MeetingsCanJoin => "meetings:join:*",
            Self::MeetingsCanManageRooms => "meetings:manage:rooms",
            Self::MeetingsCanRecord => "meetings:record:*",
            Self::MeetingsCanShareScreen => "meetings:share:screen",
            Self::MeetingsCanManageParticipants => "meetings:manage:participants",
            Self::MeetingsCanManageSettings => "meetings:manage:settings",
            Self::MeetingsCanAdmin => "meetings:admin:*",
            Self::ChatCanSendMessages => "chat:send:messages",
            Self::ChatCanReadMessages => "chat:read:messages",
            Self::ChatCanDeleteMessages => "chat:delete:messages",
            Self::ChatCanViewConversations => "chat:view:conversations",
            Self::ChatCanManageBots => "chat:manage:bots",
            Self::ChatCanCreateBots => "chat:create:bots",
            Self::ChatCanEditBots => "chat:edit:bots",
            Self::ChatCanDeleteBots => "chat:delete:bots",
            Self::ChatCanPublishBots => "chat:publish:bots",
            Self::ChatCanViewBots => "chat:view:bots",
            Self::ChatCanExecuteTools => "chat:execute:tools",
            Self::ChatCanManageKnowledgeBase => "chat:manage:knowledgebase",
            Self::ChatCanReadKnowledgeBase => "chat:read:knowledgebase",
            Self::ChatCanWriteKnowledgeBase => "chat:write:knowledgebase",
            Self::ChatCanAdminKnowledgeBase => "chat:admin:knowledgebase",
            Self::ChatCanConfigureBots => "chat:configure:bots",
            Self::TasksCanCreate => "tasks:create:*",
            Self::TasksCanRead => "tasks:read:*",
            Self::TasksCanUpdate => "tasks:update:*",
            Self::TasksCanDelete => "tasks:delete:*",
            Self::TasksCanExecute => "tasks:execute:*",
            Self::TasksCanAssign => "tasks:assign:*",
            Self::TasksCanManageProjects => "tasks:manage:projects",
            Self::TasksCanManageWorkflows => "tasks:manage:workflows",
            Self::TasksCanManageAutoTask => "tasks:manage:autotask",
            Self::AiToolsCanManageLlm => "aitools:manage:llm",
            Self::AiToolsCanConfigureLlm => "aitools:configure:llm",
            Self::AiToolsCanManageModels => "aitools:manage:models",
            Self::AiToolsCanManagePrompts => "aitools:manage:prompts",
            Self::AiToolsCanDesignScripts => "aitools:design:scripts",
            Self::AiToolsCanEditScripts => "aitools:edit:scripts",
            Self::AiToolsCanManageAutotask => "aitools:manage:autotask",
            Self::AiToolsCanManageTraining => "aitools:manage:training",
            Self::AiToolsCanManageVibe => "aitools:manage:vibe",
            Self::AiToolsCanManageMcp => "aitools:manage:mcp",
            Self::BusinessIntelligenceCanViewDashboard => "analytics:view:dashboard",
            Self::BusinessIntelligenceCanViewReports => "analytics:view:reports",
            Self::BusinessIntelligenceCanCreateReports => "analytics:create:reports",
            Self::BusinessIntelligenceCanEditReports => "analytics:edit:reports",
            Self::BusinessIntelligenceCanDeleteReports => "analytics:delete:reports",
            Self::BusinessIntelligenceCanExportReports => "analytics:export:reports",
            Self::BusinessIntelligenceCanViewMetrics => "analytics:view:metrics",
            Self::BusinessIntelligenceCanManageDashboards => "analytics:manage:dashboards",
            Self::BusinessIntelligenceCanTrace => "analytics:trace:*",
            Self::BusinessIntelligenceCanMonitorPerformance => "analytics:monitor:performance",
            Self::BusinessIntelligenceCanViewAnalytics => "analytics:view:analytics",
            Self::BusinessIntelligenceCanExportAnalytics => "analytics:export:analytics",
            Self::IntegrationsCanManageWebhooks => "integrations:manage:webhooks",
            Self::IntegrationsCanManageApiKeys => "integrations:manage:apikeys",
            Self::IntegrationsCanConnectSources => "integrations:connect:sources",
            Self::IntegrationsCanManageSocialMedia => "integrations:manage:socialmedia",
            Self::IntegrationsCanManageWhatsApp => "integrations:manage:whatsapp",
            Self::IntegrationsCanManageTelegram => "integrations:manage:telegram",
            Self::IntegrationsCanManageMsTeams => "integrations:manage:msteams",
            Self::IntegrationsCanManageInstagram => "integrations:manage:instagram",
            Self::IntegrationsCanManageImap => "integrations:manage:imap",
            Self::IntegrationsCanManageGoogle => "integrations:manage:google",
            Self::IntegrationsCanManageMicrosoft => "integrations:manage:microsoft",
            Self::IntegrationsCanManageChannels => "integrations:manage:channels",
            Self::AutomationCanCreateWorkflows => "automation:create:workflows",
            Self::AutomationCanEditWorkflows => "automation:edit:workflows",
            Self::AutomationCanDeleteWorkflows => "automation:delete:workflows",
            Self::AutomationCanExecuteWorkflows => "automation:execute:workflows",
            Self::AutomationCanManageTriggers => "automation:manage:triggers",
            Self::AutomationCanManageSchedules => "automation:manage:schedules",
            Self::AutomationCanManageEventHandlers => "automation:manage:eventhandlers",
            Self::CrmCanViewPipeline => "crm:view:pipeline",
            Self::CrmCanManageLeads => "crm:manage:leads",
            Self::CrmCanManageContacts => "crm:manage:contacts",
            Self::CrmCanManageDeals => "crm:manage:deals",
            Self::CrmCanViewReports => "crm:view:reports",
            Self::CrmCanExportReports => "crm:export:reports",
            Self::CrmCanManageForecast => "crm:manage:forecast",
            Self::CampaignsCanCreate => "campaigns:create:*",
            Self::CampaignsCanEdit => "campaigns:edit:*",
            Self::CampaignsCanDelete => "campaigns:delete:*",
            Self::CampaignsCanExecute => "campaigns:execute:*",
            Self::CampaignsCanViewAnalytics => "campaigns:view:analytics",
            Self::CampaignsCanManageSegments => "campaigns:manage:segments",
            Self::ProductsCanViewCatalog => "products:view:catalog",
            Self::ProductsCanCreateProducts => "products:create:products",
            Self::ProductsCanEditProducts => "products:edit:products",
            Self::ProductsCanDeleteProducts => "products:delete:products",
            Self::ProductsCanCreateServices => "products:create:services",
            Self::ProductsCanEditServices => "products:edit:services",
            Self::ProductsCanDeleteServices => "products:delete:services",
            Self::ProductsCanManagePriceLists => "products:manage:pricelists",
            Self::TicketsCanCreate => "tickets:create:*",
            Self::TicketsCanRead => "tickets:read:*",
            Self::TicketsCanUpdate => "tickets:update:*",
            Self::TicketsCanDelete => "tickets:delete:*",
            Self::TicketsCanAssign => "tickets:assign:*",
            Self::TicketsCanResolve => "tickets:resolve:*",
            Self::TicketsCanManagePriorities => "tickets:manage:priorities",
            Self::TicketsCanViewAnalytics => "tickets:view:analytics",
            Self::TicketsCanManageAttendant => "tickets:manage:attendant",
            Self::PeopleCanViewDirectory => "people:view:directory",
            Self::PeopleCanManageContacts => "people:manage:contacts",
            Self::PeopleCanManageGroups => "people:manage:groups",
            Self::PeopleCanManageRoles => "people:manage:roles",
            Self::PeopleCanImportContacts => "people:import:contacts",
            Self::BrowserCanNavigate => "browser:navigate:*",
            Self::BrowserCanBookmark => "browser:bookmark:*",
            Self::BrowserCanManageHistory => "browser:manage:history",
            Self::BrowserCanDownload => "browser:download:*",
            Self::TerminalCanExecute => "terminal:execute:*",
            Self::TerminalCanViewOutput => "terminal:view:output",
            Self::TerminalCanManageSessions => "terminal:manage:sessions",
            Self::ResearchCanSearch => "research:search:*",
            Self::ResearchCanManageSources => "research:manage:sources",
            Self::ResearchCanExportResults => "research:export:results",
            Self::ResearchCanManageSessions => "research:manage:sessions",
            Self::SocialCanPost => "social:post:*",
            Self::SocialCanSchedulePosts => "social:schedule:posts",
            Self::SocialCanViewFeed => "social:view:feed",
            Self::SocialCanManageAccounts => "social:manage:accounts",
            Self::SocialCanViewAnalytics => "social:view:analytics",
            Self::VideoCanUpload => "video:upload:*",
            Self::VideoCanPlay => "video:play:*",
            Self::VideoCanEdit => "video:edit:*",
            Self::VideoCanDelete => "video:delete:*",
            Self::VideoCanManageLibrary => "video:manage:library",
            Self::CanvasCanCreate => "canvas:create:*",
            Self::CanvasCanEdit => "canvas:edit:*",
            Self::CanvasCanView => "canvas:view:*",
            Self::CanvasCanDelete => "canvas:delete:*",
            Self::CanvasCanExport => "canvas:export:*",
            Self::WorkspaceCanCreateSites => "workspace:create:sites",
            Self::WorkspaceCanEditSites => "workspace:edit:sites",
            Self::WorkspaceCanDeleteSites => "workspace:delete:sites",
            Self::WorkspaceCanViewSites => "workspace:view:sites",
            Self::WorkspaceCanManagePages => "workspace:manage:pages",
            Self::WorkspaceCanManageDatabases => "workspace:manage:databases",
            Self::GoalsCanCreate => "goals:create:*",
            Self::GoalsCanEdit => "goals:edit:*",
            Self::GoalsCanDelete => "goals:delete:*",
            Self::GoalsCanView => "goals:view:*",
            Self::GoalsCanTrackProgress => "goals:track:progress",
            Self::LearnCanViewCourses => "learn:view:courses",
            Self::LearnCanEnroll => "learn:enroll:*",
            Self::LearnCanCreateCourses => "learn:create:courses",
            Self::LearnCanEditCourses => "learn:edit:courses",
            Self::LearnCanDeleteCourses => "learn:delete:courses",
            Self::LearnCanManageModules => "learn:manage:modules",
            Self::CodeCanRead => "code:read:*",
            Self::CodeCanWrite => "code:write:*",
            Self::CodeCanDelete => "code:delete:*",
            Self::CodeCanExecute => "code:execute:*",
            Self::CodeCanManageGit => "code:manage:git",
            Self::CodeCanCommit => "code:commit:*",
            Self::CodeCanPush => "code:push:*",
            Self::CodeCanDeploy => "code:deploy:*",
            Self::DatabaseCanQuery => "database:query:*",
            Self::DatabaseCanReadTables => "database:read:tables",
            Self::DatabaseCanWriteTables => "database:write:tables",
            Self::DatabaseCanDeleteTables => "database:delete:tables",
            Self::DatabaseCanAdminTables => "database:admin:tables",
            Self::DatabaseCanManageMigrations => "database:manage:migrations",
            Self::TemplatesCanView => "templates:view:*",
            Self::TemplatesCanCreate => "templates:create:*",
            Self::TemplatesCanEdit => "templates:edit:*",
            Self::TemplatesCanDelete => "templates:delete:*",
            Self::TemplatesCanApply => "templates:apply:*",
            Self::ListsCanView => "lists:view:*",
            Self::ListsCanCreate => "lists:create:*",
            Self::ListsCanEdit => "lists:edit:*",
            Self::ListsCanDelete => "lists:delete:*",
            Self::ListsCanExport => "lists:export:*",
            Self::ListsCanImport => "lists:import:*",
        }
    }

    /// Tries to resolve a Permission variant from a string alias or variant name
    pub fn from_alias(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "delete" => Some(Self::Delete),
            "admin" => Some(Self::Admin),
            "admin:manage:users" => Some(Self::ManageUsers),
            "bot:manage:*" => Some(Self::ManageBots),
            "analytics:view:*" => Some(Self::ViewAnalytics),
            "admin:manage:settings" => Some(Self::ManageSettings),
            "tasks:execute:*" => Some(Self::ExecuteTasks),
            "security:view:logs" => Some(Self::ViewLogs),
            "security:manage:secrets" => Some(Self::ManageSecrets),
            "api:access:*" => Some(Self::AccessApi),
            "drive:manage:*" => Some(Self::ManageFiles),
            "chat:send:messages" => Some(Self::SendMessages),
            "chat:view:conversations" => Some(Self::ViewConversations),
            "integrations:manage:webhooks" => Some(Self::ManageWebhooks),
            "integrations:manage:*" => Some(Self::ManageIntegrations),
            "admin:manage:organization" => Some(Self::AdministrationCanManageOrganization),
            "admin.manage.organization" => Some(Self::AdministrationCanManageOrganization),
            "admin:manage:members" => Some(Self::AdministrationCanManageMembers),
            "admin.manage.members" => Some(Self::AdministrationCanManageMembers),
            "admin:view:members" => Some(Self::AdministrationCanViewMembers),
            "admin.view.members" => Some(Self::AdministrationCanViewMembers),
            "admin.manage.settings" => Some(Self::AdministrationCanManageSettings),
            "admin:manage:billing" => Some(Self::AdministrationCanManageBilling),
            "admin.manage.billing" => Some(Self::AdministrationCanManageBilling),
            "admin:view:billing" => Some(Self::AdministrationCanViewBilling),
            "admin.view.billing" => Some(Self::AdministrationCanViewBilling),
            "admin:manage:auditlog" => Some(Self::AdministrationCanManageAuditLog),
            "admin.manage.auditlog" => Some(Self::AdministrationCanManageAuditLog),
            "admin:view:auditlog" => Some(Self::AdministrationCanViewAuditLog),
            "admin.view.auditlog" => Some(Self::AdministrationCanViewAuditLog),
            "admin:manage:dns" => Some(Self::AdministrationCanManageDns),
            "admin.manage.dns" => Some(Self::AdministrationCanManageDns),
            "admin:manage:onboarding" => Some(Self::AdministrationCanManageOnboarding),
            "admin.manage.onboarding" => Some(Self::AdministrationCanManageOnboarding),
            "admin:manage:roles" => Some(Self::AdministrationCanManageRoles),
            "admin.manage.roles" => Some(Self::AdministrationCanManageRoles),
            "admin:manage:groups" => Some(Self::AdministrationCanManageGroups),
            "admin.manage.groups" => Some(Self::AdministrationCanManageGroups),
            "compliance:view:dashboard" => Some(Self::ComplianceCanViewDashboard),
            "compliance.view.dashboard" => Some(Self::ComplianceCanViewDashboard),
            "compliance:manage:policies" => Some(Self::ComplianceCanManagePolicies),
            "compliance.manage.policies" => Some(Self::ComplianceCanManagePolicies),
            "compliance:view:reports" => Some(Self::ComplianceCanViewReports),
            "compliance.view.reports" => Some(Self::ComplianceCanViewReports),
            "compliance:export:reports" => Some(Self::ComplianceCanExportReports),
            "compliance.export.reports" => Some(Self::ComplianceCanExportReports),
            "compliance:manage:dataretention" => Some(Self::ComplianceCanManageDataRetention),
            "compliance.manage.dataretention" => Some(Self::ComplianceCanManageDataRetention),
            "compliance:manage:gdpr" => Some(Self::ComplianceCanManageGdpr),
            "compliance.manage.gdpr" => Some(Self::ComplianceCanManageGdpr),
            "compliance:manage:hipaa" => Some(Self::ComplianceCanManageHipaa),
            "compliance.manage.hipaa" => Some(Self::ComplianceCanManageHipaa),
            "compliance:manage:iso27001" => Some(Self::ComplianceCanManageIso27001),
            "compliance.manage.iso27001" => Some(Self::ComplianceCanManageIso27001),
            "compliance:manage:soc2" => Some(Self::ComplianceCanManageSoc2),
            "compliance.manage.soc2" => Some(Self::ComplianceCanManageSoc2),
            "security:manage:users" => Some(Self::SecurityCanManageUsers),
            "security.manage.users" => Some(Self::SecurityCanManageUsers),
            "security.manage.secrets" => Some(Self::SecurityCanManageSecrets),
            "security.view.logs" => Some(Self::SecurityCanViewLogs),
            "security:manage:apikeys" => Some(Self::SecurityCanManageApiKeys),
            "security.manage.apikeys" => Some(Self::SecurityCanManageApiKeys),
            "security:manage:ipsafelist" => Some(Self::SecurityCanManageIpSafelist),
            "security.manage.ipsafelist" => Some(Self::SecurityCanManageIpSafelist),
            "security:manage:mfa" => Some(Self::SecurityCanManageMfa),
            "security.manage.mfa" => Some(Self::SecurityCanManageMfa),
            "security:manage:sessions" => Some(Self::SecurityCanManageSessions),
            "security.manage.sessions" => Some(Self::SecurityCanManageSessions),
            "security:manage:encryption" => Some(Self::SecurityCanManageEncryption),
            "security.manage.encryption" => Some(Self::SecurityCanManageEncryption),
            "security:manage:integrations" => Some(Self::SecurityCanManageIntegrations),
            "security.manage.integrations" => Some(Self::SecurityCanManageIntegrations),
            "security:manage:passwordpolicy" => Some(Self::SecurityCanManagePasswordPolicy),
            "security.manage.passwordpolicy" => Some(Self::SecurityCanManagePasswordPolicy),
            "security:configure:*" => Some(Self::SecurityCanConfigure),
            "security.configure.*" => Some(Self::SecurityCanConfigure),
            "security:admin:*" => Some(Self::SecurityCanAdmin),
            "security.admin.*" => Some(Self::SecurityCanAdmin),
            "mail:read:*" => Some(Self::MailCanRead),
            "mail.read.*" => Some(Self::MailCanRead),
            "mail:send:*" => Some(Self::MailCanSend),
            "mail.send.*" => Some(Self::MailCanSend),
            "mail:delete:*" => Some(Self::MailCanDelete),
            "mail.delete.*" => Some(Self::MailCanDelete),
            "mail:manage:folders" => Some(Self::MailCanManageFolders),
            "mail.manage.folders" => Some(Self::MailCanManageFolders),
            "mail:manage:filters" => Some(Self::MailCanManageFilters),
            "mail.manage.filters" => Some(Self::MailCanManageFilters),
            "mail:manage:templates" => Some(Self::MailCanManageTemplates),
            "mail.manage.templates" => Some(Self::MailCanManageTemplates),
            "mail:manage:signatures" => Some(Self::MailCanManageSignatures),
            "mail.manage.signatures" => Some(Self::MailCanManageSignatures),
            "mail:manage:autoreply" => Some(Self::MailCanManageAutoReply),
            "mail.manage.autoreply" => Some(Self::MailCanManageAutoReply),
            "mail:manage:forwarding" => Some(Self::MailCanManageForwarding),
            "mail.manage.forwarding" => Some(Self::MailCanManageForwarding),
            "mail:send:campaigns" => Some(Self::MailCanSendCampaigns),
            "mail.send.campaigns" => Some(Self::MailCanSendCampaigns),
            "mail:admin:*" => Some(Self::MailCanAdmin),
            "mail.admin.*" => Some(Self::MailCanAdmin),
            "mail:configure:*" => Some(Self::MailCanConfigure),
            "mail.configure.*" => Some(Self::MailCanConfigure),
            "calendar:read:*" => Some(Self::CalendarCanRead),
            "calendar.read.*" => Some(Self::CalendarCanRead),
            "calendar:create:*" => Some(Self::CalendarCanCreate),
            "calendar.create.*" => Some(Self::CalendarCanCreate),
            "calendar:update:*" => Some(Self::CalendarCanUpdate),
            "calendar.update.*" => Some(Self::CalendarCanUpdate),
            "calendar:delete:*" => Some(Self::CalendarCanDelete),
            "calendar.delete.*" => Some(Self::CalendarCanDelete),
            "calendar:manage:calendars" => Some(Self::CalendarCanManageCalendars),
            "calendar.manage.calendars" => Some(Self::CalendarCanManageCalendars),
            "calendar:share:*" => Some(Self::CalendarCanShare),
            "calendar.share.*" => Some(Self::CalendarCanShare),
            "calendar:manage:reminders" => Some(Self::CalendarCanManageReminders),
            "calendar.manage.reminders" => Some(Self::CalendarCanManageReminders),
            "calendar:view:availability" => Some(Self::CalendarCanViewAvailability),
            "calendar.view.availability" => Some(Self::CalendarCanViewAvailability),
            "drive:read:*" => Some(Self::DriveCanRead),
            "drive.read.*" => Some(Self::DriveCanRead),
            "drive:write:*" => Some(Self::DriveCanWrite),
            "drive.write.*" => Some(Self::DriveCanWrite),
            "drive:delete:*" => Some(Self::DriveCanDelete),
            "drive.delete.*" => Some(Self::DriveCanDelete),
            "drive:upload:*" => Some(Self::DriveCanUpload),
            "drive.upload.*" => Some(Self::DriveCanUpload),
            "drive:download:*" => Some(Self::DriveCanDownload),
            "drive.download.*" => Some(Self::DriveCanDownload),
            "drive:manage:folders" => Some(Self::DriveCanManageFolders),
            "drive.manage.folders" => Some(Self::DriveCanManageFolders),
            "drive:share:*" => Some(Self::DriveCanShare),
            "drive.share.*" => Some(Self::DriveCanShare),
            "drive:manage:permissions" => Some(Self::DriveCanManagePermissions),
            "drive.manage.permissions" => Some(Self::DriveCanManagePermissions),
            "drive:admin:*" => Some(Self::DriveCanAdmin),
            "drive.admin.*" => Some(Self::DriveCanAdmin),
            "drive:manage:versions" => Some(Self::DriveCanManageVersions),
            "drive.manage.versions" => Some(Self::DriveCanManageVersions),
            "documents:read:*" => Some(Self::DocumentsCanRead),
            "documents.read.*" => Some(Self::DocumentsCanRead),
            "documents:create:*" => Some(Self::DocumentsCanCreate),
            "documents.create.*" => Some(Self::DocumentsCanCreate),
            "documents:update:*" => Some(Self::DocumentsCanUpdate),
            "documents.update.*" => Some(Self::DocumentsCanUpdate),
            "documents:delete:*" => Some(Self::DocumentsCanDelete),
            "documents.delete.*" => Some(Self::DocumentsCanDelete),
            "documents:share:*" => Some(Self::DocumentsCanShare),
            "documents.share.*" => Some(Self::DocumentsCanShare),
            "documents:export:*" => Some(Self::DocumentsCanExport),
            "documents.export.*" => Some(Self::DocumentsCanExport),
            "documents:manage:templates" => Some(Self::DocumentsCanManageTemplates),
            "documents.manage.templates" => Some(Self::DocumentsCanManageTemplates),
            "documents:manage:folders" => Some(Self::DocumentsCanManageFolders),
            "documents.manage.folders" => Some(Self::DocumentsCanManageFolders),
            "documents:comment:*" => Some(Self::DocumentsCanComment),
            "documents.comment.*" => Some(Self::DocumentsCanComment),
            "documents:track:changes" => Some(Self::DocumentsCanTrackChanges),
            "documents.track.changes" => Some(Self::DocumentsCanTrackChanges),
            "spreadsheets:read:*" => Some(Self::SpreadsheetsCanRead),
            "spreadsheets.read.*" => Some(Self::SpreadsheetsCanRead),
            "spreadsheets:create:*" => Some(Self::SpreadsheetsCanCreate),
            "spreadsheets.create.*" => Some(Self::SpreadsheetsCanCreate),
            "spreadsheets:update:*" => Some(Self::SpreadsheetsCanUpdate),
            "spreadsheets.update.*" => Some(Self::SpreadsheetsCanUpdate),
            "spreadsheets:delete:*" => Some(Self::SpreadsheetsCanDelete),
            "spreadsheets.delete.*" => Some(Self::SpreadsheetsCanDelete),
            "spreadsheets:share:*" => Some(Self::SpreadsheetsCanShare),
            "spreadsheets.share.*" => Some(Self::SpreadsheetsCanShare),
            "spreadsheets:export:*" => Some(Self::SpreadsheetsCanExport),
            "spreadsheets.export.*" => Some(Self::SpreadsheetsCanExport),
            "spreadsheets:import:*" => Some(Self::SpreadsheetsCanImport),
            "spreadsheets.import.*" => Some(Self::SpreadsheetsCanImport),
            "spreadsheets:manage:formulas" => Some(Self::SpreadsheetsCanManageFormulas),
            "spreadsheets.manage.formulas" => Some(Self::SpreadsheetsCanManageFormulas),
            "spreadsheets:manage:charts" => Some(Self::SpreadsheetsCanManageCharts),
            "spreadsheets.manage.charts" => Some(Self::SpreadsheetsCanManageCharts),
            "presentations:read:*" => Some(Self::PresentationsCanRead),
            "presentations.read.*" => Some(Self::PresentationsCanRead),
            "presentations:create:*" => Some(Self::PresentationsCanCreate),
            "presentations.create.*" => Some(Self::PresentationsCanCreate),
            "presentations:update:*" => Some(Self::PresentationsCanUpdate),
            "presentations.update.*" => Some(Self::PresentationsCanUpdate),
            "presentations:delete:*" => Some(Self::PresentationsCanDelete),
            "presentations.delete.*" => Some(Self::PresentationsCanDelete),
            "presentations:present:*" => Some(Self::PresentationsCanPresent),
            "presentations.present.*" => Some(Self::PresentationsCanPresent),
            "presentations:export:*" => Some(Self::PresentationsCanExport),
            "presentations.export.*" => Some(Self::PresentationsCanExport),
            "meetings:create:*" => Some(Self::MeetingsCanCreate),
            "meetings.create.*" => Some(Self::MeetingsCanCreate),
            "meetings:join:*" => Some(Self::MeetingsCanJoin),
            "meetings.join.*" => Some(Self::MeetingsCanJoin),
            "meetings:manage:rooms" => Some(Self::MeetingsCanManageRooms),
            "meetings.manage.rooms" => Some(Self::MeetingsCanManageRooms),
            "meetings:record:*" => Some(Self::MeetingsCanRecord),
            "meetings.record.*" => Some(Self::MeetingsCanRecord),
            "meetings:share:screen" => Some(Self::MeetingsCanShareScreen),
            "meetings.share.screen" => Some(Self::MeetingsCanShareScreen),
            "meetings:manage:participants" => Some(Self::MeetingsCanManageParticipants),
            "meetings.manage.participants" => Some(Self::MeetingsCanManageParticipants),
            "meetings:manage:settings" => Some(Self::MeetingsCanManageSettings),
            "meetings.manage.settings" => Some(Self::MeetingsCanManageSettings),
            "meetings:admin:*" => Some(Self::MeetingsCanAdmin),
            "meetings.admin.*" => Some(Self::MeetingsCanAdmin),
            "chat.send.messages" => Some(Self::ChatCanSendMessages),
            "chat:read:messages" => Some(Self::ChatCanReadMessages),
            "chat.read.messages" => Some(Self::ChatCanReadMessages),
            "chat:delete:messages" => Some(Self::ChatCanDeleteMessages),
            "chat.delete.messages" => Some(Self::ChatCanDeleteMessages),
            "chat.view.conversations" => Some(Self::ChatCanViewConversations),
            "chat:manage:bots" => Some(Self::ChatCanManageBots),
            "chat.manage.bots" => Some(Self::ChatCanManageBots),
            "chat:create:bots" => Some(Self::ChatCanCreateBots),
            "chat.create.bots" => Some(Self::ChatCanCreateBots),
            "chat:edit:bots" => Some(Self::ChatCanEditBots),
            "chat.edit.bots" => Some(Self::ChatCanEditBots),
            "chat:delete:bots" => Some(Self::ChatCanDeleteBots),
            "chat.delete.bots" => Some(Self::ChatCanDeleteBots),
            "chat:publish:bots" => Some(Self::ChatCanPublishBots),
            "chat.publish.bots" => Some(Self::ChatCanPublishBots),
            "chat:view:bots" => Some(Self::ChatCanViewBots),
            "chat.view.bots" => Some(Self::ChatCanViewBots),
            "chat:execute:tools" => Some(Self::ChatCanExecuteTools),
            "chat.execute.tools" => Some(Self::ChatCanExecuteTools),
            "chat:manage:knowledgebase" => Some(Self::ChatCanManageKnowledgeBase),
            "chat.manage.knowledgebase" => Some(Self::ChatCanManageKnowledgeBase),
            "chat:read:knowledgebase" => Some(Self::ChatCanReadKnowledgeBase),
            "chat.read.knowledgebase" => Some(Self::ChatCanReadKnowledgeBase),
            "chat:write:knowledgebase" => Some(Self::ChatCanWriteKnowledgeBase),
            "chat.write.knowledgebase" => Some(Self::ChatCanWriteKnowledgeBase),
            "chat:admin:knowledgebase" => Some(Self::ChatCanAdminKnowledgeBase),
            "chat.admin.knowledgebase" => Some(Self::ChatCanAdminKnowledgeBase),
            "chat:configure:bots" => Some(Self::ChatCanConfigureBots),
            "chat.configure.bots" => Some(Self::ChatCanConfigureBots),
            "tasks:create:*" => Some(Self::TasksCanCreate),
            "tasks.create.*" => Some(Self::TasksCanCreate),
            "tasks:read:*" => Some(Self::TasksCanRead),
            "tasks.read.*" => Some(Self::TasksCanRead),
            "tasks:update:*" => Some(Self::TasksCanUpdate),
            "tasks.update.*" => Some(Self::TasksCanUpdate),
            "tasks:delete:*" => Some(Self::TasksCanDelete),
            "tasks.delete.*" => Some(Self::TasksCanDelete),
            "tasks.execute.*" => Some(Self::TasksCanExecute),
            "tasks:assign:*" => Some(Self::TasksCanAssign),
            "tasks.assign.*" => Some(Self::TasksCanAssign),
            "tasks:manage:projects" => Some(Self::TasksCanManageProjects),
            "tasks.manage.projects" => Some(Self::TasksCanManageProjects),
            "tasks:manage:workflows" => Some(Self::TasksCanManageWorkflows),
            "tasks.manage.workflows" => Some(Self::TasksCanManageWorkflows),
            "tasks:manage:autotask" => Some(Self::TasksCanManageAutoTask),
            "tasks.manage.autotask" => Some(Self::TasksCanManageAutoTask),
            "aitools:manage:llm" => Some(Self::AiToolsCanManageLlm),
            "aitools.manage.llm" => Some(Self::AiToolsCanManageLlm),
            "aitools:configure:llm" => Some(Self::AiToolsCanConfigureLlm),
            "aitools.configure.llm" => Some(Self::AiToolsCanConfigureLlm),
            "aitools:manage:models" => Some(Self::AiToolsCanManageModels),
            "aitools.manage.models" => Some(Self::AiToolsCanManageModels),
            "aitools:manage:prompts" => Some(Self::AiToolsCanManagePrompts),
            "aitools.manage.prompts" => Some(Self::AiToolsCanManagePrompts),
            "aitools:design:scripts" => Some(Self::AiToolsCanDesignScripts),
            "aitools.design.scripts" => Some(Self::AiToolsCanDesignScripts),
            "aitools:edit:scripts" => Some(Self::AiToolsCanEditScripts),
            "aitools.edit.scripts" => Some(Self::AiToolsCanEditScripts),
            "aitools:manage:autotask" => Some(Self::AiToolsCanManageAutotask),
            "aitools.manage.autotask" => Some(Self::AiToolsCanManageAutotask),
            "aitools:manage:training" => Some(Self::AiToolsCanManageTraining),
            "aitools.manage.training" => Some(Self::AiToolsCanManageTraining),
            "aitools:manage:vibe" => Some(Self::AiToolsCanManageVibe),
            "aitools.manage.vibe" => Some(Self::AiToolsCanManageVibe),
            "aitools:manage:mcp" => Some(Self::AiToolsCanManageMcp),
            "aitools.manage.mcp" => Some(Self::AiToolsCanManageMcp),
            "analytics:view:dashboard" => Some(Self::BusinessIntelligenceCanViewDashboard),
            "analytics.view.dashboard" => Some(Self::BusinessIntelligenceCanViewDashboard),
            "analytics:view:reports" => Some(Self::BusinessIntelligenceCanViewReports),
            "analytics.view.reports" => Some(Self::BusinessIntelligenceCanViewReports),
            "analytics:create:reports" => Some(Self::BusinessIntelligenceCanCreateReports),
            "analytics.create.reports" => Some(Self::BusinessIntelligenceCanCreateReports),
            "analytics:edit:reports" => Some(Self::BusinessIntelligenceCanEditReports),
            "analytics.edit.reports" => Some(Self::BusinessIntelligenceCanEditReports),
            "analytics:delete:reports" => Some(Self::BusinessIntelligenceCanDeleteReports),
            "analytics.delete.reports" => Some(Self::BusinessIntelligenceCanDeleteReports),
            "analytics:export:reports" => Some(Self::BusinessIntelligenceCanExportReports),
            "analytics.export.reports" => Some(Self::BusinessIntelligenceCanExportReports),
            "analytics:view:metrics" => Some(Self::BusinessIntelligenceCanViewMetrics),
            "analytics.view.metrics" => Some(Self::BusinessIntelligenceCanViewMetrics),
            "analytics:manage:dashboards" => Some(Self::BusinessIntelligenceCanManageDashboards),
            "analytics.manage.dashboards" => Some(Self::BusinessIntelligenceCanManageDashboards),
            "analytics:trace:*" => Some(Self::BusinessIntelligenceCanTrace),
            "analytics.trace.*" => Some(Self::BusinessIntelligenceCanTrace),
            "analytics:monitor:performance" => Some(Self::BusinessIntelligenceCanMonitorPerformance),
            "analytics.monitor.performance" => Some(Self::BusinessIntelligenceCanMonitorPerformance),
            "analytics:view:analytics" => Some(Self::BusinessIntelligenceCanViewAnalytics),
            "analytics.view.analytics" => Some(Self::BusinessIntelligenceCanViewAnalytics),
            "analytics:export:analytics" => Some(Self::BusinessIntelligenceCanExportAnalytics),
            "analytics.export.analytics" => Some(Self::BusinessIntelligenceCanExportAnalytics),
            "integrations.manage.webhooks" => Some(Self::IntegrationsCanManageWebhooks),
            "integrations:manage:apikeys" => Some(Self::IntegrationsCanManageApiKeys),
            "integrations.manage.apikeys" => Some(Self::IntegrationsCanManageApiKeys),
            "integrations:connect:sources" => Some(Self::IntegrationsCanConnectSources),
            "integrations.connect.sources" => Some(Self::IntegrationsCanConnectSources),
            "integrations:manage:socialmedia" => Some(Self::IntegrationsCanManageSocialMedia),
            "integrations.manage.socialmedia" => Some(Self::IntegrationsCanManageSocialMedia),
            "integrations:manage:whatsapp" => Some(Self::IntegrationsCanManageWhatsApp),
            "integrations.manage.whatsapp" => Some(Self::IntegrationsCanManageWhatsApp),
            "integrations:manage:telegram" => Some(Self::IntegrationsCanManageTelegram),
            "integrations.manage.telegram" => Some(Self::IntegrationsCanManageTelegram),
            "integrations:manage:msteams" => Some(Self::IntegrationsCanManageMsTeams),
            "integrations.manage.msteams" => Some(Self::IntegrationsCanManageMsTeams),
            "integrations:manage:instagram" => Some(Self::IntegrationsCanManageInstagram),
            "integrations.manage.instagram" => Some(Self::IntegrationsCanManageInstagram),
            "integrations:manage:imap" => Some(Self::IntegrationsCanManageImap),
            "integrations.manage.imap" => Some(Self::IntegrationsCanManageImap),
            "integrations:manage:google" => Some(Self::IntegrationsCanManageGoogle),
            "integrations.manage.google" => Some(Self::IntegrationsCanManageGoogle),
            "integrations:manage:microsoft" => Some(Self::IntegrationsCanManageMicrosoft),
            "integrations.manage.microsoft" => Some(Self::IntegrationsCanManageMicrosoft),
            "integrations:manage:channels" => Some(Self::IntegrationsCanManageChannels),
            "integrations.manage.channels" => Some(Self::IntegrationsCanManageChannels),
            "automation:create:workflows" => Some(Self::AutomationCanCreateWorkflows),
            "automation.create.workflows" => Some(Self::AutomationCanCreateWorkflows),
            "automation:edit:workflows" => Some(Self::AutomationCanEditWorkflows),
            "automation.edit.workflows" => Some(Self::AutomationCanEditWorkflows),
            "automation:delete:workflows" => Some(Self::AutomationCanDeleteWorkflows),
            "automation.delete.workflows" => Some(Self::AutomationCanDeleteWorkflows),
            "automation:execute:workflows" => Some(Self::AutomationCanExecuteWorkflows),
            "automation.execute.workflows" => Some(Self::AutomationCanExecuteWorkflows),
            "automation:manage:triggers" => Some(Self::AutomationCanManageTriggers),
            "automation.manage.triggers" => Some(Self::AutomationCanManageTriggers),
            "automation:manage:schedules" => Some(Self::AutomationCanManageSchedules),
            "automation.manage.schedules" => Some(Self::AutomationCanManageSchedules),
            "automation:manage:eventhandlers" => Some(Self::AutomationCanManageEventHandlers),
            "automation.manage.eventhandlers" => Some(Self::AutomationCanManageEventHandlers),
            "crm:view:pipeline" => Some(Self::CrmCanViewPipeline),
            "crm.view.pipeline" => Some(Self::CrmCanViewPipeline),
            "crm:manage:leads" => Some(Self::CrmCanManageLeads),
            "crm.manage.leads" => Some(Self::CrmCanManageLeads),
            "crm:manage:contacts" => Some(Self::CrmCanManageContacts),
            "crm.manage.contacts" => Some(Self::CrmCanManageContacts),
            "crm:manage:deals" => Some(Self::CrmCanManageDeals),
            "crm.manage.deals" => Some(Self::CrmCanManageDeals),
            "crm:view:reports" => Some(Self::CrmCanViewReports),
            "crm.view.reports" => Some(Self::CrmCanViewReports),
            "crm:export:reports" => Some(Self::CrmCanExportReports),
            "crm.export.reports" => Some(Self::CrmCanExportReports),
            "crm:manage:forecast" => Some(Self::CrmCanManageForecast),
            "crm.manage.forecast" => Some(Self::CrmCanManageForecast),
            "campaigns:create:*" => Some(Self::CampaignsCanCreate),
            "campaigns.create.*" => Some(Self::CampaignsCanCreate),
            "campaigns:edit:*" => Some(Self::CampaignsCanEdit),
            "campaigns.edit.*" => Some(Self::CampaignsCanEdit),
            "campaigns:delete:*" => Some(Self::CampaignsCanDelete),
            "campaigns.delete.*" => Some(Self::CampaignsCanDelete),
            "campaigns:execute:*" => Some(Self::CampaignsCanExecute),
            "campaigns.execute.*" => Some(Self::CampaignsCanExecute),
            "campaigns:view:analytics" => Some(Self::CampaignsCanViewAnalytics),
            "campaigns.view.analytics" => Some(Self::CampaignsCanViewAnalytics),
            "campaigns:manage:segments" => Some(Self::CampaignsCanManageSegments),
            "campaigns.manage.segments" => Some(Self::CampaignsCanManageSegments),
            "products:view:catalog" => Some(Self::ProductsCanViewCatalog),
            "products.view.catalog" => Some(Self::ProductsCanViewCatalog),
            "products:create:products" => Some(Self::ProductsCanCreateProducts),
            "products.create.products" => Some(Self::ProductsCanCreateProducts),
            "products:edit:products" => Some(Self::ProductsCanEditProducts),
            "products.edit.products" => Some(Self::ProductsCanEditProducts),
            "products:delete:products" => Some(Self::ProductsCanDeleteProducts),
            "products.delete.products" => Some(Self::ProductsCanDeleteProducts),
            "products:create:services" => Some(Self::ProductsCanCreateServices),
            "products.create.services" => Some(Self::ProductsCanCreateServices),
            "products:edit:services" => Some(Self::ProductsCanEditServices),
            "products.edit.services" => Some(Self::ProductsCanEditServices),
            "products:delete:services" => Some(Self::ProductsCanDeleteServices),
            "products.delete.services" => Some(Self::ProductsCanDeleteServices),
            "products:manage:pricelists" => Some(Self::ProductsCanManagePriceLists),
            "products.manage.pricelists" => Some(Self::ProductsCanManagePriceLists),
            "tickets:create:*" => Some(Self::TicketsCanCreate),
            "tickets.create.*" => Some(Self::TicketsCanCreate),
            "tickets:read:*" => Some(Self::TicketsCanRead),
            "tickets.read.*" => Some(Self::TicketsCanRead),
            "tickets:update:*" => Some(Self::TicketsCanUpdate),
            "tickets.update.*" => Some(Self::TicketsCanUpdate),
            "tickets:delete:*" => Some(Self::TicketsCanDelete),
            "tickets.delete.*" => Some(Self::TicketsCanDelete),
            "tickets:assign:*" => Some(Self::TicketsCanAssign),
            "tickets.assign.*" => Some(Self::TicketsCanAssign),
            "tickets:resolve:*" => Some(Self::TicketsCanResolve),
            "tickets.resolve.*" => Some(Self::TicketsCanResolve),
            "tickets:manage:priorities" => Some(Self::TicketsCanManagePriorities),
            "tickets.manage.priorities" => Some(Self::TicketsCanManagePriorities),
            "tickets:view:analytics" => Some(Self::TicketsCanViewAnalytics),
            "tickets.view.analytics" => Some(Self::TicketsCanViewAnalytics),
            "tickets:manage:attendant" => Some(Self::TicketsCanManageAttendant),
            "tickets.manage.attendant" => Some(Self::TicketsCanManageAttendant),
            "people:view:directory" => Some(Self::PeopleCanViewDirectory),
            "people.view.directory" => Some(Self::PeopleCanViewDirectory),
            "people:manage:contacts" => Some(Self::PeopleCanManageContacts),
            "people.manage.contacts" => Some(Self::PeopleCanManageContacts),
            "people:manage:groups" => Some(Self::PeopleCanManageGroups),
            "people.manage.groups" => Some(Self::PeopleCanManageGroups),
            "people:manage:roles" => Some(Self::PeopleCanManageRoles),
            "people.manage.roles" => Some(Self::PeopleCanManageRoles),
            "people:import:contacts" => Some(Self::PeopleCanImportContacts),
            "people.import.contacts" => Some(Self::PeopleCanImportContacts),
            "browser:navigate:*" => Some(Self::BrowserCanNavigate),
            "browser.navigate.*" => Some(Self::BrowserCanNavigate),
            "browser:bookmark:*" => Some(Self::BrowserCanBookmark),
            "browser.bookmark.*" => Some(Self::BrowserCanBookmark),
            "browser:manage:history" => Some(Self::BrowserCanManageHistory),
            "browser.manage.history" => Some(Self::BrowserCanManageHistory),
            "browser:download:*" => Some(Self::BrowserCanDownload),
            "browser.download.*" => Some(Self::BrowserCanDownload),
            "terminal:execute:*" => Some(Self::TerminalCanExecute),
            "terminal.execute.*" => Some(Self::TerminalCanExecute),
            "terminal:view:output" => Some(Self::TerminalCanViewOutput),
            "terminal.view.output" => Some(Self::TerminalCanViewOutput),
            "terminal:manage:sessions" => Some(Self::TerminalCanManageSessions),
            "terminal.manage.sessions" => Some(Self::TerminalCanManageSessions),
            "research:search:*" => Some(Self::ResearchCanSearch),
            "research.search.*" => Some(Self::ResearchCanSearch),
            "research:manage:sources" => Some(Self::ResearchCanManageSources),
            "research.manage.sources" => Some(Self::ResearchCanManageSources),
            "research:export:results" => Some(Self::ResearchCanExportResults),
            "research.export.results" => Some(Self::ResearchCanExportResults),
            "research:manage:sessions" => Some(Self::ResearchCanManageSessions),
            "research.manage.sessions" => Some(Self::ResearchCanManageSessions),
            "social:post:*" => Some(Self::SocialCanPost),
            "social.post.*" => Some(Self::SocialCanPost),
            "social:schedule:posts" => Some(Self::SocialCanSchedulePosts),
            "social.schedule.posts" => Some(Self::SocialCanSchedulePosts),
            "social:view:feed" => Some(Self::SocialCanViewFeed),
            "social.view.feed" => Some(Self::SocialCanViewFeed),
            "social:manage:accounts" => Some(Self::SocialCanManageAccounts),
            "social.manage.accounts" => Some(Self::SocialCanManageAccounts),
            "social:view:analytics" => Some(Self::SocialCanViewAnalytics),
            "social.view.analytics" => Some(Self::SocialCanViewAnalytics),
            "video:upload:*" => Some(Self::VideoCanUpload),
            "video.upload.*" => Some(Self::VideoCanUpload),
            "video:play:*" => Some(Self::VideoCanPlay),
            "video.play.*" => Some(Self::VideoCanPlay),
            "video:edit:*" => Some(Self::VideoCanEdit),
            "video.edit.*" => Some(Self::VideoCanEdit),
            "video:delete:*" => Some(Self::VideoCanDelete),
            "video.delete.*" => Some(Self::VideoCanDelete),
            "video:manage:library" => Some(Self::VideoCanManageLibrary),
            "video.manage.library" => Some(Self::VideoCanManageLibrary),
            "canvas:create:*" => Some(Self::CanvasCanCreate),
            "canvas.create.*" => Some(Self::CanvasCanCreate),
            "canvas:edit:*" => Some(Self::CanvasCanEdit),
            "canvas.edit.*" => Some(Self::CanvasCanEdit),
            "canvas:view:*" => Some(Self::CanvasCanView),
            "canvas.view.*" => Some(Self::CanvasCanView),
            "canvas:delete:*" => Some(Self::CanvasCanDelete),
            "canvas.delete.*" => Some(Self::CanvasCanDelete),
            "canvas:export:*" => Some(Self::CanvasCanExport),
            "canvas.export.*" => Some(Self::CanvasCanExport),
            "workspace:create:sites" => Some(Self::WorkspaceCanCreateSites),
            "workspace.create.sites" => Some(Self::WorkspaceCanCreateSites),
            "workspace:edit:sites" => Some(Self::WorkspaceCanEditSites),
            "workspace.edit.sites" => Some(Self::WorkspaceCanEditSites),
            "workspace:delete:sites" => Some(Self::WorkspaceCanDeleteSites),
            "workspace.delete.sites" => Some(Self::WorkspaceCanDeleteSites),
            "workspace:view:sites" => Some(Self::WorkspaceCanViewSites),
            "workspace.view.sites" => Some(Self::WorkspaceCanViewSites),
            "workspace:manage:pages" => Some(Self::WorkspaceCanManagePages),
            "workspace.manage.pages" => Some(Self::WorkspaceCanManagePages),
            "workspace:manage:databases" => Some(Self::WorkspaceCanManageDatabases),
            "workspace.manage.databases" => Some(Self::WorkspaceCanManageDatabases),
            "goals:create:*" => Some(Self::GoalsCanCreate),
            "goals.create.*" => Some(Self::GoalsCanCreate),
            "goals:edit:*" => Some(Self::GoalsCanEdit),
            "goals.edit.*" => Some(Self::GoalsCanEdit),
            "goals:delete:*" => Some(Self::GoalsCanDelete),
            "goals.delete.*" => Some(Self::GoalsCanDelete),
            "goals:view:*" => Some(Self::GoalsCanView),
            "goals.view.*" => Some(Self::GoalsCanView),
            "goals:track:progress" => Some(Self::GoalsCanTrackProgress),
            "goals.track.progress" => Some(Self::GoalsCanTrackProgress),
            "learn:view:courses" => Some(Self::LearnCanViewCourses),
            "learn.view.courses" => Some(Self::LearnCanViewCourses),
            "learn:enroll:*" => Some(Self::LearnCanEnroll),
            "learn.enroll.*" => Some(Self::LearnCanEnroll),
            "learn:create:courses" => Some(Self::LearnCanCreateCourses),
            "learn.create.courses" => Some(Self::LearnCanCreateCourses),
            "learn:edit:courses" => Some(Self::LearnCanEditCourses),
            "learn.edit.courses" => Some(Self::LearnCanEditCourses),
            "learn:delete:courses" => Some(Self::LearnCanDeleteCourses),
            "learn.delete.courses" => Some(Self::LearnCanDeleteCourses),
            "learn:manage:modules" => Some(Self::LearnCanManageModules),
            "learn.manage.modules" => Some(Self::LearnCanManageModules),
            "code:read:*" => Some(Self::CodeCanRead),
            "code.read.*" => Some(Self::CodeCanRead),
            "code:write:*" => Some(Self::CodeCanWrite),
            "code.write.*" => Some(Self::CodeCanWrite),
            "code:delete:*" => Some(Self::CodeCanDelete),
            "code.delete.*" => Some(Self::CodeCanDelete),
            "code:execute:*" => Some(Self::CodeCanExecute),
            "code.execute.*" => Some(Self::CodeCanExecute),
            "code:manage:git" => Some(Self::CodeCanManageGit),
            "code.manage.git" => Some(Self::CodeCanManageGit),
            "code:commit:*" => Some(Self::CodeCanCommit),
            "code.commit.*" => Some(Self::CodeCanCommit),
            "code:push:*" => Some(Self::CodeCanPush),
            "code.push.*" => Some(Self::CodeCanPush),
            "code:deploy:*" => Some(Self::CodeCanDeploy),
            "code.deploy.*" => Some(Self::CodeCanDeploy),
            "database:query:*" => Some(Self::DatabaseCanQuery),
            "database.query.*" => Some(Self::DatabaseCanQuery),
            "database:read:tables" => Some(Self::DatabaseCanReadTables),
            "database.read.tables" => Some(Self::DatabaseCanReadTables),
            "database:write:tables" => Some(Self::DatabaseCanWriteTables),
            "database.write.tables" => Some(Self::DatabaseCanWriteTables),
            "database:delete:tables" => Some(Self::DatabaseCanDeleteTables),
            "database.delete.tables" => Some(Self::DatabaseCanDeleteTables),
            "database:admin:tables" => Some(Self::DatabaseCanAdminTables),
            "database.admin.tables" => Some(Self::DatabaseCanAdminTables),
            "database:manage:migrations" => Some(Self::DatabaseCanManageMigrations),
            "database.manage.migrations" => Some(Self::DatabaseCanManageMigrations),
            "templates:view:*" => Some(Self::TemplatesCanView),
            "templates.view.*" => Some(Self::TemplatesCanView),
            "templates:create:*" => Some(Self::TemplatesCanCreate),
            "templates.create.*" => Some(Self::TemplatesCanCreate),
            "templates:edit:*" => Some(Self::TemplatesCanEdit),
            "templates.edit.*" => Some(Self::TemplatesCanEdit),
            "templates:delete:*" => Some(Self::TemplatesCanDelete),
            "templates.delete.*" => Some(Self::TemplatesCanDelete),
            "templates:apply:*" => Some(Self::TemplatesCanApply),
            "templates.apply.*" => Some(Self::TemplatesCanApply),
            "lists:view:*" => Some(Self::ListsCanView),
            "lists.view.*" => Some(Self::ListsCanView),
            "lists:create:*" => Some(Self::ListsCanCreate),
            "lists.create.*" => Some(Self::ListsCanCreate),
            "lists:edit:*" => Some(Self::ListsCanEdit),
            "lists.edit.*" => Some(Self::ListsCanEdit),
            "lists:delete:*" => Some(Self::ListsCanDelete),
            "lists.delete.*" => Some(Self::ListsCanDelete),
            "lists:export:*" => Some(Self::ListsCanExport),
            "lists.export.*" => Some(Self::ListsCanExport),
            "lists:import:*" => Some(Self::ListsCanImport),
            "lists.import.*" => Some(Self::ListsCanImport),
            "manage_users" => Some(Self::ManageUsers),
            "users.manage" => Some(Self::ManageUsers),
            "manage_bots" => Some(Self::ManageBots),
            "bots.manage" => Some(Self::ManageBots),
            "view_analytics" => Some(Self::ViewAnalytics),
            "analytics.view" => Some(Self::ViewAnalytics),
            "manage_settings" => Some(Self::ManageSettings),
            "settings.manage" => Some(Self::ManageSettings),
            "execute_tasks" => Some(Self::ExecuteTasks),
            "tasks.execute" => Some(Self::ExecuteTasks),
            "view_logs" => Some(Self::ViewLogs),
            "logs.view" => Some(Self::ViewLogs),
            "manage_secrets" => Some(Self::ManageSecrets),
            "secrets.manage" => Some(Self::ManageSecrets),
            "access_api" => Some(Self::AccessApi),
            "api.access" => Some(Self::AccessApi),
            "manage_files" => Some(Self::ManageFiles),
            "files.manage" => Some(Self::ManageFiles),
            "send_messages" => Some(Self::SendMessages),
            "messages.send" => Some(Self::SendMessages),
            "view_conversations" => Some(Self::ViewConversations),
            "conversations.view" => Some(Self::ViewConversations),
            "manage_webhooks" => Some(Self::ManageWebhooks),
            "webhooks.manage" => Some(Self::ManageWebhooks),
            "manage_integrations" => Some(Self::ManageIntegrations),
            "integrations.manage" => Some(Self::ManageIntegrations),
            "manageusers" => Some(Self::ManageUsers),
            "managebots" => Some(Self::ManageBots),
            "viewanalytics" => Some(Self::ViewAnalytics),
            "managesettings" => Some(Self::ManageSettings),
            "executetasks" => Some(Self::ExecuteTasks),
            "viewlogs" => Some(Self::ViewLogs),
            "managesecrets" => Some(Self::ManageSecrets),
            "accessapi" => Some(Self::AccessApi),
            "managefiles" => Some(Self::ManageFiles),
            "sendmessages" => Some(Self::SendMessages),
            "viewconversations" => Some(Self::ViewConversations),
            "managewebhooks" => Some(Self::ManageWebhooks),
            "manageintegrations" => Some(Self::ManageIntegrations),
            "administrationcanmanageorganization" => Some(Self::AdministrationCanManageOrganization),
            "administrationcanmanagemembers" => Some(Self::AdministrationCanManageMembers),
            "administrationcanviewmembers" => Some(Self::AdministrationCanViewMembers),
            "administrationcanmanagesettings" => Some(Self::AdministrationCanManageSettings),
            "administrationcanmanagebilling" => Some(Self::AdministrationCanManageBilling),
            "administrationcanviewbilling" => Some(Self::AdministrationCanViewBilling),
            "administrationcanmanageauditlog" => Some(Self::AdministrationCanManageAuditLog),
            "administrationcanviewauditlog" => Some(Self::AdministrationCanViewAuditLog),
            "administrationcanmanagedns" => Some(Self::AdministrationCanManageDns),
            "administrationcanmanageonboarding" => Some(Self::AdministrationCanManageOnboarding),
            "administrationcanmanageroles" => Some(Self::AdministrationCanManageRoles),
            "administrationcanmanagegroups" => Some(Self::AdministrationCanManageGroups),
            "compliancecanviewdashboard" => Some(Self::ComplianceCanViewDashboard),
            "compliancecanmanagepolicies" => Some(Self::ComplianceCanManagePolicies),
            "compliancecanviewreports" => Some(Self::ComplianceCanViewReports),
            "compliancecanexportreports" => Some(Self::ComplianceCanExportReports),
            "compliancecanmanagedataretention" => Some(Self::ComplianceCanManageDataRetention),
            "compliancecanmanagegdpr" => Some(Self::ComplianceCanManageGdpr),
            "compliancecanmanagehipaa" => Some(Self::ComplianceCanManageHipaa),
            "compliancecanmanageiso27001" => Some(Self::ComplianceCanManageIso27001),
            "compliancecanmanagesoc2" => Some(Self::ComplianceCanManageSoc2),
            "securitycanmanageusers" => Some(Self::SecurityCanManageUsers),
            "securitycanmanagesecrets" => Some(Self::SecurityCanManageSecrets),
            "securitycanviewlogs" => Some(Self::SecurityCanViewLogs),
            "securitycanmanageapikeys" => Some(Self::SecurityCanManageApiKeys),
            "securitycanmanageipsafelist" => Some(Self::SecurityCanManageIpSafelist),
            "securitycanmanagemfa" => Some(Self::SecurityCanManageMfa),
            "securitycanmanagesessions" => Some(Self::SecurityCanManageSessions),
            "securitycanmanageencryption" => Some(Self::SecurityCanManageEncryption),
            "securitycanmanageintegrations" => Some(Self::SecurityCanManageIntegrations),
            "securitycanmanagepasswordpolicy" => Some(Self::SecurityCanManagePasswordPolicy),
            "securitycanconfigure" => Some(Self::SecurityCanConfigure),
            "securitycanadmin" => Some(Self::SecurityCanAdmin),
            "mailcanread" => Some(Self::MailCanRead),
            "mailcansend" => Some(Self::MailCanSend),
            "mailcandelete" => Some(Self::MailCanDelete),
            "mailcanmanagefolders" => Some(Self::MailCanManageFolders),
            "mailcanmanagefilters" => Some(Self::MailCanManageFilters),
            "mailcanmanagetemplates" => Some(Self::MailCanManageTemplates),
            "mailcanmanagesignatures" => Some(Self::MailCanManageSignatures),
            "mailcanmanageautoreply" => Some(Self::MailCanManageAutoReply),
            "mailcanmanageforwarding" => Some(Self::MailCanManageForwarding),
            "mailcansendcampaigns" => Some(Self::MailCanSendCampaigns),
            "mailcanadmin" => Some(Self::MailCanAdmin),
            "mailcanconfigure" => Some(Self::MailCanConfigure),
            "calendarcanread" => Some(Self::CalendarCanRead),
            "calendarcancreate" => Some(Self::CalendarCanCreate),
            "calendarcanupdate" => Some(Self::CalendarCanUpdate),
            "calendarcandelete" => Some(Self::CalendarCanDelete),
            "calendarcanmanagecalendars" => Some(Self::CalendarCanManageCalendars),
            "calendarcanshare" => Some(Self::CalendarCanShare),
            "calendarcanmanagereminders" => Some(Self::CalendarCanManageReminders),
            "calendarcanviewavailability" => Some(Self::CalendarCanViewAvailability),
            "drivecanread" => Some(Self::DriveCanRead),
            "drivecanwrite" => Some(Self::DriveCanWrite),
            "drivecandelete" => Some(Self::DriveCanDelete),
            "drivecanupload" => Some(Self::DriveCanUpload),
            "drivecandownload" => Some(Self::DriveCanDownload),
            "drivecanmanagefolders" => Some(Self::DriveCanManageFolders),
            "drivecanshare" => Some(Self::DriveCanShare),
            "drivecanmanagepermissions" => Some(Self::DriveCanManagePermissions),
            "drivecanadmin" => Some(Self::DriveCanAdmin),
            "drivecanmanageversions" => Some(Self::DriveCanManageVersions),
            "documentscanread" => Some(Self::DocumentsCanRead),
            "documentscancreate" => Some(Self::DocumentsCanCreate),
            "documentscanupdate" => Some(Self::DocumentsCanUpdate),
            "documentscandelete" => Some(Self::DocumentsCanDelete),
            "documentscanshare" => Some(Self::DocumentsCanShare),
            "documentscanexport" => Some(Self::DocumentsCanExport),
            "documentscanmanagetemplates" => Some(Self::DocumentsCanManageTemplates),
            "documentscanmanagefolders" => Some(Self::DocumentsCanManageFolders),
            "documentscancomment" => Some(Self::DocumentsCanComment),
            "documentscantrackchanges" => Some(Self::DocumentsCanTrackChanges),
            "spreadsheetscanread" => Some(Self::SpreadsheetsCanRead),
            "spreadsheetscancreate" => Some(Self::SpreadsheetsCanCreate),
            "spreadsheetscanupdate" => Some(Self::SpreadsheetsCanUpdate),
            "spreadsheetscandelete" => Some(Self::SpreadsheetsCanDelete),
            "spreadsheetscanshare" => Some(Self::SpreadsheetsCanShare),
            "spreadsheetscanexport" => Some(Self::SpreadsheetsCanExport),
            "spreadsheetscanimport" => Some(Self::SpreadsheetsCanImport),
            "spreadsheetscanmanageformulas" => Some(Self::SpreadsheetsCanManageFormulas),
            "spreadsheetscanmanagecharts" => Some(Self::SpreadsheetsCanManageCharts),
            "presentationscanread" => Some(Self::PresentationsCanRead),
            "presentationscancreate" => Some(Self::PresentationsCanCreate),
            "presentationscanupdate" => Some(Self::PresentationsCanUpdate),
            "presentationscandelete" => Some(Self::PresentationsCanDelete),
            "presentationscanpresent" => Some(Self::PresentationsCanPresent),
            "presentationscanexport" => Some(Self::PresentationsCanExport),
            "meetingscancreate" => Some(Self::MeetingsCanCreate),
            "meetingscanjoin" => Some(Self::MeetingsCanJoin),
            "meetingscanmanagerooms" => Some(Self::MeetingsCanManageRooms),
            "meetingscanrecord" => Some(Self::MeetingsCanRecord),
            "meetingscansharescreen" => Some(Self::MeetingsCanShareScreen),
            "meetingscanmanageparticipants" => Some(Self::MeetingsCanManageParticipants),
            "meetingscanmanagesettings" => Some(Self::MeetingsCanManageSettings),
            "meetingscanadmin" => Some(Self::MeetingsCanAdmin),
            "chatcansendmessages" => Some(Self::ChatCanSendMessages),
            "chatcanreadmessages" => Some(Self::ChatCanReadMessages),
            "chatcandeletemessages" => Some(Self::ChatCanDeleteMessages),
            "chatcanviewconversations" => Some(Self::ChatCanViewConversations),
            "chatcanmanagebots" => Some(Self::ChatCanManageBots),
            "chatcancreatebots" => Some(Self::ChatCanCreateBots),
            "chatcaneditbots" => Some(Self::ChatCanEditBots),
            "chatcandeletebots" => Some(Self::ChatCanDeleteBots),
            "chatcanpublishbots" => Some(Self::ChatCanPublishBots),
            "chatcanviewbots" => Some(Self::ChatCanViewBots),
            "chatcanexecutetools" => Some(Self::ChatCanExecuteTools),
            "chatcanmanageknowledgebase" => Some(Self::ChatCanManageKnowledgeBase),
            "chatcanreadknowledgebase" => Some(Self::ChatCanReadKnowledgeBase),
            "chatcanwriteknowledgebase" => Some(Self::ChatCanWriteKnowledgeBase),
            "chatcanadminknowledgebase" => Some(Self::ChatCanAdminKnowledgeBase),
            "chatcanconfigurebots" => Some(Self::ChatCanConfigureBots),
            "taskscancreate" => Some(Self::TasksCanCreate),
            "taskscanread" => Some(Self::TasksCanRead),
            "taskscanupdate" => Some(Self::TasksCanUpdate),
            "taskscandelete" => Some(Self::TasksCanDelete),
            "taskscanexecute" => Some(Self::TasksCanExecute),
            "taskscanassign" => Some(Self::TasksCanAssign),
            "taskscanmanageprojects" => Some(Self::TasksCanManageProjects),
            "taskscanmanageworkflows" => Some(Self::TasksCanManageWorkflows),
            "taskscanmanageautotask" => Some(Self::TasksCanManageAutoTask),
            "aitoolscanmanagellm" => Some(Self::AiToolsCanManageLlm),
            "aitoolscanconfigurellm" => Some(Self::AiToolsCanConfigureLlm),
            "aitoolscanmanagemodels" => Some(Self::AiToolsCanManageModels),
            "aitoolscanmanageprompts" => Some(Self::AiToolsCanManagePrompts),
            "aitoolscandesignscripts" => Some(Self::AiToolsCanDesignScripts),
            "aitoolscaneditscripts" => Some(Self::AiToolsCanEditScripts),
            "aitoolscanmanageautotask" => Some(Self::AiToolsCanManageAutotask),
            "aitoolscanmanagetraining" => Some(Self::AiToolsCanManageTraining),
            "aitoolscanmanagevibe" => Some(Self::AiToolsCanManageVibe),
            "aitoolscanmanagemcp" => Some(Self::AiToolsCanManageMcp),
            "businessintelligencecanviewdashboard" => Some(Self::BusinessIntelligenceCanViewDashboard),
            "businessintelligencecanviewreports" => Some(Self::BusinessIntelligenceCanViewReports),
            "businessintelligencecancreatereports" => Some(Self::BusinessIntelligenceCanCreateReports),
            "businessintelligencecaneditreports" => Some(Self::BusinessIntelligenceCanEditReports),
            "businessintelligencecandeletereports" => Some(Self::BusinessIntelligenceCanDeleteReports),
            "businessintelligencecanexportreports" => Some(Self::BusinessIntelligenceCanExportReports),
            "businessintelligencecanviewmetrics" => Some(Self::BusinessIntelligenceCanViewMetrics),
            "businessintelligencecanmanagedashboards" => Some(Self::BusinessIntelligenceCanManageDashboards),
            "businessintelligencecantrace" => Some(Self::BusinessIntelligenceCanTrace),
            "businessintelligencecanmonitorperformance" => Some(Self::BusinessIntelligenceCanMonitorPerformance),
            "businessintelligencecanviewanalytics" => Some(Self::BusinessIntelligenceCanViewAnalytics),
            "businessintelligencecanexportanalytics" => Some(Self::BusinessIntelligenceCanExportAnalytics),
            "integrationscanmanagewebhooks" => Some(Self::IntegrationsCanManageWebhooks),
            "integrationscanmanageapikeys" => Some(Self::IntegrationsCanManageApiKeys),
            "integrationscanconnectsources" => Some(Self::IntegrationsCanConnectSources),
            "integrationscanmanagesocialmedia" => Some(Self::IntegrationsCanManageSocialMedia),
            "integrationscanmanagewhatsapp" => Some(Self::IntegrationsCanManageWhatsApp),
            "integrationscanmanagetelegram" => Some(Self::IntegrationsCanManageTelegram),
            "integrationscanmanagemsteams" => Some(Self::IntegrationsCanManageMsTeams),
            "integrationscanmanageinstagram" => Some(Self::IntegrationsCanManageInstagram),
            "integrationscanmanageimap" => Some(Self::IntegrationsCanManageImap),
            "integrationscanmanagegoogle" => Some(Self::IntegrationsCanManageGoogle),
            "integrationscanmanagemicrosoft" => Some(Self::IntegrationsCanManageMicrosoft),
            "integrationscanmanagechannels" => Some(Self::IntegrationsCanManageChannels),
            "automationcancreateworkflows" => Some(Self::AutomationCanCreateWorkflows),
            "automationcaneditworkflows" => Some(Self::AutomationCanEditWorkflows),
            "automationcandeleteworkflows" => Some(Self::AutomationCanDeleteWorkflows),
            "automationcanexecuteworkflows" => Some(Self::AutomationCanExecuteWorkflows),
            "automationcanmanagetriggers" => Some(Self::AutomationCanManageTriggers),
            "automationcanmanageschedules" => Some(Self::AutomationCanManageSchedules),
            "automationcanmanageeventhandlers" => Some(Self::AutomationCanManageEventHandlers),
            "crmcanviewpipeline" => Some(Self::CrmCanViewPipeline),
            "crmcanmanageleads" => Some(Self::CrmCanManageLeads),
            "crmcanmanagecontacts" => Some(Self::CrmCanManageContacts),
            "crmcanmanagedeals" => Some(Self::CrmCanManageDeals),
            "crmcanviewreports" => Some(Self::CrmCanViewReports),
            "crmcanexportreports" => Some(Self::CrmCanExportReports),
            "crmcanmanageforecast" => Some(Self::CrmCanManageForecast),
            "campaignscancreate" => Some(Self::CampaignsCanCreate),
            "campaignscanedit" => Some(Self::CampaignsCanEdit),
            "campaignscandelete" => Some(Self::CampaignsCanDelete),
            "campaignscanexecute" => Some(Self::CampaignsCanExecute),
            "campaignscanviewanalytics" => Some(Self::CampaignsCanViewAnalytics),
            "campaignscanmanagesegments" => Some(Self::CampaignsCanManageSegments),
            "productscanviewcatalog" => Some(Self::ProductsCanViewCatalog),
            "productscancreateproducts" => Some(Self::ProductsCanCreateProducts),
            "productscaneditproducts" => Some(Self::ProductsCanEditProducts),
            "productscandeleteproducts" => Some(Self::ProductsCanDeleteProducts),
            "productscancreateservices" => Some(Self::ProductsCanCreateServices),
            "productscaneditservices" => Some(Self::ProductsCanEditServices),
            "productscandeleteservices" => Some(Self::ProductsCanDeleteServices),
            "productscanmanagepricelists" => Some(Self::ProductsCanManagePriceLists),
            "ticketscancreate" => Some(Self::TicketsCanCreate),
            "ticketscanread" => Some(Self::TicketsCanRead),
            "ticketscanupdate" => Some(Self::TicketsCanUpdate),
            "ticketscandelete" => Some(Self::TicketsCanDelete),
            "ticketscanassign" => Some(Self::TicketsCanAssign),
            "ticketscanresolve" => Some(Self::TicketsCanResolve),
            "ticketscanmanagepriorities" => Some(Self::TicketsCanManagePriorities),
            "ticketscanviewanalytics" => Some(Self::TicketsCanViewAnalytics),
            "ticketscanmanageattendant" => Some(Self::TicketsCanManageAttendant),
            "peoplecanviewdirectory" => Some(Self::PeopleCanViewDirectory),
            "peoplecanmanagecontacts" => Some(Self::PeopleCanManageContacts),
            "peoplecanmanagegroups" => Some(Self::PeopleCanManageGroups),
            "peoplecanmanageroles" => Some(Self::PeopleCanManageRoles),
            "peoplecanimportcontacts" => Some(Self::PeopleCanImportContacts),
            "browsercannavigate" => Some(Self::BrowserCanNavigate),
            "browsercanbookmark" => Some(Self::BrowserCanBookmark),
            "browsercanmanagehistory" => Some(Self::BrowserCanManageHistory),
            "browsercandownload" => Some(Self::BrowserCanDownload),
            "terminalcanexecute" => Some(Self::TerminalCanExecute),
            "terminalcanviewoutput" => Some(Self::TerminalCanViewOutput),
            "terminalcanmanagesessions" => Some(Self::TerminalCanManageSessions),
            "researchcansearch" => Some(Self::ResearchCanSearch),
            "researchcanmanagesources" => Some(Self::ResearchCanManageSources),
            "researchcanexportresults" => Some(Self::ResearchCanExportResults),
            "researchcanmanagesessions" => Some(Self::ResearchCanManageSessions),
            "socialcanpost" => Some(Self::SocialCanPost),
            "socialcanscheduleposts" => Some(Self::SocialCanSchedulePosts),
            "socialcanviewfeed" => Some(Self::SocialCanViewFeed),
            "socialcanmanageaccounts" => Some(Self::SocialCanManageAccounts),
            "socialcanviewanalytics" => Some(Self::SocialCanViewAnalytics),
            "videocanupload" => Some(Self::VideoCanUpload),
            "videocanplay" => Some(Self::VideoCanPlay),
            "videocanedit" => Some(Self::VideoCanEdit),
            "videocandelete" => Some(Self::VideoCanDelete),
            "videocanmanagelibrary" => Some(Self::VideoCanManageLibrary),
            "canvascancreate" => Some(Self::CanvasCanCreate),
            "canvascanedit" => Some(Self::CanvasCanEdit),
            "canvascanview" => Some(Self::CanvasCanView),
            "canvascandelete" => Some(Self::CanvasCanDelete),
            "canvascanexport" => Some(Self::CanvasCanExport),
            "workspacecancreatesites" => Some(Self::WorkspaceCanCreateSites),
            "workspacecaneditsites" => Some(Self::WorkspaceCanEditSites),
            "workspacecandeletesites" => Some(Self::WorkspaceCanDeleteSites),
            "workspacecanviewsites" => Some(Self::WorkspaceCanViewSites),
            "workspacecanmanagepages" => Some(Self::WorkspaceCanManagePages),
            "workspacecanmanagedatabases" => Some(Self::WorkspaceCanManageDatabases),
            "goalscancreate" => Some(Self::GoalsCanCreate),
            "goalscanedit" => Some(Self::GoalsCanEdit),
            "goalscandelete" => Some(Self::GoalsCanDelete),
            "goalscanview" => Some(Self::GoalsCanView),
            "goalscantrackprogress" => Some(Self::GoalsCanTrackProgress),
            "learncanviewcourses" => Some(Self::LearnCanViewCourses),
            "learncanenroll" => Some(Self::LearnCanEnroll),
            "learncancreatecourses" => Some(Self::LearnCanCreateCourses),
            "learncaneditcourses" => Some(Self::LearnCanEditCourses),
            "learncandeletecourses" => Some(Self::LearnCanDeleteCourses),
            "learncanmanagemodules" => Some(Self::LearnCanManageModules),
            "codecanread" => Some(Self::CodeCanRead),
            "codecanwrite" => Some(Self::CodeCanWrite),
            "codecandelete" => Some(Self::CodeCanDelete),
            "codecanexecute" => Some(Self::CodeCanExecute),
            "codecanmanagegit" => Some(Self::CodeCanManageGit),
            "codecancommit" => Some(Self::CodeCanCommit),
            "codecanpush" => Some(Self::CodeCanPush),
            "codecandeploy" => Some(Self::CodeCanDeploy),
            "databasecanquery" => Some(Self::DatabaseCanQuery),
            "databasecanreadtables" => Some(Self::DatabaseCanReadTables),
            "databasecanwritetables" => Some(Self::DatabaseCanWriteTables),
            "databasecandeletetables" => Some(Self::DatabaseCanDeleteTables),
            "databasecanadmintables" => Some(Self::DatabaseCanAdminTables),
            "databasecanmanagemigrations" => Some(Self::DatabaseCanManageMigrations),
            "templatescanview" => Some(Self::TemplatesCanView),
            "templatescancreate" => Some(Self::TemplatesCanCreate),
            "templatescanedit" => Some(Self::TemplatesCanEdit),
            "templatescandelete" => Some(Self::TemplatesCanDelete),
            "templatescanapply" => Some(Self::TemplatesCanApply),
            "listscanview" => Some(Self::ListsCanView),
            "listscancreate" => Some(Self::ListsCanCreate),
            "listscanedit" => Some(Self::ListsCanEdit),
            "listscandelete" => Some(Self::ListsCanDelete),
            "listscanexport" => Some(Self::ListsCanExport),
            "listscanimport" => Some(Self::ListsCanImport),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    #[default]
    Anonymous,
    User,
    Moderator,
    Admin,
    SuperAdmin,
    Service,
    Bot,
    BotOwner,
    BotOperator,
    BotViewer,
}

impl Role {
    pub fn permissions(&self) -> HashSet<Permission> {
        match self {
            Self::Anonymous => HashSet::new(),
            Self::User => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::AccessApi);
                perms
            }
            Self::Moderator => {
                let mut perms = Self::User.permissions();
                perms.insert(Permission::Write);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewConversations);
                perms
            }
            Self::Admin => {
                let mut perms = Self::Moderator.permissions();
                perms.insert(Permission::Delete);
                perms.insert(Permission::ManageUsers);
                perms.insert(Permission::ManageBots);
                perms.insert(Permission::ManageSettings);
                perms.insert(Permission::ExecuteTasks);
                perms.insert(Permission::ManageFiles);
                perms.insert(Permission::ManageWebhooks);
                perms
            }
            Self::SuperAdmin => {
                let mut perms = Self::Admin.permissions();
                perms.insert(Permission::Admin);
                perms.insert(Permission::ManageSecrets);
                perms.insert(Permission::ManageIntegrations);
                perms
            }
            Self::Service => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ExecuteTasks);
                perms.insert(Permission::SendMessages);
                perms
            }
            Self::Bot => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::SendMessages);
                perms
            }
            Self::BotOwner => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::Delete);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ManageBots);
                perms.insert(Permission::ManageSettings);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::ManageFiles);
                perms.insert(Permission::SendMessages);
                perms.insert(Permission::ViewConversations);
                perms.insert(Permission::ManageWebhooks);
                perms
            }
            Self::BotOperator => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewLogs);
                perms.insert(Permission::SendMessages);
                perms.insert(Permission::ViewConversations);
                perms
            }
            Self::BotViewer => {
                let mut perms = HashSet::new();
                perms.insert(Permission::Read);
                perms.insert(Permission::AccessApi);
                perms.insert(Permission::ViewAnalytics);
                perms.insert(Permission::ViewConversations);
                perms
            }
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }
}

impl std::str::FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anonymous" => Ok(Self::Anonymous),
            "user" => Ok(Self::User),
            "moderator" | "mod" => Ok(Self::Moderator),
            "admin" => Ok(Self::Admin),
            "superadmin" | "super_admin" | "super" => Ok(Self::SuperAdmin),
            "service" | "svc" => Ok(Self::Service),
            "bot" => Ok(Self::Bot),
            "bot_owner" | "botowner" | "owner" => Ok(Self::BotOwner),
            "bot_operator" | "botoperator" | "operator" => Ok(Self::BotOperator),
            "bot_viewer" | "botviewer" | "viewer" => Ok(Self::BotViewer),
            _ => Ok(Self::Anonymous),
        }
    }
}

impl Role {
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            Self::Anonymous => 0,
            Self::User => 1,
            Self::BotViewer => 2,
            Self::BotOperator => 3,
            Self::BotOwner => 4,
            Self::Bot => 4,
            Self::Moderator => 5,
            Self::Service => 6,
            Self::Admin => 7,
            Self::SuperAdmin => 8,
        }
    }

    pub fn is_at_least(&self, other: &Role) -> bool {
        self.hierarchy_level() >= other.hierarchy_level()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotAccess {
    pub bot_id: Uuid,
    pub role: Role,
    pub granted_at: Option<i64>,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<i64>,
}

impl BotAccess {
    pub fn new(bot_id: Uuid, role: Role) -> Self {
        Self {
            bot_id,
            role,
            granted_at: Some(chrono::Utc::now().timestamp()),
            granted_by: None,
            expires_at: None,
        }
    }

    pub fn owner(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotOwner)
    }

    pub fn operator(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotOperator)
    }

    pub fn viewer(bot_id: Uuid) -> Self {
        Self::new(bot_id, Role::BotViewer)
    }

    pub fn with_expiry(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_grantor(mut self, granted_by: Uuid) -> Self {
        self.granted_by = Some(granted_by);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now().timestamp() > expires
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<Role>,
    pub bot_access: HashMap<Uuid, BotAccess>,
    pub current_bot_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub organization_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

/// Marker extension inserted by auth middleware when a path is allowed as public/anonymous.
/// RBAC middleware checks for this marker and skips route permission checks when present.
#[derive(Debug, Clone)]
pub struct PublicPathAllowed;

impl Default for AuthenticatedUser {
    fn default() -> Self {
        Self::anonymous()
    }
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid, username: String) -> Self {
        Self {
            user_id,
            username,
            email: None,
            roles: vec![Role::User],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            username: "anonymous".to_string(),
            email: None,
            roles: vec![Role::Anonymous],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn service(name: &str) -> Self {
        Self {
            user_id: Uuid::nil(),
            username: format!("service:{}", name),
            email: None,
            roles: vec![Role::Service],
            bot_access: HashMap::new(),
            current_bot_id: None,
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn bot_user(bot_id: Uuid, bot_name: &str) -> Self {
        Self {
            user_id: bot_id,
            username: format!("bot:{}", bot_name),
            email: None,
            roles: vec![Role::Bot],
            bot_access: HashMap::new(),
            current_bot_id: Some(bot_id),
            session_id: None,
            organization_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_role(mut self, role: Role) -> Self {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
        self
    }

    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_bot_access(mut self, access: BotAccess) -> Self {
        self.bot_access.insert(access.bot_id, access);
        self
    }

    pub fn with_current_bot(mut self, bot_id: Uuid) -> Self {
        self.current_bot_id = Some(bot_id);
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.roles.iter().any(|r| r.has_permission(permission))
    }

    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|r| self.roles.contains(r))
    }

    pub fn highest_role(&self) -> &Role {
        self.roles
            .iter()
            .max_by_key(|r| r.hierarchy_level())
            .unwrap_or(&Role::Anonymous)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin) || self.has_role(&Role::SuperAdmin)
    }

    pub fn is_super_admin(&self) -> bool {
        self.has_role(&Role::SuperAdmin)
    }

    pub fn is_authenticated(&self) -> bool {
        !self.has_role(&Role::Anonymous) && self.user_id != Uuid::nil()
    }

    pub fn is_service(&self) -> bool {
        self.has_role(&Role::Service)
    }

    pub fn is_bot(&self) -> bool {
        self.has_role(&Role::Bot)
    }

    pub fn get_bot_access(&self, bot_id: &Uuid) -> Option<&BotAccess> {
        self.bot_access.get(bot_id).filter(|a| a.is_valid())
    }

    pub fn get_bot_role(&self, bot_id: &Uuid) -> Option<&Role> {
        self.get_bot_access(bot_id).map(|a| &a.role)
    }

    pub fn has_bot_permission(&self, bot_id: &Uuid, permission: &Permission) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.has_permission(permission)
        } else {
            false
        }
    }

    pub fn can_access_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() || self.is_service() {
            return true;
        }

        if self.current_bot_id.as_ref() == Some(bot_id) && self.is_bot() {
            return true;
        }

        self.get_bot_access(bot_id).is_some()
    }

    pub fn can_manage_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role == Role::BotOwner
        } else {
            false
        }
    }

    pub fn can_operate_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.is_at_least(&Role::BotOperator)
        } else {
            false
        }
    }

    pub fn can_view_bot(&self, bot_id: &Uuid) -> bool {
        if self.is_admin() || self.is_service() {
            return true;
        }

        if let Some(access) = self.get_bot_access(bot_id) {
            access.role.is_at_least(&Role::BotViewer)
        } else {
            false
        }
    }

    pub fn can_access_organization(&self, org_id: &Uuid) -> bool {
        if self.is_admin() {
            return true;
        }
        self.organization_id
            .as_ref()
            .map(|id| id == org_id)
            .unwrap_or(false)
    }

    pub fn accessible_bot_ids(&self) -> Vec<Uuid> {
        self.bot_access
            .iter()
            .filter(|(_, access)| access.is_valid())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn owned_bot_ids(&self) -> Vec<Uuid> {
        self.bot_access
            .iter()
            .filter(|(_, access)| access.is_valid() && access.role == Role::BotOwner)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({"error": "Authentication required"})),
            ))
    }
}
