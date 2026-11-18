# =============================================================================
# General Bots — de — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Aufgaben
tasks-new = Neue Aufgabe
tasks-due-today = Heute fällig
tasks-overdue = Überfällig
tasks-completed = Abgeschlossen
tasks-all = Alle Aufgaben
tasks-priority-high = Hohe Priorität
tasks-priority-medium = Mittlere Priorität
tasks-priority-low = Niedrige Priorität
tasks-no-due-date = Kein Fälligkeitsdatum
tasks-add-subtask = Unteraufgabe hinzufügen
tasks-mark-complete = Als abgeschlossen markieren
tasks-mark-incomplete = Als unvollständig markieren
tasks-delete-confirm = Sind Sie sicher, dass Sie diese Aufgabe löschen möchten?
tasks-count = { $count ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Kalender
calendar-today = Heute
calendar-new-event = Neue Veranstaltung
calendar-all-day = Den ganzen Tag
calendar-repeat = Wiederholen
calendar-reminder = Erinnerung
calendar-view-day = Tag
calendar-view-week = Woche
calendar-view-month = Monat
calendar-view-year = Jahr
calendar-no-events = Keine Veranstaltungen geplant
calendar-event-title = Veranstaltungstitel
calendar-event-location = Standort
calendar-event-description = Beschreibung
calendar-event-attendees = Teilnehmer

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Treffen
meet-join = Treten Sie der Besprechung bei
meet-start = Treffen starten
meet-mute = Stumm
meet-unmute = Stummschaltung aufheben
meet-video-on = Kamera an
meet-video-off = Kamera aus
meet-share-screen = Bildschirm teilen
meet-stop-sharing = Hören Sie auf zu teilen
meet-end-call = Anruf beenden
meet-leave = Besprechung verlassen
meet-participants = { $count ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = Wartezimmer
meet-admit = Geben Sie zu
meet-remove = Entfernen
meet-chat = Meeting-Chat
meet-raise-hand = Hand heben
meet-lower-hand = Untere Hand
meet-recording = Aufnahme läuft
meet-start-recording = Starten Sie die Aufnahme
meet-stop-recording = Stoppen Sie die Aufnahme

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Post
mail-compose = Komponieren
mail-inbox = Posteingang
mail-sent = Gesendet
mail-drafts = Entwürfe
mail-trash = Müll
mail-spam = Spam
mail-starred = Mit einem Stern versehen
mail-archive = Archiv
mail-to = Zu
mail-cc = CC
mail-bcc = BCC
mail-subject = Betreff
mail-body = Nachricht
mail-reply = Antwort
mail-reply-all = Allen antworten
mail-forward = Vorwärts
mail-send = Senden
mail-discard = Verwerfen
mail-save-draft = Entwurf speichern
mail-attach = Dateien anhängen
mail-unread = { $count ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = Ihr Posteingang ist leer
mail-no-subject = (Kein Thema)

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Papier
paper-new-note = Neue Notiz
paper-search-notes = Notizen durchsuchen...
paper-quick-start = Schnellstart
paper-template-blank = Leer
paper-template-meeting = Treffen
paper-template-todo = Zu erledigen
paper-template-research = Forschung
paper-untitled = Ohne Titel
paper-placeholder = Beginnen Sie mit dem Schreiben oder geben Sie / für Befehle ein ...
paper-commands = Befehle
paper-heading1 = Überschrift 1
paper-heading1-desc = Große Abschnittsüberschrift
paper-heading2 = Überschrift 2
paper-heading2-desc = Überschrift des mittleren Abschnitts
paper-heading3 = Überschrift 3
paper-heading3-desc = Kleine Abschnittsüberschrift
paper-paragraph = Absatz
paper-paragraph-desc = Klartext
paper-bullet-list = Aufzählungsliste
paper-bullet-list-desc = Ungeordnete Liste
paper-numbered-list = Nummerierte Liste
paper-numbered-list-desc = Geordnete Liste
paper-todo-list = To-Do-Liste
paper-todo-list-desc = Überprüfbare Aufgabenliste
paper-quote = Zitat
paper-quote-desc = Blockquote für Zitate
paper-divider = Teiler
paper-divider-desc = Horizontale Linie
paper-code-block = Codeblock
paper-code-block-desc = Formatierter Code
paper-table = Tisch
paper-table-desc = Tabelle einfügen
paper-image = Bild
paper-image-desc = Bild von URL einfügen
paper-callout = Hinweis
paper-callout-desc = Hervorgehobenes Informationsfeld
paper-ai-write = KI-Schreiben
paper-ai-write-desc = Generieren Sie Text mit KI
paper-ai-summarize = KI zusammenfassen
paper-ai-summarize-desc = Fassen Sie den ausgewählten Text zusammen
paper-ai-expand = KI erweitern
paper-ai-expand-desc = Erweitern Sie den ausgewählten Text
paper-ai-improve = KI verbessern
paper-ai-improve-desc = Verbessern Sie die Schreibqualität
paper-ai-translate = KI-Übersetzung
paper-ai-translate-desc = In eine andere Sprache übersetzen
paper-ai-assistant = KI-Assistent
paper-ai-quick-actions = Schnelle Aktionen
paper-ai-rewrite = Umschreiben
paper-ai-make-shorter = Kürzer machen
paper-ai-make-longer = Länger machen
paper-ai-fix-grammar = Korrigieren Sie die Grammatik
paper-ai-tone = Ton
paper-ai-tone-professional = Professionell
paper-ai-tone-casual = Lässig
paper-ai-tone-friendly = Freundlich
paper-ai-tone-formal = Formell
paper-ai-translate-to = Übersetzen nach
paper-ai-custom-prompt = Benutzerdefinierte Eingabeaufforderung
paper-ai-custom-placeholder = Beschreiben Sie, was Sie wollen...
paper-ai-generate = Generieren
paper-ai-response = KI-Reaktion
paper-ai-apply = Bewerben
paper-ai-regenerate = Regenerieren
paper-ai-copy = Kopieren
paper-word-count = { $count } Wörter
paper-char-count = { $count } Zeichen
paper-saved = Gespeichert
paper-saving = Sparen...
paper-last-edited = Zuletzt bearbeitet: { $time }
paper-last-edited-now = Zuletzt bearbeitet: Gerade eben
paper-export = Dokument exportieren
paper-export-pdf = PDF
paper-export-docx = Word (.docx)
paper-export-markdown = Abschlag
paper-export-html = HTML
paper-export-txt = Klartext

# Tasks translations
tasks-active = Aktive Absichten
tasks-awaiting = Warten auf Entscheidung
tasks-paused = Angehalten
tasks-blocked = Blockiert/Probleme
tasks-time-saved = Aktive Zeitersparnis:
tasks-input-placeholder = Was möchten Sie tun? z. B. „Erstelle eine CRM-App“ oder „Erinnere mich daran, John morgen anzurufen“

# Calendar additional translations
calendar-my-calendars = Meine Kalender

# Email additional translations
email-scheduled = Geplant
email-tracking = Verfolgung

# Email folder translations
email-inbox = Posteingang
email-starred = Mit einem Stern versehen
email-sent = Gesendet
email-drafts = Entwürfe
email-spam = Spam
email-trash = Müll
email-compose = Komponieren

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Forschung
research-search-placeholder = Fragen Sie alles...
research-collections = Sammlungen
research-new-collection = Neue Kollektion
research-recent = Neu
research-academic = Akademisch
research-code = Code
research-internal = Intern
research-search-all = Alles durchsuchen
research-academic-papers = Akademische Arbeiten
research-code-docs = Code und Dokumentation
research-internal-kb = Interne Wissensdatenbank
research-sources = Quellen
research-trending = Im Trend
research-pro-search = Pro-Suche
research-include-images = Fügen Sie Bilder hinzu
research-try-asking = Versuchen Sie, nachzufragen
research-related = Verwandte Fragen
research-view-all-sources = Alle Quellen anzeigen
research-export-citations = Zitate exportieren
research-save-to-collection = In der Sammlung speichern

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = Neues Treffen
meet-join-meeting = Treten Sie der Besprechung bei
meet-active-rooms = Aktive Räume
meet-room-title = Besprechungsraum
meet-record = Aufnehmen
meet-camera = Kamera
meet-share = Teilen
meet-info = Infos
meet-more = Mehr
meet-share-meeting = Aktientreffen
meet-meeting-title = Titel der Besprechung
meet-meeting-code = Besprechungscode
meet-meeting-link = Link zum Treffen
meet-send-invite = Einladung senden

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Unbenanntes Ziel
tasks-detail-goal-progress = Ziel-Fortschritt
tasks-detail-current-value = Aktueller Wert
tasks-detail-target-value = Zielwert
tasks-detail-percent = Fortschritt
tasks-detail-no-actions = Noch keine Aktionen
tasks-detail-pending-info = Fehlende Informationen
tasks-detail-scheduler-info = Planerinformationen
tasks-detail-monitor-info = Monitorinformationen
tasks-detail-unavailable = Nicht verfügbar
tasks-detail-load-failed = Zieldetails konnten nicht geladen werden


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Zeile einfügen
db-insert-into = Einfügen in
db-provide-value = Geben Sie mindestens einen Wert an
db-insert-failed = Einfügen fehlgeschlagen
db-confirm-title = Bestätigen
db-confirm-accept = Bestätigen
db-confirm-delete = Diese Zeile löschen?
db-confirm-drop = Tabelle löschen
db-delete-failed = Löschen fehlgeschlagen
db-drop-failed = Löschen der Tabelle fehlgeschlagen

# Tasks detail view (#2307)
tasks-detail-type-goal = Goal
tasks-detail-status-pending = Pending
tasks-detail-pending-title = Pending items
tasks-detail-required-reason = A reason is required.
tasks-detail-type-scheduler = Scheduler
tasks-detail-type-monitor = Monitor
tasks-detail-last-action = Last action
tasks-detail-next-run = Next run
tasks-detail-file = File
tasks-detail-interval = Interval
tasks-detail-last-check = Last check
tasks-detail-last-value = Last value
tasks-detail-progress = Progress
tasks-detail-app = App
tasks-detail-field = Field
tasks-detail-reason = Reason
tasks-detail-cron = Cron

# Database dialog copy (#2307)
db-error = Error
db-database = Database
db-empty-query = Execute a query to see results
db-insert = Insert
db-select-table = Select a table
tasks-load-error = Aufgaben konnten nicht geladen werden.
tasks-sign-in-to-load = Melden Sie sich an, um Aufgaben zu laden.
calendar-collapse-sidebar = Seitenleiste einklappen
calendar-prev-month = Vorheriger Monat
calendar-next-month = Nächster Monat
calendar-prev-period = Vorheriger Zeitraum
calendar-next-period = Nächster Zeitraum
calendar-add-calendar = Kalender hinzufügen
paper-toggle-sidebar = Seitenleiste umschalten
paper-insert-ai-response = KI-Antwort einfügen
paper-replace-selection = Auswahl ersetzen
paper-highlight = Hervorheben
paper-align-left = Linksbündig
paper-align-center = Zentrieren
paper-align-right = Rechtsbündig
paper-insert-link = Link einfügen
paper-insert-image = Bild einfügen
paper-insert-table = Tabelle einfügen
paper-bold = Fett
paper-italic = Kursiv
paper-underline = Unterstrichen
paper-strikethrough = Durchgestrichen
products-view-grid = Rasteransicht
products-view-list = Listenansicht
sheet-freeze-panes = Bereiche fixieren

# semantic i18n key (#2542)
db-schema-title = 🗄️ Datenbankschema

# semantic i18n key (#2542)
db-loading-tables = Tabellen werden geladen...

# semantic i18n key (#2542)
db-query-builder = Abfrage-Generator

# semantic i18n key (#2542)
db-run-query = Abfrage ausführen

# semantic i18n key (#2542)
db-select-table-hint = Wählen Sie eine Tabelle aus, um Daten anzuzeigen

# semantic i18n key (#2542)
db-prev-page = ◀ Zurück

# semantic i18n key (#2542)
db-page-info-default = Seite 1 von 1

# semantic i18n key (#2542)
db-next-page = Weiter ▶

# semantic i18n key (#2542)
research-searching-sources = Quellen werden durchsucht...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Neueste KI-Entwicklungen

# semantic i18n key (#2542)
research-suggestion-quantum = Quantencomputing erklärt

# semantic i18n key (#2542)
research-suggestion-software-architecture = Softwarearchitektur

# semantic i18n key (#2542)
research-suggestion-ai-productivity = KI-Produktivitätstools

# semantic i18n key (#2542)
research-related-heading = Verwandt

# semantic i18n key (#2542)
research-loading = Recherche wird geladen…

# semantic i18n key (#2542)
research-service-unavailable = Recherche-Dienst nicht verfügbar

# semantic i18n key (#2542)
calendar-mini-month-placeholder = Januar 2025

# semantic i18n key (#2542)
calendar-upcoming = Demnächst

# semantic i18n key (#2542)
calendar-import-export = Importieren / Exportieren

# semantic i18n key (#2542)
calendar-popup-event-title = Veranstaltungstitel

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Konferenzraum A

# semantic i18n key (#2542)
calendar-popup-event-description = Veranstaltungsbeschreibung

# semantic i18n key (#2542)
calendar-personal = Persönlich

# semantic i18n key (#2542)
calendar-import-ical-title = iCal-Datei importieren

# semantic i18n key (#2542)
calendar-select-ics-file = .ics-Datei auswählen

# semantic i18n key (#2542)
calendar-import-to-calendar-label = In Kalender importieren

# semantic i18n key (#2542)
calendar-default-calendar = Standardkalender

# semantic i18n key (#2542)
calendar-loading = Kalender wird geladen…

# semantic i18n key (#2542)
calendar-unavailable = Kalender nicht verfügbar

# semantic i18n key (#2542)
tasks-progress-status = STATUS

# semantic i18n key (#2542)
tasks-progress-sample-title = Benutzerauthentifizierungssystem implementieren

# semantic i18n key (#2542)
tasks-progress-runtime = Laufzeit:

# semantic i18n key (#2542)
tasks-progress-sample-action = Strategie für Token-Ablauf wählen

# semantic i18n key (#2542)
tasks-progress-estimated = Geschätzt:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Genehmigung für die Sicherheitskonfiguration erforderlich

# semantic i18n key (#2542)
tasks-progress-log = FORTSCHRITTSPROTOKOLL

# semantic i18n key (#2542)
tasks-progress-terminal-live = TERMINAL (LIVE-AGENTENAKTIVITÄT)

# semantic i18n key (#2542)
tasks-progress-processed = Verarbeitet:

# semantic i18n key (#2542)
tasks-progress-data-points = Datenpunkte

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 Quellen/Min.

# semantic i18n key (#2542)
tasks-progress-sample-insight = Wettbewerbsanalysen werden zusammengefasst...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Preisdaten werden abgeglichen...

# semantic i18n key (#2542)
tasks-progress-view-details = Details anzeigen ▸

# semantic i18n key (#2542)
tasks-create-run = Erstellen und ausführen

# semantic i18n key (#2542)
tasks-loading-tasks = Aufgaben werden geladen...

# semantic i18n key (#2542)
tasks-select-task = Wählen Sie eine Aufgabe aus

# semantic i18n key (#2542)
tasks-select-task-desc = Klicken Sie auf eine Aufgabe in der Liste, um Details anzuzeigen

# semantic i18n key (#2542)
tasks-bot-database-label = Bot-Datenbank:

# semantic i18n key (#2542)
tasks-shared-resources-label = Gemeinsame Ressourcen:

# semantic i18n key (#2542)
tasks-starting = Wird gestartet...

# semantic i18n key (#2542)
tasks-create-new-intent = Neue Absicht erstellen

# semantic i18n key (#2542)
tasks-intent-accomplish-label = Was möchten Sie erreichen?

# semantic i18n key (#2542)
tasks-execution-mode = Ausführungsmodus

# semantic i18n key (#2542)
tasks-mode-automatic = Automatisch

# semantic i18n key (#2542)
tasks-mode-supervised = Überwacht

# semantic i18n key (#2542)
tasks-mode-manual-approval = Manuelle Genehmigung

# semantic i18n key (#2542)
tasks-make-decision = Entscheidung treffen

# semantic i18n key (#2542)
tasks-decision-required = Entscheidung erforderlich

# semantic i18n key (#2542)
tasks-loading-decision-details = Entscheidungsdetails werden geladen...

# semantic i18n key (#2542)
tasks-skip-for-now = Vorerst überspringen

# semantic i18n key (#2542)
tasks-confirm-decision = Entscheidung bestätigen

# semantic i18n key (#2542)
tasks-due-soon = Bald fällig

# semantic i18n key (#2542)
tasks-autotask-items = // AUTOTASK-ELEMENTE

# semantic i18n key (#2542)
tasks-edit-task = Aufgabe bearbeiten

# semantic i18n key (#2542)
tasks-assignee-user-id = Zugewiesen (Benutzer-ID)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Übergeordnete Aufgabe (Unteraufgabe von…)

# semantic i18n key (#2542)
tasks-reopen-action = ↩ Wieder öffnen

# semantic i18n key (#2542)
tasks-complete-action = ✓ Abschließen

# semantic i18n key (#2542)
tasks-delete-action = 🗑 Löschen

# semantic i18n key (#2542)
db-schema-browser = Schema-Browser

# semantic i18n key (#2542)
db-loading-schema = Schema wird geladen...

# semantic i18n key (#2542)
db-new-table = + Neue Tabelle

# semantic i18n key (#2542)
db-previous = ← Zurück

# semantic i18n key (#2542)
db-next = Weiter →

# semantic i18n key (#2542)
db-sql-console = SQL-Konsole

# semantic i18n key (#2542)
db-execute-query = Ausführen (Strg+Eingabe)

# semantic i18n key (#2542)
db-explain = EXPLAIN

# semantic i18n key (#2542)
db-create-new-table = Neue Tabelle erstellen

# semantic i18n key (#2542)
db-table-name = Tabellenname

# semantic i18n key (#2542)
db-columns = Spalten

# semantic i18n key (#2542)
db-not-null = NOT NULL

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR(255)

# semantic i18n key (#2542)
db-type-text = TEXT

# semantic i18n key (#2542)
db-type-integer = INTEGER

# semantic i18n key (#2542)
db-type-bigint = BIGINT

# semantic i18n key (#2542)
db-type-decimal = DECIMAL

# semantic i18n key (#2542)
db-type-boolean = BOOLEAN

# semantic i18n key (#2542)
db-type-date = DATE

# semantic i18n key (#2542)
db-type-timestamptz = TIMESTAMPTZ

# semantic i18n key (#2542)
db-type-jsonb = JSONB

# semantic i18n key (#2542)
db-add-column = + Spalte hinzufügen

# semantic i18n key (#2542)
db-create-table = Tabelle erstellen

# semantic i18n key (#2542)
db-edit-row = Zeile bearbeiten

# semantic i18n key (#2542)
db-import-csv = CSV importieren

# semantic i18n key (#2542)
db-select-csv-file = CSV-Datei auswählen

# semantic i18n key (#2542)
analytics-ask-placeholder = Fragen Sie nach Ihren Analysedaten...
tasks-intent-description-placeholder = Beschreiben Sie, was Sie erstellen oder automatisieren möchten...
tasks-assignee-placeholder = UUID des Verantwortlichen (optional)
tasks-parent-placeholder = UUID der übergeordneten Aufgabe (optional)

# semantic i18n key (#2542)
db-placeholder-search-tables = Tabellen suchen...
db-aria-close-sql-console = SQL-Konsole schließen
db-placeholder-sql-query = SELECT * FROM tabellenname WHERE bedingung...
db-aria-sql-query = SQL-Abfrage
db-placeholder-table-name-example = meine_tabelle
db-placeholder-column-id = id
db-placeholder-column-name-example = spaltenname
