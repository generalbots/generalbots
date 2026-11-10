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
