# =============================================================================
# General Bots — fr — CRM
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
crm-title = GRC
crm-pipeline = Pipeline
crm-leads = Pistes
crm-opportunities = Opportunités
crm-accounts = Comptes
crm-contacts = Contacts
crm-activities = Activités

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = Diriger
crm-lead-desc = Prospect non qualifié
crm-opportunity = Opportunité
crm-opportunity-desc = Opportunité de vente qualifiée
crm-account = Compte
crm-account-desc = Entreprise ou organisation
crm-contact = Contacter
crm-contact-desc = Personne sur un compte
crm-activity = Activité
crm-activity-desc = Tâche, appel ou e-mail

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = Qualifier
crm-convert = Convertir
crm-won = Gagné
crm-lost = Perdu
crm-new-lead = Nouveau responsable
crm-new-opportunity = Nouvelle opportunité
crm-new-account = Nouveau compte
crm-new-contact = Nouveau contact

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = Scène
crm-value = Valeur
crm-probability = Probabilité
crm-close-date = Date de clôture
crm-company = Entreprise
crm-phone = Téléphone
crm-email = Courriel
crm-source = Source
crm-owner = Propriétaire

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = Nouveau
crm-pipeline-contacted = Contacté
crm-pipeline-qualified = Qualifié
crm-pipeline-proposal = Proposition
crm-pipeline-negotiation = Négociation
crm-pipeline-closed-won = Fermé Gagné
crm-pipeline-closed-lost = Fermé Perdu

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = Gérer les leads, les opportunités et les clients
crm-stage-lead = Diriger
crm-stage-qualified = Qualifié
crm-stage-proposal = Proposition
crm-stage-negotiation = Négociation
crm-stage-won = Gagné
crm-stage-lost = Perdu
crm-conversion-rate = Taux de conversion
crm-pipeline-value = Valeur du pipeline
crm-avg-deal = Taille moyenne de la transaction
crm-won-month = Gagné ce mois-ci

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = Aucune piste trouvée
crm-no-opportunities = Aucune opportunité trouvée
crm-no-accounts = Aucun compte trouvé
crm-no-contacts = Aucun contact trouvé
crm-drag-hint = Faites glisser les cartes pour changer d'étape

# CRM / Mail / Campaigns integration keys
crm-compose-email = Composer un e-mail
crm-send-email = Envoyer un e-mail
mail-snooze = Répéter
mail-snooze-later-today = Plus tard dans la journée (18h00)
mail-snooze-tomorrow = Demain (8h00)
mail-snooze-next-week = La semaine prochaine (lundi 8h00)
mail-crm-log = Connectez-vous au CRM
mail-crm-create-lead = Créer un prospect
mail-add-to-list = Ajouter à la liste
campaign-send-email = Envoyer un e-mail
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
crm-filter-opportunities-placeholder = Filtrer les opportunités…
crm-filter-deals-placeholder = Filtrer les affaires…
crm-filter-accounts-placeholder = Filtrer les comptes…
crm-filter-contacts-placeholder = Filtrer les contacts…
crm-filter-campaigns-placeholder = Filtrer les campagnes…

# semantic i18n key (#2542)
crm-placeholder-email = jean@entreprise.com
crm-placeholder-deal-lost-reason = Quand l'affaire a été perdue
crm-placeholder-first-name = Jean
crm-placeholder-last-name = Dupont
crm-placeholder-phone = +33 6 12 34 56 78
crm-placeholder-company-name = Nom de l'entreprise
crm-placeholder-job-title = PDG, Responsable, etc.
crm-placeholder-amount = 0,00
crm-placeholder-lead-lost-reason = Pourquoi ce prospect s'est refroidi
crm-placeholder-lead-notes = Informations supplémentaires sur ce prospect...
crm-placeholder-website = https://

# semantic i18n key (#2542)
crm-placeholder-search-contacts = Rechercher des contacts...

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
crm-opps-empty-title = Aucune opportunité pour le moment
crm-opps-empty-desc = Une opportunité suit une vente possible du premier contact à la conclusion.
crm-opps-empty-action = Créer une opportunité
crm-deals-empty-title = Aucune affaire pour le moment
crm-deals-empty-desc = Les affaires de chaque étape apparaissent ici, avec le montant et le responsable.
crm-deals-empty-action = Ajouter une affaire
crm-accounts-empty-title = Aucun compte pour le moment
crm-accounts-empty-desc = Un compte est une entreprise à laquelle vous vendez. Contacts et affaires y sont rattachés.
crm-accounts-empty-action = Ajouter un compte
crm-contacts-empty-title = Aucun contact pour le moment
crm-contacts-empty-desc = Les personnes avec qui vous échangez sont ici : nom, e-mail, téléphone et compte de rattachement.
crm-contacts-empty-action = Ajouter un contact
crm-campaigns-empty-title = Aucune campagne pour le moment
crm-campaigns-empty-desc = Une campagne regroupe les actions qui amènent des prospects dans le pipeline.
crm-campaigns-empty-action = Créer une campagne

# -----------------------------------------------------------------------------
# #2328 — audit drawer + campaign metrics state copy
# -----------------------------------------------------------------------------
crm-audit-empty = Aucune entrée d'audit pour le moment
crm-audit-unavailable = Audit indisponible ({ $status })
crm-campaign-metrics-sent = Envoyés { $sent } · distribués { $delivered } · ouverts { $opened } · cliqués { $clicked }
crm-revenue-detail = Replies: { $reply } · Unsubscribes: { $unsubscribe } · Bounces: { $bounce } · Complaints: { $complaint } · Skips: { $skip }
crm-revenue-influenced = Influenced deals: { $deals } · won: { $won }
crm-revenue-unattributed = Unattributed touches: { $count }
crm-revenue-unavailable = Revenue attribution is unavailable.
