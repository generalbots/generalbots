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
