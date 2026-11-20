# =============================================================================
# General Bots — de — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = Chatten
chat-placeholder = Geben Sie Ihre Nachricht ein...
chat-send = Senden
chat-new-conversation = Neues Gespräch
chat-history = Chat-Verlauf
chat-clear = Chat löschen
chat-export = Chat exportieren
chat-typing = { $name } tippt...
chat-online = Online
chat-offline = Offline
chat-last-seen = Zuletzt gesehen { $time }
chat-mention-title = Referenzunternehmen
chat-mention-placeholder = Nachricht... (Geben Sie zur Erwähnung @ ein)
chat-mention-search = Entitäten suchen...
chat-mention-no-results = Keine Ergebnisse gefunden
chat-mention-type-hint = Geben Sie Folgendes ein, um zu suchen

# Additional Chat translations
chat-camera = Kamera
chat-camera-title = Foto aufnehmen
chat-camera-capture = Foto aufnehmen
chat-camera-cancel = Abbrechen
chat-camera-sent = Foto gesendet.
chat-camera-denied = Die Kameraberechtigung wurde verweigert. Erlauben Sie der Desktop-App den Kamerazugriff in den Systemeinstellungen, um Fotos aufzunehmen.
chat-camera-missing = Auf diesem Gerät wurde keine Kamera gefunden.
chat-camera-error = Die Kamera konnte nicht gestartet werden.
chat-camera-unsupported = Diese Desktop-App bietet keinen Kamerazugriff.
voice-mode-title = Sprachgespräch
voice-mode-listening = Zuhören… sprechen Sie natürlich
voice-mode-thinking = Denke nach…
voice-mode-speaking = Spricht…
voice-mode-idle = Sprachgespräch
voice-mode-end = Sprachgespräch beenden
chat-message-placeholder = Nachricht...

# Chat message controls and notices
chat-copy-message = Nachricht kopieren
chat-copied = Kopiert
chat-message-copied = Nachricht kopiert
chat-copy-failed = Kopieren fehlgeschlagen
chat-listen-tts = Anhören (TTS)
chat-listen-message = Diese Nachricht anhören
chat-image-unavailable = Bild nicht verfügbar
chat-open = Öffnen
chat-expand = Erweitern
chat-collapse = Einklappen
chat-preview = Vorschau
chat-close-preview = Vorschau schließen
chat-preview-load-failed = Vorschau konnte nicht geladen werden.
chat-show-more = Mehr anzeigen
chat-show-less = Weniger anzeigen
chat-tool = Werkzeug
chat-param = Parameter
chat-params = Parameter
chat-tool-result = Werkzeugergebnis
chat-retry = Erneut versuchen
chat-open-as-document = Als Dokument öffnen
chat-show-more-history = mehr anzeigen…
chat-disconnected-reconnecting = Chat getrennt. Verbindung wird wiederhergestellt...
chat-connect-retrying = Chat-Server nicht erreichbar. Erneuter Versuch...
chat-connection-lost = Verbindung verloren. Neuverbindung wird versucht...
chat-attachment-sent = Anhang gesendet: 
chat-attachments-sent = Anhänge gesendet: 
chat-message-not-delivered = Nachricht nicht zugestellt — Verbindung wird wiederhergestellt
chat-attachment-read-failed = Ein Anhang konnte nicht gelesen werden. Bitte erneut anhängen.
chat-attachment-queued = Verbindung gleich wieder da — Anhang in Warteschlange
chat-reconnecting-queued = Verbindung wird wiederhergestellt — Ihre Nachricht wird gesendet, sobald die Verbindung zurück ist.
chat-bot-private-signin = Dieser Bot ist privat — melden Sie sich an, um fortzufahren
chat-approved-plan = Genehmigter Plan: 
chat-retrying-attachment = Anhang wird erneut gesendet
chat-expanded-preview = Erweiterte Vorschau

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Aktionen der Unterhaltung
chat-export-md = Unterhaltung als Markdown exportieren
chat-export-print = Drucken oder als PDF speichern
chat-export-done = Unterhaltung exportiert
chat-export-empty = Noch nichts zu exportieren
chat-export-untitled = Unterhaltung
chat-export-user = Sie
chat-export-assistant = Assistent
chat-feedback-label = Antwortbewertung
chat-feedback-good = Gute Antwort
chat-feedback-bad = Schlechte Antwort
chat-feedback-comment = Was könnte besser sein?
chat-feedback-send = Senden
chat-staged-files = Angehängte Dateien
chat-staged-close = Vorschau schließen
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = Bot
chat-status-session = Sitzung
chat-status-model = Modell
chat-status-state = Status
chat-status-no-session = —
chat-status-copy-session = Sitzungs-ID kopieren
chat-status-copied = Kopiert
chat-status-idle = Leerlauf
chat-status-streaming = Streaming
chat-status-thinking = Denkt
chat-status-error = Fehler
chat-status-stopped = Gestoppt

chat-response-ready = Antwort bereit
chat-response-started = Assistent antwortet
chat-tool-result-ready = Toolergebnis verfügbar

# semantic i18n key (#2542)
chat-no-suggestions-available = Keine Vorschläge verfügbar

# semantic i18n key (#2542)
chat-start-conversation = Gespräch starten

# semantic i18n key (#2542)
chat-empty-hint = Geben Sie unten eine Nachricht ein oder verwenden Sie die Spracheingabe

# semantic i18n key (#2542)
chat-no-conversations-yet = Noch keine Gespräche

# semantic i18n key (#2542)
chat-context-label = Kontext:

# semantic i18n key (#2542)
chat-select-context = Kontext auswählen...

# semantic i18n key (#2542)
chat-no-contexts-configured = Keine Kontexte konfiguriert

# semantic i18n key (#2542)
projector-title = Inhaltsanzeige

# semantic i18n key (#2542)
projector-loading-content = Inhalt wird geladen...

# semantic i18n key (#2542)
projector-slide-info = Folie 1 von 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = Drehen

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Beginnen Sie das Gespräch
chat-empty-desc = Schreiben Sie unten eine Nachricht oder wählen Sie einen Vorschlag des Bots.
chat-empty-action = Erste Nachricht schreiben

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = Stopp
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = Antwort geschrieben von
