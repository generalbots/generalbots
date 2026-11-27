# =============================================================================
# General Bots — de — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Zuhause
nav-chat = Chatten
nav-drive = Fahren
nav-tasks = Aufgaben
nav-mail = Post
nav-calendar = Kalender
nav-meet = Treffen
nav-paper = Papier
nav-video = Video
nav-research = Forschung
nav-analytics = Analytik
nav-settings = Einstellungen
nav-admin = Admin
nav-monitoring = Überwachung
nav-sources = Quellen
nav-tools = Werkzeuge
nav-attendant = Begleiter
nav-learn = Lernen
nav-crm = CRM
nav-billing = Abrechnung
nav-products = Produkte
nav-tickets = Tickets
nav-docs = Dokumente
nav-sheet = Blätter
nav-slides = Folien
nav-social = Sozial
nav-all-apps = Alle Anwendungen
nav-people = Menschen
nav-editor = Herausgeber
nav-dashboards = Dashboards
nav-security = Sicherheit
nav-designer = Designer
nav-project = Projekt
nav-canvas = Leinwand
nav-goals = Ziele
nav-player = Spieler
nav-workspace = Arbeitsbereich

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Armaturenbrett
dashboard-welcome = Willkommen zurück, { $name }!
dashboard-quick-actions = Schnelle Aktionen
dashboard-recent-activity = Letzte Aktivität
dashboard-no-activity = Noch keine aktuelle Aktivität. Beginnen Sie mit der Erkundung!
dashboard-analytics = Analytik

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Chat starten
quick-upload-files = Dateien hochladen
quick-new-task = Neue Aufgabe
quick-compose-email = E-Mail verfassen
quick-start-meeting = Treffen starten
quick-new-event = Neue Veranstaltung

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Chatten
app-chat-desc = KI-gestützte Gespräche. Stellen Sie Fragen, holen Sie sich Hilfe und automatisieren Sie Aufgaben.
app-drive-name = Fahren
app-drive-desc = Cloud-Speicher für alle Ihre Dateien. Hochladen, organisieren und teilen.
app-tasks-name = Aufgaben
app-tasks-desc = Bleiben Sie mit To-Do-Listen, Prioritäten und Fälligkeitsterminen organisiert.
app-mail-name = Post
app-mail-desc = E-Mail-Client mit KI-gestütztem Schreiben und intelligenter Organisation.
app-calendar-name = Kalender
app-calendar-desc = Planen Sie Besprechungen und Veranstaltungen und verwalten Sie Ihre Zeit effektiv.
app-meet-name = Treffen
app-meet-desc = Videokonferenzen mit Bildschirmfreigabe und Live-Transkription.
app-paper-name = Papier
app-paper-desc = Schreiben Sie Dokumente mit KI-Unterstützung. Notizen, Berichte und mehr.
app-research-name = Forschung
app-research-desc = KI-gestützte Suche und Entdeckung in allen Ihren Quellen.
app-analytics-name = Analytik
app-analytics-desc = Dashboards und Berichte zur Verfolgung der Nutzung und Erkenntnisse.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = Allgemeine Bots-Suite
suite-tagline = Ihr KI-gestützter Produktivitätsarbeitsbereich. Chatten, zusammenarbeiten und erstellen.
suite-new-intent = Neue Absicht

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = KI-Entwickler
ai-developing = Entwicklung: { $project }
ai-quick-actions = Schnelle Aktionen
ai-add-field = Feld hinzufügen
ai-change-color = Farbe ändern
ai-add-validation = Validierung hinzufügen
ai-export-data = Daten exportieren
ai-placeholder = Geben Sie Ihre Änderungen ein...
ai-thinking = KI denkt...
ai-status-online = Online
ai-status-offline = Offline

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Suchen...
search-no-results = Keine Ergebnisse gefunden
search-results = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = Suche...
search-advanced = Erweiterte Suche
search-filters = Filter
search-clear-filters = Filter löschen

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = Keine Daten verfügbar
table-loading = Daten werden geladen...
table-actions = Aktionen
table-select-all = Wählen Sie „Alle“ aus
table-deselect-all = Alle abwählen
table-export = Exportieren
table-import = Importieren

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Erforderlich
form-optional = Optional
form-submit = Senden
form-reset = Zurücksetzen
form-clear = Klar
form-uploading = Hochladen...
form-processing = Verarbeitung...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Aktion bestätigen
modal-confirm-message = Sind Sie sicher, dass Sie fortfahren möchten?
modal-delete-title = Bestätigung löschen
modal-delete-message = Diese Aktion kann nicht rückgängig gemacht werden. Bist du sicher?

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = In die Zwischenablage kopieren
tooltip-copied = Kopiert!
tooltip-expand = Erweitern
tooltip-collapse = Zusammenbruch
tooltip-refresh = Aktualisieren
tooltip-download = Herunterladen
tooltip-upload = Hochladen
tooltip-print = Drucken
tooltip-fullscreen = Vollbild
tooltip-exit-fullscreen = Beenden Sie den Vollbildmodus

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Benutzername
label-email = E-Mail
label-display-name = Anzeigename
label-password = Passwort
label-role = Rolle
label-group-name = Gruppenname
label-hostname = Hostname
label-record-type = Datensatztyp
label-target = Ziel
label-your-name = Dein Name

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = Registrieren

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Benachrichtigungen
notifications-clear = Alles löschen
notifications-empty = Keine Benachrichtigungen

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Bereit
window-loading = App wird geladen…
window-minimize = Minimieren
window-maximize = Maximieren
window-fullscreen = Vollbild
window-exit-fullscreen = Vollbild beenden
window-close = Schließen
app-loading = Anwendung wird geladen…
start-menu-search-placeholder = Tippen, um eine App zu finden…
start-menu-recent = Zuletzt verwendet
start-menu-no-apps = Keine Apps gefunden
start-menu-user = Benutzer
category-ai = KI
category-business = Business
category-office = Office
category-dev = Entwicklung
category-system = System
category-tools = Werkzeuge
sidebar-apps = Apps
sidebar-apps-toggle = Apps ein-/ausblenden
sidebar-apps-toggle-aria = App-Bereich umschalten
sidebar-apps-filter = Apps filtern…
sidebar-apps-filter-aria = Apps filtern
sidebar-preview = Vorschaumodus
sidebar-preview-hint = Apps anzeigen, die sich noch in der Vorschau befinden
sidebar-preview-toggle = Vorschaumodus umschalten
sidebar-app-preview-title = {app} (Vorschau)
sidebar-apps-count = {count} Apps
sidebar-apps-count-preview = {count} (+ {preview} in Vorschau)
spotlight-placeholder = Apps, Dateien suchen… oder ein Ziel für den Concierge eingeben
spotlight-hint = ↑↓ navigieren · Enter ausführen · Strg+Leertaste oder Esc schließen
spotlight-empty = Keine Ergebnisse
spotlight-goal = Ziel ausführen: „{goal}“
spotlight-goal-subtitle = Concierge · planen und appübergreifend ausführen
palette-placeholder = Apps und Befehle suchen. Oder @ <Befehl> eingeben, um an den Assistenten zu senden...
palette-empty = Keine passenden Apps oder Befehle
palette-kind-app = App
palette-hint = ↑↓ navigieren · Enter ausführen · Esc schließen · Strg+K umschalten
palette-open-app = {app} öffnen
palette-value-prompt = {name}: {summary} — Wert für {key}:
palette-execute = {method} {path} ausführen und das Ergebnis zusammenfassen.
palette-use-command = Verwende den Befehl {command}, um {summary}.
toast-dismiss = Verwerfen
settings-language-display = Anzeigesprache
settings-language-display-desc = Wählen Sie die Sprache der Anwendungsoberfläche
settings-language-regional = Regionale Einstellungen
settings-language-regional-desc = Datums-, Zeit- und Zahlenformate konfigurieren
settings-language-date-format = Datumsformat
settings-language-time-format = Zeitformat
settings-language-timezone = Zeitzone
settings-language-first-day = Erster Wochentag
settings-language-time-12h = 12 Stunden (1:30 PM)
settings-language-time-24h = 24 Stunden (13:30)
settings-language-sunday = Sonntag
settings-language-monday = Montag
settings-email-notifications-desc = Werden vom Benachrichtigungsabsender des Arbeitsbereichs gesendet, niemals von einem verbundenen Gmail-/Outlook-Postfach.
settings-push-enable = Push-Benachrichtigungen aktivieren
settings-push-enable-desc = Browser-Push-Benachrichtigungen erhalten
settings-push-sound = Ton
settings-push-sound-desc = Ton für Benachrichtigungen abspielen
settings-desktop-notifications = Desktop-Benachrichtigungen
settings-desktop-notifications-desc = Benachrichtigungen auf dem Desktop anzeigen
settings-badge-count = Abzeichen-Zähler
settings-badge-count-desc = Abzeichen für ungelesene Elemente anzeigen
settings-save-preferences = Einstellungen speichern
settings-notifications-saved = Benachrichtigungseinstellungen gespeichert
settings-notification-recipient = Zustellpostfach
settings-notification-recipient-desc = Leer lassen, um Ihr primäres Mail-Konto oder Ihre Profil-E-Mail zu verwenden.
language-search-placeholder = Sprachen suchen…
settings-profile-title = Profileinstellungen
settings-security-title = Sicherheitseinstellungen

