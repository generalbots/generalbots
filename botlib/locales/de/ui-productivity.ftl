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
