# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots Desktop
sidebar-toggle = Seitenleiste umschalten
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminal
sidebar-collapsed-drive = Drive
sidebar-collapsed-settings = Einstellungen
sidebar-collapsed-account = Konto
setup-welcome-title = Willkommen bei General Bots
setup-welcome-subtitle = Richten Sie alles in wenigen schnellen Schritten ein
setup-welcome-quick-subtitle = Zwei schnelle Schritte zum Start
setup-step-llm = LLM-Anbieter
setup-step-profile = Benutzerprofil
setup-step-bot = Bot-Konfiguration
setup-step-training = Training
setup-step-ai-model = KI-Modell
setup-step-your-name = Ihr Name
setup-choose-provider-title = Wählen Sie Ihren LLM-Anbieter
setup-choose-provider-subtitle = Wählen Sie den KI-Modellanbieter für Ihren Bot
setup-provider-openai-desc = GPT-4- und GPT-3.5-Modelle
setup-provider-nvidia-desc = NVIDIA-NIM-Inferenz
setup-provider-anthropic-desc = Claude-3- und Claude-3.5-Modelle
setup-provider-local-desc = llama.cpp, Ollama usw.
setup-profile-title = Was beschreibt Sie am besten?
setup-profile-subtitle = Wählen Sie Ihren Profiltyp für personalisierte Standardwerte
setup-profile-developer = Entwickler
setup-profile-developer-desc = Bots erstellen und bereitstellen, BASIC-Skripte schreiben
setup-profile-business = Geschäftlich
setup-profile-business-desc = Workflows automatisieren, CRM verwalten
setup-profile-support = Support
setup-profile-support-desc = Kundenservice, Ticketverwaltung
setup-bot-title = Konfigurieren Sie Ihren Bot
setup-bot-subtitle = Geben Sie Ihrem Bot einen Namen, einen Zweck und optional eine Vorlage
setup-bot-name = Bot-Name
setup-bot-name-placeholder = Mein Assistent
setup-bot-purpose = Zweck des Bots
setup-bot-purpose-placeholder = Beschreiben Sie, was Ihr Bot tun soll...
setup-bot-template = Bot-Vorlage
setup-optional = (optional)
setup-template-scratch = — Von Grund auf beginnen —
setup-name-title = Wie sollen wir Sie nennen?
setup-name-subtitle = Dieser Name erscheint in Ihren Unterhaltungen
setup-name-label = Ihr Name
setup-name-placeholder = z. B. Maria
setup-training-title = Trainieren Sie Ihren Bot
setup-training-subtitle = Legen Sie .pdf-, .xls- und .doc-Dateien für das erste Training ab. Sie haben Zugriff auf Drive und können es später ändern.
setup-drop-text = Dateien hier ablegen oder klicken zum Auswählen
setup-drop-hint = .pdf, .xls, .doc — jeweils bis zu 10 MB
setup-select-files = Dateien auswählen
setup-file-status-pending = Ausstehend
setup-file-status-uploading = Wird hochgeladen
setup-file-status-done = Fertig
setup-file-status-error = Fehler
setup-saving = Wird gespeichert...
setup-complete-message = Einrichtung abgeschlossen! Willkommen bei General Bots.
setup-failed = Einrichtung fehlgeschlagen: {error}
setup-unknown-error = Unbekannter Fehler
setup-back = Zurück
setup-next = Weiter
setup-finish = Abschließen
setup-skip = Einrichtung überspringen
agent-all-apps = Alle Apps

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = Komponente
versions-version = Version
versions-status = Status
versions-action = Aktion
versions-up-to-date = Aktuell
versions-unknown = Unbekannt
versions-check-update = Auf Update prüfen
versions-checking = Wird geprüft...
versions-checked = Geprüft
versions-error = Fehler
versions-update-available = Update: { $version }
versions-could-not-load = Versionsinformationen konnten nicht geladen werden.
versions-ensure-backend = Stellen Sie sicher, dass der Backend-Server läuft.
versions-retry = Erneut versuchen

