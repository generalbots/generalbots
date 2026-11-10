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
