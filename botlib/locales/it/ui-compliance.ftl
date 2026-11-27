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
compliance-title = Report di conformità API
compliance-subtitle = Scansione di sicurezza per tutti i bot - Controllare le password, il codice fragile e le configurazioni errate
compliance-export-report = Esporta rapporto
compliance-run-scan = Esegui scansione della conformità
compliance-scanning = Scansione...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Tutti i bot
compliance-select-bots = Seleziona bot

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Critico
compliance-critical-desc = Richiede un'azione immediata
compliance-high = Alto
compliance-high-desc = Rischio di sicurezza
compliance-medium = Media
compliance-medium-desc = Dovresti essere trattato
compliance-low = Bassa
compliance-low-desc = Migliore pratica
compliance-info = Informazioni
compliance-info-desc = Informazioni

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Severità
compliance-filter-type = Tipo
compliance-filter-all-severities = Tutte le gravità
compliance-filter-all-types = Tutti i tipi
compliance-search-issues = Problemi di ricerca...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Password in Config
compliance-type-hardcoded = Segreti codificati
compliance-type-deprecated = Parole chiave deprecate
compliance-type-fragile = Codice fragile
compliance-type-config = Problemi di configurazione

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Risultati
compliance-results-count = { $count ->
    [one] Trovato problema { $count }
    *[other] Problemi { $count } trovati
    }
compliance-col-severity = Severità
compliance-col-issue = Questione
compliance-col-location = Ubicazione
compliance-col-details = Dettagli
compliance-col-action = Azione
compliance-view-details = Visualizza dettagli
compliance-fix-issue = Risolvi problema
compliance-ignore = Ignora
compliance-no-issues = Nessun problema trovato
compliance-no-issues-desc = I tuoi bot sono conformi.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scansione in corso...
compliance-scan-checking = Verifica di { $item }...
compliance-scan-complete = Scansione completata
compliance-scan-failed = Scansione non riuscita

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Quadri
compliance-new-framework = Nuovo
compliance-framework-name = Nome
compliance-framework-version = Versione
compliance-framework-description = Descrizione
compliance-create-framework = Crea framework
compliance-controls = Controlli
compliance-add-control = Aggiungi controllo
compliance-control-id = ID di controllo
compliance-control-title = Titolo
compliance-control-category = Categoria
compliance-control-description = Descrizione
compliance-mandatory = Obbligatorio
compliance-optional = Facoltativo
compliance-evidence = Evidenze
compliance-attach-evidence = Allega prove
compliance-evidence-path = Percorso file (fatto di guida)
compliance-evidence-type = Tipo
compliance-approve = Approva
compliance-covered = Coperto
compliance-no-evidence = Nessuna prova
compliance-export-csv = Esporta CSV
compliance-archive = Archivio
compliance-total-controls = Totale controlli
compliance-coverage = Copertura
compliance-no-frameworks = Nessun framework configurato.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Dashboard di conformità

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Stato di conformità di tipo II, controllo e prove di audit

# semantic i18n key (#2542)
admin-compliance-compliant = Conforme

# semantic i18n key (#2542)
admin-compliance-overall-score = Valutazione complessiva della conformità

# semantic i18n key (#2542)
admin-compliance-controls-passing = Controlli superati

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Evidenze in scadenza

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Fino al prossimo audit

# semantic i18n key (#2542)
admin-compliance-next-audit-date = In programma: 7 marzo 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Criteri di servizio di fiducia

# semantic i18n key (#2542)
admin-compliance-availability = Disponibilità

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Elaborazione integrità

# semantic i18n key (#2542)
admin-compliance-confidentiality = Riservatezza

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Controlli che richiedono attenzione

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Autenticazione multi-fattori per account privilegiati

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Scadenza: 25 gennaio 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Rimedia

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Prova del piano di risposta agli incidenti di sicurezza

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Test annuale in ritardo di 15 giorni

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Documentazione dell'accordo sul trattamento dei dati

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Scadenza: 1 febbraio 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Revisione dei criteri di classificazione dei dati

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Riesame politico in attesa dell'approvazione

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Scadenza: 15 febbraio 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Aggiornamenti della Privacy

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = La politica sulla privacy necessita di aggiornamento per la nuova raccolta dei dati

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Scadenza: 28 febbraio 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Raccolta delle evidenze

