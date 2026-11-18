# =============================================================================
# General Bots — de — Compliance
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
compliance-title = API-Konformitätsbericht
compliance-subtitle = Sicherheitsscan für alle Bots – Suchen Sie nach Passwörtern, fragilem Code und Fehlkonfigurationen
compliance-export-report = Bericht exportieren
compliance-run-scan = Führen Sie den Compliance-Scan aus
compliance-scanning = Scannen...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Alle Bots
compliance-select-bots = Wählen Sie Bots aus

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Kritisch
compliance-critical-desc = Erfordert sofortiges Handeln
compliance-high = Hoch
compliance-high-desc = Sicherheitsrisiko
compliance-medium = Mittel
compliance-medium-desc = Sollte angegangen werden
compliance-low = Niedrig
compliance-low-desc = Best Practice
compliance-info = Infos
compliance-info-desc = Informativ

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Schweregrad
compliance-filter-type = Typ
compliance-filter-all-severities = Alle Schweregrade
compliance-filter-all-types = Alle Typen
compliance-search-issues = Suchprobleme...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Passwort in der Konfiguration
compliance-type-hardcoded = Hartcodierte Geheimnisse
compliance-type-deprecated = Veraltete Schlüsselwörter
compliance-type-fragile = Fragiler Code
compliance-type-config = Konfigurationsprobleme

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Ergebnisse
compliance-results-count = { $count ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Schweregrad
compliance-col-issue = Problem
compliance-col-location = Standort
compliance-col-details = Einzelheiten
compliance-col-action = Aktion
compliance-view-details = Details anzeigen
compliance-fix-issue = Problem beheben
compliance-ignore = Ignorieren
compliance-no-issues = Keine Probleme gefunden
compliance-no-issues-desc = Großartig! Ihre Bots sind konform.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scan läuft...
compliance-scan-checking = Überprüfen { $item }...
compliance-scan-complete = Scan abgeschlossen
compliance-scan-failed = Der Scan ist fehlgeschlagen

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Rahmenwerke
compliance-new-framework = Neu
compliance-framework-name = Name
compliance-framework-version = Version
compliance-framework-description = Beschreibung
compliance-create-framework = Rahmen erstellen
compliance-controls = Kontrollen
compliance-add-control = Kontrolle hinzufügen
compliance-control-id = Kontroll-ID
compliance-control-title = Titel
compliance-control-category = Kategorie
compliance-control-description = Beschreibung
compliance-mandatory = Obligatorisch
compliance-optional = Optional
compliance-evidence = Beweise
compliance-attach-evidence = Beweise beifügen
compliance-evidence-path = Dateipfad (Laufwerksartefakt)
compliance-evidence-type = Typ
compliance-approve = Genehmigen
compliance-covered = Bedeckt
compliance-no-evidence = Keine Beweise
compliance-export-csv = CSV exportieren
compliance-archive = Archiv
compliance-total-controls = Totale Kontrolle
compliance-coverage = Abdeckung
compliance-no-frameworks = Noch keine Frameworks konfiguriert.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Compliance-Dashboard

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Typ II-Compliance-Status, Kontrollüberwachung und Auditnachweise

# semantic i18n key (#2542)
admin-compliance-compliant = Konform

# semantic i18n key (#2542)
admin-compliance-overall-score = Gesamt-Compliance-Score

# semantic i18n key (#2542)
admin-compliance-controls-passing = Bestandene Kontrollen

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Fällige Nachweise

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Bis zum nächsten Audit

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Geplant: 7. März 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Trust-Service-Kriterien

# semantic i18n key (#2542)
admin-compliance-availability = Verfügbarkeit

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Verarbeitungsintegrität

# semantic i18n key (#2542)
admin-compliance-confidentiality = Vertraulichkeit

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Kontrollen, die Aufmerksamkeit erfordern

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Multi-Faktor-Authentifizierung für privilegierte Konten

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Fällig: 25. Jan. 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Beheben

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Test des Sicherheitsvorfall-Reaktionsplans

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Jährlicher Test um 15 Tage überfällig

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Dokumentation des Auftragsverarbeitungsvertrags

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Fällig: 1. Feb. 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Überprüfung der Datenklassifizierungsrichtlinie

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Richtlinienüberprüfung steht zur Genehmigung aus

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Fällig: 15. Feb. 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Aktualisierungen der Datenschutzhinweise

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = Datenschutzrichtlinie muss für neue Datenerhebung aktualisiert werden

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Fällig: 28. Feb. 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Nachweissammlung

# semantic i18n key (#2542)
admin-compliance-current = Aktuell

# semantic i18n key (#2542)
admin-compliance-due-soon = Bald fällig

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Penetrationstest-Bericht Q4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Schwachstellenbewertung

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Zugriffsüberprüfungsbericht - Januar 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Zugriffskontrolle

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Fällig in 3 Tagen

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Ergebnisse des Backup-Wiederherstellungstests

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Sicherung und Wiederherstellung

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Fällig in 5 Tagen

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Schulungsnachweise zur Sicherheitsbewusstseinsschulung

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Sicherheitsschulung

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Alle Nachweise anzeigen

# semantic i18n key (#2542)
admin-compliance-access-events = Zugriffsereignisse

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Konfigurationsänderungen

# semantic i18n key (#2542)
admin-compliance-security-events = Sicherheitsereignisse

# semantic i18n key (#2542)
admin-compliance-compliance-events = Compliance-Ereignisse

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Fehlgeschlagener Anmeldeversuch

# semantic i18n key (#2542)
admin-compliance-failed-login-details = Benutzer: admin@company.com | IP: 192.168.1.105 | Grund: Ungültiges Passwort

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Benutzer angemeldet

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Benutzer: sarah.johnson@company.com | IP: 10.0.0.45
# semantic i18n key (#2542) — compliance app shell
compliance-run-check = Prüfung ausführen
compliance-stat-total-checks = Prüfungen gesamt
compliance-stat-passed = Bestanden
compliance-stat-failed = Fehlgeschlagen
compliance-stat-warnings = Warnungen
compliance-stat-score = Punktzahl
compliance-tab-dashboard = Dashboard
compliance-tab-checks = Prüfungen
compliance-tab-issues = Probleme
compliance-tab-audit-log = Audit-Protokoll
compliance-tab-risk-matrix = Risikomatrix
compliance-tab-training = Schulung
compliance-recent-activity = Letzte Aktivitäten
compliance-score-heading = Compliance-Punktzahl
compliance-check-results-heading = Prüfergebnisse
compliance-checks-heading = Compliance-Prüfungen
compliance-issues-heading = Probleme
compliance-audit-log-heading = Audit-Protokoll
compliance-risk-matrix-title = Risikomatrix (Wahrscheinlichkeit x Auswirkung)
compliance-impact-axis = Auswirkung →
compliance-likelihood-axis = Wahrscheinlichkeit
compliance-training-records = Schulungsnachweise
compliance-loading = Laden...
compliance-no-recent-activity = Keine aktuellen Aktivitäten
compliance-no-checks-found = Keine Prüfungen gefunden
compliance-no-issues-found = Keine Probleme gefunden
compliance-no-audit-entries = Keine Audit-Einträge
compliance-no-risks-recorded = Keine Risiken erfasst
compliance-no-training-records = Keine Schulungsnachweise
compliance-overall-compliance-score = Gesamt-Compliance-Punktzahl
compliance-col-id = ID
compliance-col-framework = Rahmenwerk
compliance-col-status = Status
compliance-col-created = Erstellt
compliance-col-timestamp = Zeitstempel
compliance-col-user = Benutzer
compliance-action-run = Ausführen
compliance-create-issue = Problem erstellen
compliance-new-training = Neue Schulung
compliance-framework-details = Rahmenwerkdetails
compliance-failed-load-framework = Rahmenwerk konnte nicht geladen werden
compliance-risk-score = Punktzahl: { $score }
compliance-issue-title-placeholder = Problemtitel
compliance-issue-assignee-placeholder = Name des Verantwortlichen
compliance-issue-description-placeholder = Beschreiben Sie das Problem...
compliance-training-title-placeholder = Schulungstitel
compliance-training-assignee-placeholder = Name des Verantwortlichen
compliance-due-date = Fälligkeitsdatum
compliance-due-label = Fällig:
compliance-col-assignee = Verantwortlich
compliance-create-training = Schulung erstellen
