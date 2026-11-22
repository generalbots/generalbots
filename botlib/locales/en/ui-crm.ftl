# =============================================================================
# General Bots — en — CRM
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# CRM
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = Pipeline
crm-leads = Leads
crm-opportunities = Opportunities
crm-accounts = Accounts
crm-contacts = Contacts
crm-activities = Activities

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = Lead
crm-lead-desc = Unqualified prospect
crm-opportunity = Opportunity
crm-opportunity-desc = Qualified sales opportunity
crm-account = Account
crm-account-desc = Company or organization
crm-contact = Contact
crm-contact-desc = Person at an account
crm-activity = Activity
crm-activity-desc = Task, call, or email

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = Qualify
crm-convert = Convert
crm-won = Won
crm-lost = Lost
crm-new-lead = New Lead
crm-new-opportunity = New Opportunity
crm-new-account = New Account
crm-new-contact = New Contact

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = Stage
crm-value = Value
crm-probability = Probability
crm-close-date = Close Date
crm-company = Company
crm-phone = Phone
crm-email = Email
crm-source = Source
crm-owner = Owner

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = New
crm-pipeline-contacted = Contacted
crm-pipeline-qualified = Qualified
crm-pipeline-proposal = Proposal
crm-pipeline-negotiation = Negotiation
crm-pipeline-closed-won = Closed Won
crm-pipeline-closed-lost = Closed Lost

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = Manage leads, opportunities, and customers
crm-stage-lead = Lead
crm-stage-qualified = Qualified
crm-stage-proposal = Proposal
crm-stage-negotiation = Negotiation
crm-stage-won = Won
crm-stage-lost = Lost
crm-conversion-rate = Conversion Rate
crm-pipeline-value = Pipeline Value
crm-avg-deal = Avg Deal Size
crm-won-month = Won This Month

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = No leads found
crm-no-opportunities = No opportunities found
crm-no-accounts = No accounts found
crm-no-contacts = No contacts found
crm-drag-hint = Drag cards to change stage

# CRM / Mail / Campaigns integration keys
crm-compose-email = Compose Email
crm-send-email = Send Email
mail-snooze = Snooze
mail-snooze-later-today = Later today (6:00 PM)
mail-snooze-tomorrow = Tomorrow (8:00 AM)
mail-snooze-next-week = Next week (Mon 8:00 AM)
mail-crm-log = Log to CRM
mail-crm-create-lead = Create Lead
mail-add-to-list = Add to List
campaign-send-email = Send Email

# -----------------------------------------------------------------------------
# CRM
# -----------------------------------------------------------------------------
crm-add-lead = Add Lead
crm-audit-title = Audit trail
crm-campaigns = Campaigns
crm-col-account = Account
crm-col-actions = Actions
crm-col-city = City
crm-col-close = Expected Close
crm-col-close-date = Expected Close
crm-col-contact = Contact
crm-col-contacts = Deals
crm-col-customer = Customer
crm-col-delivery = Delivery
crm-col-email = Email
crm-col-industry = Industry
crm-col-items = Items
crm-col-name = Name
crm-col-number = Number
crm-col-owner = Owner
crm-col-phone = Phone
crm-col-probability = Probability
crm-col-revenue = Annual Revenue
crm-col-source = Source
crm-col-stage = Stage
crm-col-status = Status
crm-col-title = Title
crm-col-total = Total
crm-col-valid-until = Valid until
crm-col-value = Value
crm-create-lead = Create Contact
crm-currency = Currency
crm-deals = Deals
crm-estimated-value = Estimated Value
crm-filter-all = All
crm-filter-lost = Lost
crm-filter-negotiation = Negotiation
crm-filter-new = New
crm-filter-proposal = Proposal
crm-filter-qualified = Qualified
crm-filter-won = Won
crm-first-name = First Name
crm-job-title = Job Title
crm-last-name = Last Name
crm-loading = Loading…
crm-new = New
crm-new-deal = New Deal
crm-notes = Notes
crm-orders = Orders
crm-quotes = Quotes & Orders
crm-search-placeholder-deals = Search deals, accounts...
crm-source-cold-call = Cold Call
crm-source-email = Email Campaign
crm-source-event = Event/Conference
crm-source-linkedin = LinkedIn
crm-source-other = Other
crm-source-partner = Partner
crm-source-referral = Referral
crm-source-website = Website
crm-this-month = Won This Month
crm-total-value = Total Pipeline Value

# semantic i18n key (#2542)
crm-filter-opportunities-placeholder = Filter opportunities…
crm-filter-deals-placeholder = Filter deals…
crm-filter-accounts-placeholder = Filter accounts…
crm-filter-contacts-placeholder = Filter contacts…
crm-filter-campaigns-placeholder = Filter campaigns…

# semantic i18n key (#2542)
crm-placeholder-email = john@company.com
crm-placeholder-deal-lost-reason = When the deal is lost
crm-placeholder-first-name = John
crm-placeholder-last-name = Doe
crm-placeholder-phone = +1 (555) 123-4567
crm-placeholder-company-name = Company Name
crm-placeholder-job-title = CEO, Manager, etc.
crm-placeholder-amount = 0.00
crm-placeholder-lead-lost-reason = Why this lead went cold
crm-placeholder-lead-notes = Additional information about this lead...
crm-placeholder-website = https://

# semantic i18n key (#2542)
crm-placeholder-search-contacts = Search contacts...

# Chat header / app chrome (#2542)
crm-lost-reason-title = Why was the deal lost?

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
crm-opps-empty-title = No opportunities yet
crm-opps-empty-desc = An opportunity tracks a possible sale from first contact to close.
crm-opps-empty-action = Create an opportunity
crm-deals-empty-title = No deals yet
crm-deals-empty-desc = Deals in every stage appear here, with the amount and the owner.
crm-deals-empty-action = Add a deal
crm-accounts-empty-title = No accounts yet
crm-accounts-empty-desc = An account is a company you sell to. Contacts and deals are grouped under it.
crm-accounts-empty-action = Add an account
crm-contacts-empty-title = No contacts yet
crm-contacts-empty-desc = People you talk to live here: name, e-mail, phone and the account they belong to.
crm-contacts-empty-action = Add a contact
crm-campaigns-empty-title = No campaigns yet
crm-campaigns-empty-desc = A campaign groups the outreach that brings leads into the pipeline.
crm-campaigns-empty-action = Create a campaign

# -----------------------------------------------------------------------------
# #2328 — audit drawer + campaign metrics state copy
# -----------------------------------------------------------------------------
crm-audit-empty = No audit entries yet
crm-audit-unavailable = Audit unavailable ({ $status })
crm-campaign-metrics-sent = Sent { $sent } · delivered { $delivered } · opened { $opened } · clicked { $clicked }
crm-revenue-detail = Replies: { $reply } · Unsubscribes: { $unsubscribe } · Bounces: { $bounce } · Complaints: { $complaint } · Skips: { $skip }
crm-revenue-influenced = Influenced deals: { $deals } · won: { $won }
crm-revenue-unattributed = Unattributed touches: { $count }
crm-revenue-unavailable = Revenue attribution is unavailable.
crm-revenue-note-unattributed = Some touches could not be attributed to a campaign.
crm-revenue-note-unknown = Some events use an unrecognised type.
crm-revenue-note-incomplete-money = Revenue is omitted because deal values are incomplete or mixed.
crm-revenue-note-no-cohort = Conversion rate is unavailable without a deal cohort.
crm-revenue-note-roi = ROI is not reported because campaign budget is not reconciled.