# semantic i18n key (#2542)
admin-compliance-current = Corrente

# semantic i18n key (#2542)
admin-compliance-due-soon = Scadenza imminente

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Rapporto della prova di penetrazione Q4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Valutazione delle vulnerabilità

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Rapporto di revisione accessi - Gennaio 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Controllo accessi

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = In scadenza tra 3 giorni

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Risultati del test di ripristino backup

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Backup e ripristino

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Scadenza tra 5 giorni

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Informazioni sulla formazione per la sensibilizzazione alla sicurezza

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Formazione sulla sicurezza

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Visualizza tutte le prove

# semantic i18n key (#2542)
admin-compliance-access-events = Eventi di accesso

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Modifiche configurazione

# semantic i18n key (#2542)
admin-compliance-security-events = Eventi di sicurezza

# semantic i18n key (#2542)
admin-compliance-compliance-events = Eventi di conformità

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Tentativo di accesso non riuscito

# semantic i18n key (#2542)
admin-compliance-failed-login-details = Utente: admin@company.com | IP: 192.168.1.105 | Motivo: Password non valida

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Accesso utente

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Utente: Sarah.johnson@company.com | IP: 10.0.0.0.45
# semantic i18n key (#2542) — compliance app shell
compliance-run-check = Esegui controllo
compliance-stat-total-checks = Totale controlli
compliance-stat-passed = Passato
compliance-stat-failed = Non riuscito
compliance-stat-warnings = Avvertenze
compliance-stat-score = Punteggio
compliance-tab-dashboard = Cruscotto
compliance-tab-checks = Verifiche
compliance-tab-issues = Questioni
compliance-tab-audit-log = Registro di controllo
compliance-tab-risk-matrix = Matrice di rischio
compliance-tab-training = Formazione
compliance-recent-activity = Attività recente
compliance-score-heading = Punteggio di conformità
compliance-check-results-heading = Risultati del controllo
compliance-checks-heading = Controlli di conformità
compliance-issues-heading = Questioni
compliance-audit-log-heading = Registro di controllo
compliance-risk-matrix-title = Matrice di rischio (Likelihood x Impact)
compliance-impact-axis = Impatto →
compliance-likelihood-axis = Probabilità
compliance-training-records = Registri di formazione
compliance-loading = Caricamento in corso...
compliance-no-recent-activity = Nessuna attività recente
compliance-no-checks-found = Nessun controllo trovato
compliance-no-issues-found = Nessun problema trovato
compliance-no-audit-entries = Nessuna voce di audit
compliance-no-risks-recorded = Nessun rischio registrato
compliance-no-training-records = Nessun registro di addestramento
compliance-overall-compliance-score = Valutazione complessiva della conformità
compliance-col-id = ID
compliance-col-framework = Quadro
compliance-col-status = Stato
compliance-col-created = Creato
compliance-col-timestamp = Orario
compliance-col-user = Utente
compliance-action-run = Esegui
compliance-create-issue = Crea problema
compliance-new-training = Nuova formazione
compliance-framework-details = Dettagli quadro
compliance-failed-load-framework = Impossibile caricare il framework
compliance-risk-score = Punteggio: { $score }
compliance-issue-title-placeholder = Titolo
compliance-issue-assignee-placeholder = Nome assegnatario
compliance-issue-description-placeholder = Descriva il problema...
compliance-training-title-placeholder = Titolo della formazione
compliance-training-assignee-placeholder = Nome assegnatario
compliance-due-date = Data di scadenza
compliance-due-label = Scadenza:
compliance-col-assignee = Assegnatario
compliance-create-training = Crei una formazione
