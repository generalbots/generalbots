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

# Assistant parity in chat
assistant-parity-block-allow = Erlauben
assistant-parity-block-allowed = Erlaubt
assistant-parity-block-api-command = API-Befehl
assistant-parity-block-attachment-read = Anhang gelesen
assistant-parity-block-chars = Zeichen
assistant-parity-block-command = Befehl
assistant-parity-block-decision-failed = Entscheidung konnte nicht gespeichert werden
assistant-parity-block-denied = Abgelehnt
assistant-parity-block-deny = Ablehnen
assistant-parity-block-detail = Detail
assistant-parity-block-drive-list = Drive-Auflistung
assistant-parity-block-drive-read = Drive-Datei gelesen
assistant-parity-block-drive-search = Drive-Suche
assistant-parity-block-elapsed = Verstrichen
assistant-parity-block-error = Fehler
assistant-parity-block-file = Datei
assistant-parity-block-kb-search = Wissenssuche
assistant-parity-block-notice = Hinweis
assistant-parity-block-path = Pfad
assistant-parity-block-permission = Berechtigung angefordert
assistant-parity-block-query = Abfrage
assistant-parity-block-read = Lesen
assistant-parity-block-reason = Grund
assistant-parity-block-result = Ergebnis
assistant-parity-block-retry = Erneut versuchen
assistant-parity-block-seconds = s
assistant-parity-block-size = Größe
assistant-parity-block-status-done = Fertig
assistant-parity-block-status-failed = Fehlgeschlagen
assistant-parity-block-status-needs-you = Benötigt Ihre Eingabe
assistant-parity-block-status-running = Läuft
assistant-parity-block-step = Schritt
assistant-parity-block-thinking = Denken
assistant-parity-block-tool = Werkzeug
assistant-parity-block-tool-run = Werkzeuglauf
assistant-parity-block-total = Gesamt
assistant-parity-block-truncated = gekürzt
assistant-parity-block-type = Typ
assistant-parity-block-yes = Ja
chat-parity-attach-title = Anhangsvorschau
chat-parity-attach-unavailable = Inhalt nicht verfügbar – zum Öffnen Herunterladen verwenden.
chat-parity-attach-download-failed = Anhang konnte nicht heruntergeladen werden.
chat-parity-model-tiers = Automatisch & Stufen
chat-parity-model-accounts = Anbieterkonten
chat-parity-model-local = Lokal & kostenlos
chat-drive-parity-attaching = Anhängen...
chat-drive-parity-cancel = Abbrechen
chat-drive-parity-delete = Löschen
chat-drive-parity-deleted = Gelöscht
chat-drive-parity-delete-failed = Datei konnte nicht gelöscht werden.
chat-drive-parity-delete-question = Diese Datei löschen?
chat-drive-parity-delete-title = Datei löschen
chat-drive-parity-deleting = Löschen...
chat-drive-parity-download = Herunterladen
chat-drive-parity-download-failed = Datei konnte nicht heruntergeladen werden.
chat-drive-parity-downloading = Herunterladen...
chat-drive-parity-more = Weitere Aktionen
chat-drive-parity-new-name = Neuer Name
chat-drive-parity-preview = Vorschau
chat-drive-parity-preview-failed = Vorschau konnte nicht angezeigt werden.
chat-drive-parity-previewing = Vorschau wird geöffnet...
chat-drive-parity-rename = Umbenennen
chat-drive-parity-renamed = Umbenannt
chat-drive-parity-rename-failed = Datei konnte nicht umbenannt werden.
chat-drive-parity-rename-title = Datei umbenennen
chat-drive-parity-renaming = Umbenennen...
chat-drive-parity-unavailable = Diese Aktion ist in dieser Installation nicht verfügbar.
chat-drive-parity-use-in-chat = In der Unterhaltung verwenden
chat-parity-agent-does = Der Agent erledigt es
chat-parity-back = Zurück
chat-parity-checked-now = Gerade geprüft
chat-parity-close = Schließen
chat-parity-coming-soon = Demnächst verfügbar
chat-parity-cost-contract = Gemäß Ihrem Vertrag
chat-parity-cost-free = Kostenlos
chat-parity-cost-free-limit = Kostenlos, mit Nutzungsgrenzen
chat-parity-cost-sub = Ihr Abonnement
chat-parity-cuda-no = Keine geeignete GPU erkannt
chat-parity-cuda-yes = Grafikkarte erkannt
chat-parity-depends-agent = Hängt vom Agenten ab
chat-parity-driver = Treiber
chat-parity-engine = Verwendetes Modell
chat-parity-feature-docs = Dokumente
chat-parity-feature-ocr = Text in Bildern
chat-parity-feature-photos = Fotos und Diagramme
chat-parity-feature-web = Websuche
chat-parity-features = Aktive Funktionen
chat-parity-gains = Ihre Vorteile
chat-parity-go-agent = Zum Anbieter des Agenten
chat-parity-go-ext = Zu einem externen Anbieter
chat-parity-go-net = Bleibt im Netzwerk Ihrer Organisation
chat-parity-go-server = Zum von Ihnen eingestellten Server
chat-parity-guide = Sie wissen nicht, welches Sie wählen sollen?
chat-parity-guide-sub = Beantworten Sie zwei Fragen.
chat-parity-in-use = In Verwendung
chat-parity-load-failed = Diese Informationen konnten nicht geladen werden.
chat-parity-loading = Lädt...
chat-parity-machine = Dieser Computer
chat-parity-machine-ok = Dieser Computer hat: %d, %v GB
chat-parity-machine-slow = Dieser Computer hat keine geeignete Grafikkarte: es wäre langsam
chat-parity-need-acp = Installieren Sie den Agenten und melden Sie sich an
chat-parity-need-free = Nichts
chat-parity-need-local = Eine Grafikkarte mit 6 GB oder mehr und ein 4–6 GB Download
chat-parity-need-own = Die Serveradresse und ein Schlüssel
chat-parity-no-read = Liest nicht
chat-parity-no-work = Funktioniert nicht
chat-parity-none-found = Keine gefunden
chat-parity-not-yet = Noch nicht
chat-parity-only-net = Nur mit einem Server in Ihrem Netzwerk
chat-parity-optional = Optional, Sie aktivieren es
chat-parity-pick = Wählen Sie eine Option
chat-parity-priv-ext = Ihre Gespräche werden an einen externen Host gesendet (%h).
chat-parity-priv-local = Ihre Gespräche bleiben auf diesem Computer.
chat-parity-priv-net = Ihre Gespräche werden an den Server Ihrer Organisation gesendet (%h).
chat-parity-privacy = Datenschutz
chat-parity-q1 = Darf das, was Sie schreiben, diesen Computer verlassen?
chat-parity-q1-help = Denken Sie an vertrauliche oder persönliche Informationen.
chat-parity-q1-no = Nein, muss hier bleiben
chat-parity-q1-no-desc = Nur lokale Verarbeitung ist akzeptabel.
chat-parity-q1-yes = Ja, darf verlassen
chat-parity-q1-yes-desc = Ein externer Dienst darf es verarbeiten.
chat-parity-q2-no = Nein, oder ich weiß es nicht
chat-parity-q2-no-desc = Kein Firmenserver oder Schlüssel vorhanden.
chat-parity-q2-open = Bietet Ihre Organisation einen KI-Server oder Schlüssel an?
chat-parity-q2-open-help = Im Zweifel fragen Sie die IT.
chat-parity-q2-secret = Erlaubt Ihre Organisation externe KI?
chat-parity-q2-secret-help = Manche Richtlinien verbieten externe Dienste.
chat-parity-q2-secret-no-desc = Externe KI ist nicht erlaubt.
chat-parity-q2-secret-yes-desc = Externe KI ist erlaubt.
chat-parity-q2-yes = Ja
chat-parity-q2-yes-desc = Ein Firmenserver oder Schlüssel ist vorhanden.
chat-parity-reads = Liest
chat-parity-ready = Bereit
chat-parity-rec-free = Beginnen Sie mit dem kostenlosen Modell: keine Einrichtung. Für Vertrauliches bevorzugen Sie diesen Computer.
chat-parity-rec-local = Verwenden Sie diesen Computer: nichts verlässt ihn und die Grafikkarte ist geeignet.
chat-parity-rec-local-slow = Dieser Computer hat keine geeignete Grafikkarte für ein lokales Modell. Nutzen Sie den Firmenserver im internen Netzwerk.
chat-parity-rec-local-tight = Ein lokales Modell funktioniert, kann mit dieser Grafikkarte aber langsam sein.
chat-parity-rec-own = Verwenden Sie den Server oder Schlüssel Ihrer Organisation: Vertrag bestimmt Kosten und Grenzen.
chat-parity-rec-own-net = Nutzen Sie den Server Ihrer Organisation im Netzwerk, um Daten privat zu halten.
chat-parity-restart = Neu beginnen
chat-parity-retry = Erneut versuchen
chat-parity-row-cost = Kosten
chat-parity-row-docs = Dokumente
chat-parity-row-needs = Sie benötigen
chat-parity-row-ocr = Text in Bildern
chat-parity-row-offline = Ohne Internet
chat-parity-row-photos = Fotos und Diagramme
chat-parity-row-privacy = Ihre Texte
chat-parity-row-web = Websuche
chat-parity-rule = Regel
chat-parity-show-me = Sehen, wie man es aktiviert
chat-parity-stay-here = Bleibt auf diesem Computer
chat-parity-step = Frage %n von 2
chat-parity-sub-acp = Ihr Agenten-Abonnement
chat-parity-sub-free = Einsatzbereit
chat-parity-sub-local = Privat und offline
chat-parity-sub-own = Ihrer Organisation
chat-parity-suggested = Empfohlen
chat-parity-system-memory = Arbeitsspeicher
chat-parity-verdict-bad-gpu = Keine geeignete Grafikkarte
chat-parity-verdict-bad-gpu-long = Es wurde keine NVIDIA-Grafikkarte mit CUDA gefunden. Lokale Modelle würden auf dem Prozessor laufen, viel langsamer: eine Antwort kann Minuten dauern. Um Ihre Texte im Netzwerk zu behalten, bevorzugen Sie den Server Ihrer Organisation.
chat-parity-verdict-capacity = Diese Karte führt Modelle aus, die bis zu %v Videospeicher benötigen.
chat-parity-verdict-good = Komfortabel
chat-parity-verdict-low-vram = Karte mit zu wenig Speicher (%v)
chat-parity-verdict-low-vram-long = Die Karte hat %v Videospeicher; das kleinste lokale Modell benötigt %m.
chat-parity-verdict-mid = Knapp, aber nutzbar
chat-parity-verdict-none = Dieser Computer konnte nicht geprüft werden
chat-parity-video-memory = Videospeicher
chat-parity-where = Wählen Sie das Modell
chat-parity-where-acp = Agent (ACP)
chat-parity-where-free = Kostenloses Modell
chat-parity-where-local = Auf diesem Computer
chat-parity-where-own = Eigener Server oder Schlüssel
chat-parity-where-sub = Vergleichen Sie die vier Möglichkeiten, den Assistenten zu nutzen. Klicken Sie auf eine, um Details zu sehen und sie zu aktivieren.
chat-parity-works = Funktioniert
chat-parity-workshop = Assistenten-Werkstatt
chat-parity-workshop-lead = Stellen Sie ein, wo und wie Ihr Assistent denkt.
chat-parity-workshop-title = Assistenten-Werkstatt
chat-parity-graphics-card = Grafikkarte
