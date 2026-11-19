# =============================================================================
# General Bots — de — Billing, products and tickets
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = Analytics-Dashboard
analytics-last-hour = Letzte Stunde
analytics-last-6h = Letzte 6 Stunden
analytics-last-24h = Letzte 24 Stunden
analytics-last-7d = Letzte 7 Tage
analytics-last-30d = Letzte 30 Tage

# =============================================================================
# Billing
# =============================================================================
# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = Abrechnung
billing-invoices = Rechnungen
billing-payments = Zahlungen
billing-quotes = Zitate
billing-dashboard = Armaturenbrett

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = Rechnung
billing-invoice-desc = Rechnung an den Kunden
billing-payment = Zahlung
billing-payment-desc = Zahlung erhalten
billing-quote = Zitat
billing-quote-desc = Preisangebot

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = Fälligkeitsdatum
billing-overdue = Überfällig
billing-paid = Bezahlt
billing-pending = Ausstehend
billing-draft = Entwurf
billing-sent = Gesendet
billing-partial = Teilweise
billing-cancelled = Abgesagt

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = Neue Rechnung
billing-new-quote = Neues Zitat
billing-new-payment = Neue Zahlung
billing-send-invoice = Rechnung senden
billing-record-payment = Zahlung aufzeichnen
billing-mark-paid = Als bezahlt markieren
billing-void = Nichtig

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = Betrag
billing-tax = Steuer
billing-subtotal = Zwischensumme
billing-total = Insgesamt
billing-discount = Rabatt
billing-line-items = Werbebuchungen
billing-add-item = Artikel hinzufügen
billing-remove-item = Artikel entfernen
billing-customer = Kunde
billing-issue-date = Ausgabedatum
billing-payment-terms = Zahlungsbedingungen
billing-notes = Notizen
billing-invoice-number = Rechnungsnummer
billing-quote-number = Angebotsnummer

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = Einnahmen
billing-outstanding = Hervorragend
billing-this-month = Diesen Monat
billing-last-month = Letzten Monat
billing-total-paid = Gesamtbezahlt
billing-total-overdue = Total überfällig
billing-subtitle = Rechnungen, Zahlungen und Angebote
billing-revenue-month = Umsatz in diesem Monat
billing-total-revenue = Gesamtumsatz
billing-paid-month = Diesen Monat bezahlt

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = Keine Rechnungen gefunden
billing-no-payments = Keine Zahlungen gefunden
billing-no-quotes = Keine Zitate gefunden

# =============================================================================
# Products
# =============================================================================
# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = Produkte
products-catalog = Katalog
products-services = Dienstleistungen
products-price-lists = Preislisten
products-inventory = Inventar

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = Produkt
products-product-desc = Physisches oder digitales Produkt
products-service = Service
products-service-desc = Serviceangebot
products-price-list = Preisliste
products-price-list-desc = Preisstufen

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = Neues Produkt
products-new-service = Neuer Dienst
products-new-price-list = Neue Preisliste
products-new-pricelist = Neue Preisliste
products-edit-product = Produkt bearbeiten
products-duplicate = Duplizieren

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = Artikelnummer
products-category = Kategorie
products-price = Preis
products-unit = Einheit
products-stock = Lager
products-cost = Kosten
products-margin = Marge
products-barcode = Barcode

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = Auf Lager
products-out-of-stock = Ausverkauft
products-low-stock = Geringer Lagerbestand
products-active = Aktiv
products-inactive = Inaktiv
products-featured = Hervorgehoben
products-archived = Archiviert

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = Verwalten Sie Produkte, Dienstleistungen und Preise
products-items = Produkte
products-pricelists = Preislisten
products-total-products = Gesamtprodukte
products-total-services = Gesamtleistungen

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = Keine Produkte gefunden
products-no-services = Keine Dienste gefunden
products-no-price-lists = Keine Preislisten gefunden

