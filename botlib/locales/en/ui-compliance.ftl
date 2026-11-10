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
