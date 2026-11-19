# General Bots - Drive, lists, people, player, settings and workspace labels (#2087)
# Keys asked for by those suite apps that the catalog did not define yet.

drive-new-bot = New Bot
drive-upload-folder = Upload folder
drive-take-photo = Take photo
drive-camera-permission-title = Use camera
drive-camera-permission-explainer = General Bots will ask for camera access so you can take a photo and upload it to this folder. Your browser controls the permission.
lists-title = Marketing Lists
cancel = Cancel
save = Save
player-previous = Previous
player-play = Play
player-next = Next
player-volume = Volume
settings-shared-links = Shared Links
workspace-members = Members

# ---- #2303: app logic i18n (tax/fraud/vision/minutes/brazil/timeclock/plan/docs) ----
tax-empty-nfe = No NF-e found
tax-empty-nfse = No NFS-e found
tax-empty-cte = No CT-e found
tax-nfe-issued = NF-e issued successfully
tax-nfse-issued = NFS-e issued successfully
tax-cte-issued = CT-e issued successfully
tax-error-emit-nfe = Error issuing NF-e
tax-error-emit-nfse = Error issuing NFS-e
tax-error-emit-cte = Error issuing CT-e
tax-sped-contributions = Contributions
tax-sped-fiscal = Fiscal
tax-sped-generated = SPED {name} generated successfully
tax-error-generate-sped = Error generating SPED
tax-error-download-sped = Error downloading SPED
fraud-status-approved = Approved
fraud-status-review = Review
fraud-status-blocked = Blocked
fraud-empty-transactions = No transactions found
fraud-empty-rules = No rules configured
fraud-error-update-rule = Error updating rule
fraud-rule-deleted = Rule deleted
fraud-error-delete-rule = Error deleting rule
fraud-empty-blocklist = Blocklist is empty
fraud-item-removed = Item removed from list
fraud-error-remove-item = Error removing item
fraud-empty-reports = No reports generated
fraud-error-download-report = Error downloading report
fraud-name-condition-required = Name and condition are required
fraud-rule-saved = Rule saved successfully
fraud-error-save-rule = Error saving rule
fraud-value-required = Value is required
fraud-item-added-blocklist = Item added to blocklist
fraud-error-add-item = Error adding item
fraud-report-generated = Report generated successfully
fraud-error-generate-report = Error generating report
fraud-new-rule = New Rule
minutes-loading-upcoming = Loading upcoming meetings…
brazil-loading-overview = Loading overview…
vision-invalid-file-type = Invalid file type
vision-file-too-large = File too large (max. 10MB)
timeclock-open-entry-exists = There is already an open clock-in
timeclock-no-open-entry = No open clock-in
timeclock-break-open = There is already an open break
timeclock-no-open-break = No open break
timeclock-break-ended = Break already ended
plan-confirm-delete-task = Delete task?
plan-import-success = Plan imported with {count} task(s).
plan-invalid-file = Invalid plan file.
docs-spellcheck-title = Spell check
docs-spellcheck-checking = Checking…
docs-spellcheck-error = Error checking spelling
docs-spellcheck-none = No corrections found.
docs-spellcheck-all-applied = All corrections applied.
docs-spellcheck-accept = Accept
docs-spellcheck-not-found = Text not found in document.

# semantic i18n key (#2542)
tax-errors-label = Errors

# semantic i18n key (#2542)
tax-cte-documents-title = Transport Documents (CT-e)

# semantic i18n key (#2542)
tax-cfop-label = CFOP

# semantic i18n key (#2542)
tax-products-label = Products

# semantic i18n key (#2542)
tax-sender-label = Sender

# semantic i18n key (#2542)
tax-origin-city-label = Origin City

# semantic i18n key (#2542)
tax-destination-city-label = Destination City

# semantic i18n key (#2542)
tax-weight-kg-label = Weight (kg)

# semantic i18n key (#2542)
tax-mode-label = Mode

# semantic i18n key (#2542)
minutes-title = Meeting Minutes

# semantic i18n key (#2542)
minutes-action-signature = Signature

# semantic i18n key (#2542)
minutes-tab-live = In progress

# semantic i18n key (#2542)
minutes-tab-documents = Minutes

# semantic i18n key (#2542)
minutes-tab-signatures = Signatures

# semantic i18n key (#2542)
minutes-label-date = Date

# semantic i18n key (#2542)
minutes-label-location = Location