# semantic i18n key (#2542)
apps-menu-placeholder-search = Apps suchen...
apps-menu-aria-search = Apps suchen
apps-menu-aria-close = App-Menü schließen

# --- #2921 missing keys added to every locale catalogue ---
palette-kind-tour = Rundgang
palette-show-tour = Rundgang anzeigen
palette-show-tour-hint = Produktrundgang anzeigen
sidebar-app-hide = { $app } aus dieser Leiste ausblenden
sidebar-apps-count-off = { $count } ausgeblendet
sidebar-apps-empty = Keine Apps zum Anzeigen
sidebar-apps-manage = Apps verwalten
sidebar-apps-manage-all = Alle anzeigen
sidebar-apps-manage-close = Schließen
sidebar-apps-manage-empty = Keine Apps verfügbar
sidebar-apps-manage-summary = { $on } von { $total } angezeigt
sidebar-apps-none = Keine Apps
toast-open = Öffnen

# Assistant shell — knowledge-base source blocks
trecho = Auszug
trechos = Auszüge

# Assistant/Drive shell keys integrated from the gbgov edition
arquivo = file
arquivos = files
assistant-agent-backend-off = Agent backend is off
assistant-requires-agent-mode = Erfordert den Agentenmodus
assistant-agent-mode-disabled = Der Agentenmodus muss installiert sein, um Abonnement-Agenten zu verwenden.
assistant-agent-locked-hint = Agent is locked for this model
assistant-agent-locked-tip = Choose another model to use the agent
assistant-regenerate-older = Regenerating an older answer starts a new turn
caracteres = characters
corrigido = fixed
de = of
drive-actions = Actions
drive-folders = Folders
drive-usage-hint = Storage used
drive-used = Used
item = item
itens = items
linha = line
linhas = lines
resultado = result
resultados = results
shell-close = Close
shell-conversations = Conversations
shell-machine = Machine