a11y-chat-conversation = Chat-Unterhaltung
a11y-chat-messages = Chat-Nachrichten
a11y-send-message = Nachricht senden
a11y-type-message = Geben Sie Ihre Nachricht ein
a11y-keyboard-shortcuts = Tastenkürzel
a11y-shortcut-focus-chat = Chat-Eingabe fokussieren
a11y-shortcut-show-help = Diese Hilfe anzeigen
a11y-shortcut-escape = Dialog schließen / Auswahl aufheben
a11y-shortcut-command-palette = Befehlspalette
a11y-shortcut-privacy = Datenschutzeinstellungen
a11y-shortcut-scroll-history = Chat-Verlauf scrollen
a11y-shortcut-navigate = Zwischen Elementen navigieren
a11y-shortcut-activate = Fokussiertes Element aktivieren
a11y-shortcut-delete-files = Ausgewählte Dateien löschen (Drive)
a11y-shortcut-select-all = Alle Dateien auswählen (Drive)
a11y-shortcut-clipboard = Dateien kopieren / ausschneiden / einfügen
a11y-shortcut-navigate-up = In Drive nach oben navigieren
a11y-shortcut-rename = Ausgewählte Datei umbenennen

sidebar-locked-conversations = Gesperrte Unterhaltungen
sidebar-no-conversations = Noch keine Unterhaltungen
sidebar-sign-in-conversations = Melden Sie sich an, um Ihre Unterhaltungen zu sehen
sidebar-history-unavailable = Verlauf nicht verfügbar
sidebar-back-to-conversations = ← Zurück zu den Unterhaltungen
sidebar-no-locked-conversations = Keine gesperrten Unterhaltungen
sidebar-locked-conversation = Gesperrte Unterhaltung
sidebar-sign-in = Anmelden
sidebar-add-pin = PIN hinzufügen

collab-collaborators = Mitwirkende ({ $count })
collab-no-one-else = Niemand sonst ist hier

update-available-title = Update verfügbar — v{ $version }
update-now = Jetzt aktualisieren
update-later = Später
update-skip-version = Diese Version überspringen
update-platform = Plattform: { $name }
update-asset = Asset: { $name }
update-check-failed = Update-Prüfung fehlgeschlagen: { $error }
update-view-release-notes = Versionshinweise anzeigen
update-release-notes-title = Versionshinweise — v{ $version }
update-no-release-notes = Keine Versionshinweise verfügbar.

modal-alert-title = Warnung
modal-input-title = Eingabe

external-save-to = In { $target } speichern

usage-title = Nutzung und Kosten
usage-last-days = Letzte { $days } Tage
usage-loading = Nutzung wird geladen…
usage-per-model = Pro Modell
usage-per-day = Pro Tag
usage-model = Modell
usage-tokens = Token
usage-messages = Nachrichten
usage-cost = Kosten
usage-day = Tag
usage-days-7 = 7 Tage
usage-days-30 = 30 Tage
usage-days-90 = 90 Tage

pin-close = Schließen
pin-length-hint = 4–8 Ziffern
pin-cancel = Abbrechen
pin-ok = OK

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = Eigentümer
role-editor = Bearbeiter
role-commenter = Kommentator
role-viewer = Betrachter

share-request-failed = Anfrage fehlgeschlagen ({ $status })
share-warning-title = Geteilte Links sind noch nicht erreichbar
share-warning-body = Diese Installation ist nur über localhost erreichbar, daher führt ein Link von hier nirgendwohin. Richten Sie zuerst eines davon ein: P2P unter Einstellungen → Netzwerk aktivieren oder geteilte Links auf Ihre eigene Domain verweisen.
share-open-shared-links = Geteilte Links öffnen
share-open-network = Netzwerk öffnen
share-invite-people = Personen einladen
share-anyone-with-link = Jeder mit dem Link
share-create-link = Link erstellen
share-transfer-ownership = Eigentum übertragen
share-transfer = Übertragen
share-new-owner-placeholder = E-Mail des neuen Eigentümers
share-owner-suffix = · Eigentümer
share-no-people = Noch keine Personen hinzugefügt.
share-no-links = Noch kein öffentlicher Link.
share-copied = Kopiert
share-revoke = Widerrufen
share-enter-email = Geben Sie eine E-Mail-Adresse ein
share-enter-new-owner = Geben Sie die E-Mail-Adresse des neuen Eigentümers ein
share-needs-base = Richten Sie P2P oder Ihre eigene Domain ein, bevor Sie einen geteilten Link erstellen.