# =============================================================================
# Tickets (Support Cases)
# =============================================================================
# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = Tickets
tickets-cases = Fälle
tickets-open = Offen
tickets-closed = Geschlossen
tickets-all = Alle Tickets
tickets-my-tickets = Meine Tickets

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = Fall
tickets-case-desc = Support-Ticket
tickets-resolution = Auflösung
tickets-resolution-desc = Von der KI vorgeschlagene Lösung

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = Priorität
tickets-priority-low = Niedrig
tickets-priority-medium = Mittel
tickets-priority-high = Hoch
tickets-priority-urgent = Dringend

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = Status
tickets-status-new = Neu
tickets-status-open = Offen
tickets-status-pending = Ausstehend
tickets-status-resolved = Gelöst
tickets-status-closed = Geschlossen
tickets-status-on-hold = In der Warteschleife

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = Neues Ticket
tickets-assign = Zuweisen
tickets-reassign = Neu zuweisen
tickets-escalate = Eskalieren
tickets-resolve = Lösen
tickets-reopen = Wieder öffnen
tickets-close = Schließen
tickets-merge = Zusammenführen

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = Betreff
tickets-description = Beschreibung
tickets-category = Kategorie
tickets-assigned = Zugewiesen an
tickets-unassigned = Nicht zugewiesen
tickets-created = Erstellt
tickets-updated = Aktualisiert
tickets-response-time = Reaktionszeit
tickets-resolution-time = Lösungszeit
tickets-customer = Kunde
tickets-internal-notes = Interne Notizen
tickets-attachments = Anhänge

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = KI-Vorschlag
tickets-apply-suggestion = Vorschlag anwenden
tickets-ai-summary = KI-Zusammenfassung
tickets-similar-tickets = Ähnliche Tickets
tickets-suggested-articles = Empfohlene Artikel

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = Keine Tickets gefunden
tickets-no-open = Keine offenen Tickets
tickets-no-closed = Keine geschlossenen Tickets
common-no = No
common-yes = Yes
products-billing-cycle = Billing Cycle
products-billing-type = Billing Type *
products-cat-all = All Categories
products-cat-consulting = Consulting
products-cat-development = Development
products-cat-hardware = Hardware
products-cat-implementation = Implementation
products-cat-maintenance = Maintenance
products-cat-other = Other
products-cat-software = Software
products-cat-subscription = Subscription
products-cat-support = Support
products-cat-training = Training
products-col-actions = Actions
products-col-currency = Currency
products-col-default = Default
products-col-description = Description
products-col-items = Items
products-col-name = Name
products-col-price = Price
products-col-status = Status
products-col-type = Type
products-col-unit = Unit
products-col-valid-from = Valid From
products-col-valid-to = Valid To
products-create = Create Product
products-create-service = Create Service
products-currency-all = All Currencies
products-cycle-annually = Annually
products-cycle-monthly = Monthly
products-cycle-quarterly = Quarterly
products-description = Description
products-image = Product Image
products-max-hours = Max. Hours (estimate)
products-min-hours = Min. Hours
products-name = Product Name *
products-new = New Product
products-pricelist-create = Create Price List
products-pricelist-currency = Currency
products-pricelist-description = Description
products-pricelist-markup = Default Markup %
products-pricelist-name = Price List Name *
products-pricelist-type = Price List Type
products-pricelist-type-contract = Contract
products-pricelist-type-promotional = Promotional
products-pricelist-type-standard = Standard
products-pricelist-valid-from = Valid From
products-pricelist-valid-until = Valid Until
products-search-placeholder = Search products, services...
products-select-category = Select category...
products-select-type = Select type...
products-service-name = Service Name *
products-status = Status
products-status-active = Active
products-status-all = All
products-status-inactive = Inactive
products-taxable = Taxable
products-trial-days = Trial Period (days)
products-type-all = All Types
products-type-fixed = Fixed Price
products-type-hourly = Hourly Rate
products-type-recurring = Recurring (Subscription)
products-unit-day = Per Day
products-unit-hour = Hour
products-unit-license = License
products-unit-month = Month
products-unit-project = Per Project
products-unit-seat = Seat
products-unit-unit = Unit
products-unit-year = Year
products-upload-image = Click to upload image

# semantic i18n key (#2542)
billing-placeholder-invoice-amount = 0,00 €
billing-placeholder-amount = 0,00
billing-placeholder-reference = Transaktions-ID, Schecknr. usw.
billing-placeholder-payment-notes = Zusätzliche Zahlungsnotizen...
products-placeholder-service-name = z. B. Technische Beratung
products-placeholder-service-description = Beschreiben Sie das Serviceangebot...
products-placeholder-price = 0,00
products-placeholder-trial-days = 0
products-placeholder-min-hours = 1
products-placeholder-max-hours = 40
products-placeholder-pricelist-name = z. B. Standardpreise 2026
products-placeholder-pricelist-description = Beschreibung der Preisliste...
products-placeholder-markup = 0,00
products-placeholder-product-name = Produktname eingeben
products-placeholder-sku = PRD-001
products-placeholder-product-description = Produktbeschreibung...
products-placeholder-cost = 0,00
# semantic i18n key (#2542)
billing-notes-placeholder = Zusätzliche Hinweise für den Kunden...
billing-invoice-notes-placeholder = Zahlungsanweisungen oder zusätzliche Hinweise...
billing-remove-line-item = Position entfernen

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
products-state-empty-title = Noch keine Produkte
products-state-empty-desc = Produkte, die Sie dem Katalog hinzufügen, erscheinen hier und sind verkaufsbereit.
products-state-empty-action = Erstes Produkt hinzufügen
products-state-empty-services-title = Noch keine Dienstleistungen
products-state-empty-services-desc = Angebotene Dienstleistungen erscheinen hier mit Preis und Einheit.
products-state-empty-services-action = Erste Dienstleistung hinzufügen
products-state-empty-pricelists-title = Noch keine Preislisten
products-state-empty-pricelists-desc = Eine Preisliste gruppiert Produkte unter einer Währung und einem Gültigkeitszeitraum.
products-state-empty-pricelists-action = Preisliste erstellen
billing-payments-empty-title = Noch keine Zahlungen erfasst
billing-payments-empty-desc = Zahlungen, die Sie zu einer Rechnung erfassen, erscheinen hier mit Art und Datum.
billing-quotes-empty-title = Noch keine Angebote
billing-quotes-empty-desc = Senden Sie einem Kunden ein Angebot; es wird hier bis zur Annahme verfolgt.
billing-inventory-empty-title = Noch keine Lagergüter
billing-inventory-empty-desc = Lagerartikel und ihr Bestand erscheinen hier, sobald Sie sie anlegen.
billing-gl-empty-title = Noch keine Sachkonten
billing-gl-empty-desc = Jede Buchung braucht ein Konto. Legen Sie sie an, um Saldenliste und Abschlüsse zu erstellen.
billing-procurement-empty-title = Noch keine Bestellungen
billing-procurement-empty-desc = Bestellungen bei Lieferanten werden hier bis zum Wareneingang verfolgt.