# Assistant shell — literal keys
assistant-vm-wants = The agent wants to perform an action
assistant-vm-wants-command = The agent wants to run a command on the machine
drive-label = Drive

# Chat / assistant parity — VM consent and subscription-agent UI keys
chat-vm-consent-assistant = Assistent
chat-vm-consent-subject = Agent
chat-vm-consent-wants = Der Agent möchte eine Aktion ausführen
chat-vm-consent-wants-command = Der Agent möchte einen Befehl ausführen
chat-vm-consent-kicker = Benötigt Ihre Berechtigung
chat-vm-consent-command-label = Befehl
chat-vm-consent-where = Wo
chat-vm-consent-where-vm = Virtuelle Maschine
chat-vm-consent-where-drive = Ihr Drive
chat-vm-consent-destination = Ziel
chat-vm-consent-language = Sprache
chat-vm-consent-packages = Installiert aus dem Internet
chat-vm-consent-from = Quelle
chat-vm-consent-background = Läuft im Hintergrund
chat-vm-consent-repair = Korrigierte Version des Programms
chat-vm-consent-protected = Diese Anfrage betrifft die Zugangsdaten des Agenten; sie darf nur einmal erlaubt werden.
chat-vm-consent-all-note = Alles erlauben gilt für alle Aktionen dieses Agenten in dieser Unterhaltung, nur in der virtuellen Maschine.
chat-vm-consent-outside-note = Diese Aktion verlässt die virtuelle Maschine: Alles erlauben gilt nicht dafür.
chat-vm-consent-allow-once = Einmal erlauben
chat-vm-consent-allow-conversation = In dieser Unterhaltung erlauben
chat-vm-consent-allow-all = Alles erlauben
chat-vm-consent-deny = Verweigern
chat-vm-consent-outcome-all = Volle Berechtigung in dieser Unterhaltung
chat-vm-consent-outcome-conversation = In dieser Unterhaltung erlaubt
chat-vm-consent-outcome-once = Einmal erlaubt
chat-vm-consent-outcome-denied = Von Ihnen verweigert: nichts wurde ausgeführt
chat-vm-consent-outcome-failed = Ihre Entscheidung konnte nicht gespeichert werden
chat-vm-consent-forbidden = Diese Entscheidung kann hier nicht gespeichert werden. Die Anfrage bleibt offen.
chat-vm-consent-announce = Berechtigungsanfrage geöffnet. Drücken Sie Alt+P zum Antworten.
chat-vm-consent-still-open = Eine Berechtigungsanfrage ist noch offen. Wählen Sie eine Option; Verweigern führt nichts aus.
chat-vm-consent-waiting = Warte auf Ihre Berechtigung
chat-vm-all-chip = Volle Berechtigung
chat-vm-all-scope = nur in der virtuellen Maschine
chat-vm-all-revoke = Widerrufen
chat-vm-all-revoked = Volle Berechtigung widerrufen
chat-vm-send-head = Bestätigung
chat-vm-send-text = Das Senden bricht die offene Berechtigungsanfrage ab. Trotzdem senden?
chat-vm-send-back = Zurück zur Anfrage
chat-vm-send-go = Senden
chat-vm-engine-local-short = Lokal
chat-vm-engine-local-group = Lokale Modelle
chat-vm-engine-local-detail = Auf diesem Computer
chat-vm-engine-free-short = Kostenlos
chat-vm-engine-free-group = Kostenloses Modell
chat-vm-engine-free-detail = Kostenloses Modell
chat-vm-engine-own-short = Eigener Schlüssel
chat-vm-engine-own-group = Eigene Schlüssel
chat-vm-engine-own-detail = Eigener Server oder Schlüssel
chat-vm-engine-sub-short = Abonnement
chat-vm-engine-sub-group = Abonnements
chat-vm-engine-sub-detail = Abonnement-Agent
chat-vm-agent-locked-tip = Der ausgewählte Agent läuft in der virtuellen Maschine; wählen Sie ein anderes Modell, um ihn auszuschalten
chat-vm-agent-locked-note = läuft in der virtuellen Maschine, daher bleibt der Agent-Schalter aktiv, solange er ausgewählt ist. Wählen Sie ein anderes Modell, um ihn auszuschalten.
assistant-no-update = Kein Update verfügbar