activity-title = Aktivität
activity-request-failed = Anfrage fehlgeschlagen ({ $status })
activity-load-more = Mehr laden
activity-loading = Wird geladen…
activity-empty = Noch keine Aktivität
activity-action-create = hat erstellt
activity-action-edit = hat bearbeitet
activity-action-comment = hat kommentiert
activity-action-delete = hat gelöscht
activity-action-resolve = hat einen Kommentar gelöst zu
activity-action-reopen = hat einen Kommentar wieder geöffnet zu
activity-action-reaction = hat reagiert auf
activity-action-share = hat geteilt
activity-action-restore = hat wiederhergestellt
activity-action-transfer = hat übertragen

conflict-title = Konkurrierende Änderungen
conflict-dismiss = Verwerfen
conflict-edited-concurrently = gleichzeitig bearbeitet ({ $op })
conflict-keep-server = Serverversion behalten
conflict-keep-mine = Meine Änderung behalten
conflict-not-started = Konfliktüberwachung nicht gestartet
conflict-request-failed = Anfrage fehlgeschlagen ({ $status })

versions-title = Versionsverlauf
versions-loading = Wird geladen…
versions-empty = Noch keine Versionen — das erste Speichern erstellt eine.
versions-version-n = Version { $n }
versions-current-tag = (aktuell)
versions-name = Name…
versions-restore-failed = Wiederherstellung fehlgeschlagen: { $error }
versions-rename-failed = Umbenennen fehlgeschlagen: { $error }
versions-restore-dialog-title = Version wiederherstellen
versions-restore-confirm = Diese Version wiederherstellen? Es wird eine neue aktuelle Version erstellt — nichts geht verloren.
versions-name-dialog-title = Version benennen
versions-name-prompt = Benennen Sie diese Version (z. B. "v2 — freigegeben"):
versions-request-failed = Anfrage fehlgeschlagen ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = "{ $app }" andocken
snap-layout-top-left = Oben links
snap-layout-top = Obere Hälfte
snap-layout-top-right = Oben rechts
snap-layout-left = Linke Hälfte
snap-layout-full = Maximieren
snap-layout-right = Rechte Hälfte
snap-layout-bottom-left = Unten links
snap-layout-bottom = Untere Hälfte
snap-layout-bottom-right = Unten rechts
snap-layout-third-left = Linkes Drittel
snap-layout-two-thirds = Zwei Drittel
snap-layout-third-right = Rechtes Drittel

wstabs-close-tab = Tab schließen
wstabs-new-tab = Neuer Workspace-Tab
wstabs-default-name = Workspace { $n }
wstabs-default-main = Haupt
wstabs-rename-title = Workspace umbenennen
wstabs-name-prompt = Workspace-Name:

vdesk-default-name = Desktop { $n }
vdesk-title = Virtuelle Desktops
vdesk-new = Neuer Desktop
vdesk-rename-title = Desktop umbenennen
vdesk-name-prompt = Desktop-Name:
vdesk-restore-title = Sitzung wiederherstellen
vdesk-restore-body = Die { $count } Fenster Ihrer letzten Sitzung auf diesem Desktop erneut öffnen?
vdesk-restore-body-short = Ihre letzte Sitzung wiederherstellen ({ $count } Fenster)?

ctxmenu-search = DuckDuckGo durchsuchen
ctxmenu-search-prompt = DuckDuckGo durchsuchen:
ctxmenu-wikipedia = Wikipedia-Definition
ctxmenu-wikipedia-prompt = Wikipedia-Definition:

