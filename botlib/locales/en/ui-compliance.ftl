# =============================================================================
# General Bots — en — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = API Compliance Report
compliance-subtitle = Security scan for all bots - Check for passwords, fragile code, and misconfigurations
compliance-export-report = Export Report
compliance-run-scan = Run Compliance Scan
compliance-scanning = Scanning...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = All Bots
compliance-select-bots = Select Bots

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Critical
compliance-critical-desc = Requires immediate action
compliance-high = High
compliance-high-desc = Security risk
compliance-medium = Medium
compliance-medium-desc = Should be addressed
compliance-low = Low
compliance-low-desc = Best practice
compliance-info = Info
compliance-info-desc = Informational

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Severity
compliance-filter-type = Type
compliance-filter-all-severities = All Severities
compliance-filter-all-types = All Types
compliance-search-issues = Search issues...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Password in Config
compliance-type-hardcoded = Hardcoded Secrets
compliance-type-deprecated = Deprecated Keywords
compliance-type-fragile = Fragile Code
compliance-type-config = Configuration Issues

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Results
compliance-results-count = { $count ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Severity
compliance-col-issue = Issue
compliance-col-location = Location
compliance-col-details = Details
compliance-col-action = Action
compliance-view-details = View Details
compliance-fix-issue = Fix Issue
compliance-ignore = Ignore
compliance-no-issues = No issues found
compliance-no-issues-desc = Great! Your bots are compliant.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scan in progress...
compliance-scan-checking = Checking { $item }...
compliance-scan-complete = Scan complete
compliance-scan-failed = Scan failed

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Frameworks
compliance-new-framework = New
compliance-framework-name = Name
compliance-framework-version = Version
compliance-framework-description = Description
compliance-create-framework = Create Framework
compliance-controls = Controls
compliance-add-control = Add Control
compliance-control-id = Control ID
compliance-control-title = Title
compliance-control-category = Category
compliance-control-description = Description
compliance-mandatory = Mandatory
compliance-optional = Optional
compliance-evidence = Evidence
compliance-attach-evidence = Attach Evidence
compliance-evidence-path = File path (drive artifact)
compliance-evidence-type = Type
compliance-approve = Approve
compliance-covered = Covered
compliance-no-evidence = No evidence
compliance-export-csv = Export CSV
compliance-archive = Archive
compliance-total-controls = Total Controls
compliance-coverage = Coverage
compliance-no-frameworks = No frameworks configured yet.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Compliance Dashboard

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Type II compliance status, controls monitoring, and audit evidence

# semantic i18n key (#2542)
admin-compliance-compliant = Compliant

# semantic i18n key (#2542)
admin-compliance-overall-score = Overall Compliance Score

# semantic i18n key (#2542)
admin-compliance-controls-passing = Controls Passing

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Evidence Items Due

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Until Next Audit

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Scheduled: March 7, 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Trust Service Criteria

# semantic i18n key (#2542)
admin-compliance-availability = Availability

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Processing Integrity

# semantic i18n key (#2542)
admin-compliance-confidentiality = Confidentiality

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Controls Requiring Attention

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Multi-factor authentication for privileged accounts

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Due: Jan 25, 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Remediate

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Security incident response plan testing

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Annual test overdue by 15 days

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Data processing agreement documentation

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Due: Feb 1, 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Data classification policy review

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Policy review pending approval

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Due: Feb 15, 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Privacy notice updates

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = Privacy policy needs update for new data collection

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Due: Feb 28, 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Evidence Collection

# semantic i18n key (#2542)
admin-compliance-current = Current

# semantic i18n key (#2542)
admin-compliance-due-soon = Due Soon

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Penetration Test Report Q4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Vulnerability Assessment

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Access Review Report - January 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Access Control

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Due in 3 days

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Backup Restoration Test Results

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Backup & Recovery

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Due in 5 days

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Security Awareness Training Records

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Security Training

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = View All Evidence

# semantic i18n key (#2542)
admin-compliance-access-events = Access Events

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Configuration Changes

# semantic i18n key (#2542)
admin-compliance-security-events = Security Events

# semantic i18n key (#2542)
admin-compliance-compliance-events = Compliance Events

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Failed login attempt

# semantic i18n key (#2542)
admin-compliance-failed-login-details = User: admin@company.com | IP: 192.168.1.105 | Reason: Invalid password

# semantic i18n key (#2542)
admin-compliance-user-logged-in = User logged in

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = User: sarah.johnson@company.com | IP: 10.0.0.45