# semantic i18n key (#2542)
minutes-label-deadline = Deadline

# semantic i18n key (#2542)
minutes-label-participant = Participant

# semantic i18n key (#2542)
minutes-status-confirmed = Confirmed

# semantic i18n key (#2542)
minutes-status-declined = Declined

# semantic i18n key (#2542)
minutes-status-tentative = Maybe

# semantic i18n key (#2542)
minutes-action-register = Register

# semantic i18n key (#2542)
minutes-editor-title = Minutes editor

# semantic i18n key (#2542)
minutes-option-summary = Summary

# semantic i18n key (#2542)
minutes-digital-signature-title = Digital signature

# semantic i18n key (#2542)
minutes-label-document-id = Document ID

# semantic i18n key (#2542)
minutes-label-signer = Signer

# semantic i18n key (#2542)
minutes-action-sign = Sign

# semantic i18n key (#2542)
lists-search-placeholder = Search lists...
lists-name-placeholder = e.g., Active Customers
lists-query-placeholder = e.g., status = 'active' AND country = 'US'
lists-add-member-placeholder = Search contacts to add...

# semantic i18n key (#2542)
tax-placeholder-razao-cnpj = Company name or CNPJ
tax-placeholder-cfop-example = e.g. 5102
tax-placeholder-description = Description
tax-placeholder-value-brl = Amount (R$)
tax-placeholder-zero = 0
tax-placeholder-zero-decimal = 0.00
tax-placeholder-service-description = Description of service provided
tax-placeholder-service-code-example = e.g. 17.02
tax-placeholder-city-uf = City - State

# semantic i18n key (#2542)
minutes-placeholder-location = Room or link
minutes-placeholder-meeting-id = Meeting UUID
minutes-placeholder-participant = Name or email
minutes-placeholder-document-id = Document UUID
# semantic i18n key (#2542) — fraud app UI
fraud-add-rule = + New Rule
fraud-blocked-today = Blocked Today
fraud-review-queue = Review Queue
fraud-transactions-analyzed = Transactions Analyzed
fraud-tab-transactions = Transactions
fraud-tab-reports = Reports
fraud-transactions-heading = Transactions
fraud-filter-all-statuses = All statuses
fraud-rules-heading = Detection Rules
fraud-blocklist-heading = Blocklist
fraud-reports-heading = Reports
fraud-generate-report = Generate Report
fraud-rule-name-label = Rule Name
fraud-condition-label = Condition
fraud-action-label = Action
fraud-action-review = Review
fraud-priority-low = Low
fraud-priority-medium = Medium
fraud-priority-high = High
fraud-blocklist-add-title = Add to Blocklist
fraud-blocklist-type-phone = Phone
fraud-value-label = Value
fraud-add-to-blocklist = Add
fraud-rule-name-placeholder = e.g., High suspicious amount
fraud-condition-placeholder = e.g., amount > 10000 AND country != 'BR'
fraud-blocklist-value-placeholder = e.g., user@email.com
fraud-blocklist-reason-placeholder = e.g., Confirmed fraud
fraud-col-id = ID
fraud-col-value = Value
fraud-col-risk = Risk
fraud-col-status = Status
fraud-col-date = Date
fraud-col-name = Name
fraud-col-condition = Condition
fraud-col-action = Action
fraud-col-priority = Priority
fraud-col-active = Active
fraud-col-actions = Actions
fraud-col-type = Type
fraud-remove = Remove
fraud-col-period = Period
fraud-col-transactions = Transactions
fraud-col-blocked = Blocked
fraud-col-review = Review
fraud-col-false-positive = False Positive

# Chat header / app chrome (#2542)
docs-bibliography-title = Bibliography
docs-citations-empty = No citations
docs-footnotes-empty = No footnotes
docs-page-x-of-y = Page X of Y
docs-toc-empty = No headings found.
tax-emit = Issue

drive-upload-queue-body = {count} upload(s) did not finish. Retry?
drive-upload-queue-retry = Retry
drive-upload-queue-discard = Discard
drive-upload-queue-recovered = Uploaded {count} recovered file(s)

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
lists-state-empty-title = No lists yet
lists-state-empty-desc = A list groups contacts or companies so you can target them together.
lists-state-empty-action = Create your first list
minutes-state-empty-title = No meetings scheduled
minutes-state-empty-desc = Schedule a meeting and the transcript, summary and actions land here.