git-loading-status = Status wird geladen...
git-no-changes = Keine Änderungen
git-new-branch-prompt = Name des neuen Branches:
git-loading-diff = Diff wird geladen...
git-diff-failed = Diff konnte nicht geladen werden
git-commit-message-required = Bitte geben Sie eine Commit-Nachricht ein
git-pushing = Push läuft...
git-committed = Erfolgreich committet und gepusht
git-failed = Git-Vorgang fehlgeschlagen

browser-session-active = Browsersitzung { $id } aktiv.
browser-init-failed = Browser konnte nicht initialisiert werden
browser-init-first = Bitte initialisieren Sie zuerst den Browser
browser-capture = Aufnehmen
browser-no-session = Keine aktive Sitzung
browser-export-failed = Export fehlgeschlagen
browser-no-actions = Keine Aktionen aufgezeichnet

widget-load-failed = Widget konnte nicht geladen werden
widget-no-source = Keine Widget-Quelle konfiguriert
widget-open-windowed = Im Fenster öffnen
widget-pin-launcher = An Launcher anheften

dataexport-title = Datenexport und Sicherung
dataexport-desc = Exportieren Sie alle Ihre Daten, einschließlich Chat-Verlauf, Gedächtnis, Drive-Metadaten und Einstellungen, als JSON-Sicherungsdatei.
dataexport-export-all = Alle Daten exportieren
dataexport-import = Sicherung importieren
dataexport-exported = Sicherung erfolgreich exportiert
dataexport-too-large = Sicherung abgelehnt: Die Datei ist größer als 4 MB
dataexport-invalid = Ungültige Sicherungsdatei: { $error }
dataexport-nothing = Diese Sicherung enthält keine wiederherstellbaren Einstellungen
dataexport-unknown-date = unbekanntes Datum
dataexport-restore-confirm = Dadurch werden { $count } Einstellung(en) aus der Sicherung vom { $date } wiederhergestellt. Anmeldedaten und Sitzungstoken werden nie importiert. Fortfahren?
dataexport-restored = Sicherung wiederhergestellt ({ $count } Elemente).
dataexport-rejected = { $count } Eintrag/Einträge von der Zulassungsliste abgelehnt.
dataexport-reloading = Wird neu geladen...

comments-title = Kommentare
comments-request-failed = Anfrage fehlgeschlagen ({ $status })
comments-loading = Kommentare werden geladen…
comments-load-failed = Kommentare konnten nicht geladen werden: { $error }
comments-empty-open = Keine offenen Kommentare. Verwenden Sie @name, um jemanden zu erwähnen.
comments-empty-resolved = Keine gelösten Kommentare. Verwenden Sie @name, um jemanden zu erwähnen.
comments-resolved-chip = Gelöst
comments-resolve = Lösen
comments-reopen = Wieder öffnen
comments-resolved-announce = Kommentar gelöst
comments-reopened-announce = Kommentar wieder geöffnet
comments-resolve-failed = Lösen fehlgeschlagen: { $error }
comments-added = Kommentar hinzugefügt
comments-post = Senden
comments-post-failed = Kommentar fehlgeschlagen: { $error }
comments-reaction-failed = Reaktion fehlgeschlagen: { $error }
comments-delete-failed = Löschen fehlgeschlagen: { $error }
comments-reply-failed = Antworten fehlgeschlagen: { $error }
comments-delete-title = Kommentar löschen
comments-delete-confirm = Diesen Kommentar löschen?
comments-reply-title = Antworten
comments-reply-prompt = Auf Kommentar antworten:
comments-filter-open = Offen
comments-filter-all = Alle
comments-filter-resolved = Gelöst
comments-input-placeholder = Kommentar hinzufügen… @name verwenden, um zu erwähnen
comments-someone = jemand
comments-mention-announce = Sie wurden von { $author } in einem Kommentar erwähnt
