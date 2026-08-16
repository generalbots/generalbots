# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = Verwaltung
admin-dashboard = Admin-Dashboard
admin-overview = Übersicht
admin-welcome = Willkommen im Admin-Panel

admin-nav-dashboard = Armaturenbrett
admin-nav-users = Benutzer
admin-nav-bots = Bots
admin-nav-tenants = Mieter
admin-nav-settings = Einstellungen
admin-nav-logs = Protokolle
admin-nav-analytics = Analytik
admin-nav-security = Sicherheit
admin-nav-integrations = Integrationen
admin-nav-billing = Abrechnung
admin-nav-support = Unterstützung
admin-nav-groups = Gruppen
admin-nav-dns = DNS
admin-nav-system = System

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = Schnelle Aktionen
admin-create-user = Benutzer erstellen
admin-create-group = Gruppe erstellen
admin-register-dns = Registrieren Sie DNS
admin-recent-activity = Letzte Aktivität
admin-system-health = Systemgesundheit

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = Benutzerverwaltung
admin-users-list = Benutzerliste
admin-users-add = Benutzer hinzufügen
admin-users-edit = Benutzer bearbeiten
admin-users-delete = Benutzer löschen
admin-users-search = Benutzer suchen...
admin-users-filter = Benutzer filtern
admin-users-export = Benutzer exportieren
admin-users-import = Benutzer importieren
admin-users-total = Gesamtzahl der Benutzer
admin-users-active = Aktive Benutzer
admin-users-inactive = Inaktive Benutzer
admin-users-suspended = Gesperrte Benutzer
admin-users-pending = Ausstehende Überprüfung
admin-users-last-login = Letzte Anmeldung
admin-users-created = Erstellt
admin-users-role = Rolle
admin-users-status = Status
admin-users-actions = Aktionen
admin-users-no-users = Keine Benutzer gefunden
admin-users-confirm-delete = Sind Sie sicher, dass Sie diesen Benutzer löschen möchten?
admin-users-deleted = Benutzer erfolgreich gelöscht
admin-users-saved = Benutzer erfolgreich gespeichert
admin-users-invite = Benutzer einladen
admin-users-invite-sent = Einladung erfolgreich gesendet
admin-users-bulk-actions = Massenaktionen
admin-users-select-all = Wählen Sie „Alle“ aus
admin-users-deselect-all = Alle abwählen

# User Details
admin-user-details = Benutzerdetails
admin-user-profile = Profil
admin-user-email = E-Mail
admin-user-name = Name
admin-user-phone = Telefon
admin-user-avatar = Avatar
admin-user-timezone = Zeitzone
admin-user-language = Sprache
admin-user-role-admin = Administrator
admin-user-role-manager = Manager
admin-user-role-user = Benutzer
admin-user-role-viewer = Zuschauer
admin-user-status-active = Aktiv
admin-user-status-inactive = Inaktiv
admin-user-status-suspended = Suspendiert
admin-user-status-pending = Ausstehend
admin-user-permissions = Berechtigungen
admin-user-activity = Aktivitätsprotokoll
admin-user-sessions = Aktive Sitzungen
admin-user-terminate-session = Sitzung beenden
admin-user-terminate-all = Beenden Sie alle Sitzungen
admin-user-reset-password = Passwort zurücksetzen
admin-user-force-logout = Abmelden erzwingen
admin-user-enable-2fa = Aktivieren Sie 2FA
admin-user-disable-2fa = Deaktivieren Sie 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = Gruppenleitung
admin-groups-subtitle = Verwalten Sie Gruppen, Mitglieder und Berechtigungen
admin-groups-list = Gruppenliste
admin-groups-add = Gruppe hinzufügen
admin-groups-create = Gruppe erstellen
admin-groups-edit = Gruppe bearbeiten
admin-groups-delete = Gruppe löschen
admin-groups-search = Gruppen durchsuchen...
admin-groups-filter = Gruppen filtern
admin-groups-total = Gesamtzahl der Gruppen
admin-groups-active = Aktive Gruppen
admin-groups-no-groups = Keine Gruppen gefunden
admin-groups-confirm-delete = Sind Sie sicher, dass Sie diese Gruppe löschen möchten?
admin-groups-deleted = Gruppe erfolgreich gelöscht
admin-groups-saved = Gruppe erfolgreich gespeichert
admin-groups-created = Gruppe erfolgreich erstellt
admin-groups-loading = Gruppen werden geladen...

# Group Details
admin-group-details = Gruppendetails
admin-group-name = Gruppenname
admin-group-description = Beschreibung
admin-group-visibility = Sichtbarkeit
admin-group-visibility-public = Öffentlich
admin-group-visibility-private = Privat
admin-group-visibility-hidden = Versteckt
admin-group-join-policy = Beitrittsrichtlinie
admin-group-join-invite = Nur auf Einladung
admin-group-join-request = Beitrittsanfrage
admin-group-join-open = Offen
admin-group-members = Mitglieder
admin-group-member-count = { $count ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = Mitglied hinzufügen
admin-group-remove-member = Mitglied entfernen
admin-group-permissions = Berechtigungen
admin-group-settings = Einstellungen
admin-group-analytics = Analytik
admin-group-overview = Übersicht

# Group View Modes
admin-groups-view-grid = Rasteransicht
admin-groups-view-list = Listenansicht
admin-groups-all-visibility = Alle Sichtbarkeit

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = DNS-Verwaltung
admin-dns-subtitle = Registrieren und verwalten Sie DNS-Hostnamen für Ihre Bots
admin-dns-register = Registrieren Sie den Hostnamen
admin-dns-registered = Registrierte Hostnamen
admin-dns-search = Hostnamen suchen...
admin-dns-refresh = Aktualisieren
admin-dns-loading = DNS-Einträge werden geladen...
admin-dns-no-records = Keine DNS-Einträge gefunden
admin-dns-confirm-delete = Möchten Sie diesen Hostnamen wirklich entfernen?
admin-dns-deleted = Hostname erfolgreich entfernt
admin-dns-saved = DNS-Eintrag erfolgreich gespeichert
admin-dns-created = Hostname erfolgreich registriert

# DNS Form Fields
admin-dns-hostname = Hostname
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = Geben Sie den vollständigen Domainnamen ein, den Sie registrieren möchten
admin-dns-record-type = Datensatztyp
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (Sekunden)
admin-dns-ttl-5min = 5 Minuten (300)
admin-dns-ttl-1hour = 1 Stunde (3600)
admin-dns-ttl-1day = 1 Tag (86400)
admin-dns-target = Ziel/IP-Adresse
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = Geben Sie die IPv4-Adresse ein, auf die verwiesen werden soll
admin-dns-target-help-aaaa = Geben Sie die IPv6-Adresse ein, auf die verwiesen werden soll
admin-dns-target-help-cname = Geben Sie den Zieldomänennamen ein
admin-dns-auto-ssl = SSL-Zertifikat automatisch bereitstellen

# DNS Table Headers
admin-dns-col-hostname = Hostname
admin-dns-col-type = Typ
admin-dns-col-target = Ziel
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Status
admin-dns-col-actions = Aktionen

# DNS Status
admin-dns-status-active = Aktiv
admin-dns-status-pending = Ausstehend
admin-dns-status-error = Fehler
admin-dns-ssl-enabled = SSL aktiviert
admin-dns-ssl-disabled = Kein SSL
admin-dns-ssl-pending = SSL ausstehend

# DNS Info Cards
admin-dns-help-title = Hilfe zur DNS-Konfiguration
admin-dns-help-a-record = Ein Rekord
admin-dns-help-a-record-desc = Ordnet einen Domänennamen einer IPv4-Adresse zu. Verwenden Sie dies, um Ihren Hostnamen direkt auf eine Server-IP zu verweisen.
admin-dns-help-aaaa-record = AAAA-Rekord
admin-dns-help-aaaa-record-desc = Ordnet einen Domänennamen einer IPv6-Adresse zu. Ähnlich wie ein A-Eintrag, jedoch für IPv6-Konnektivität.
admin-dns-help-cname-record = CNAME-Eintrag
admin-dns-help-cname-record-desc = Erstellt einen Alias von einer Domäne zu einer anderen. Nützlich, um Subdomains auf Ihre Hauptdomain zu verweisen.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Stellt automatisch Let's Encrypt-Zertifikate für sichere HTTPS-Verbindungen bereit.

# DNS Edit/Remove Modals
admin-dns-edit-title = Bearbeiten Sie den DNS-Eintrag
admin-dns-remove-title = Hostnamen entfernen
admin-dns-remove-warning = Dadurch werden der DNS-Eintrag und alle zugehörigen SSL-Zertifikate gelöscht. Der Hostname wird nicht mehr aufgelöst.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = Bot-Management
admin-bots-list = Bot-Liste
admin-bots-add = Bot hinzufügen
admin-bots-edit = Bot bearbeiten
admin-bots-delete = Bot löschen
admin-bots-search = Bots suchen...
admin-bots-filter = Bots filtern
admin-bots-total = Insgesamt Bots
admin-bots-active = Aktive Bots
admin-bots-inactive = Inaktive Bots
admin-bots-draft = Draft-Bots
admin-bots-published = Veröffentlichte Bots
admin-bots-no-bots = Keine Bots gefunden
admin-bots-confirm-delete = Sind Sie sicher, dass Sie diesen Bot löschen möchten?
admin-bots-deleted = Bot erfolgreich gelöscht
admin-bots-saved = Bot erfolgreich gespeichert
admin-bots-duplicate = Doppelter Bot
admin-bots-export = Bot exportieren
admin-bots-import = Bot importieren
admin-bots-publish = Veröffentlichen
admin-bots-unpublish = Veröffentlichung aufheben
admin-bots-test = Testbot
admin-bots-logs = Bot-Protokolle
admin-bots-analytics = Bot-Analyse
admin-bots-conversations = Gespräche
admin-bots-templates = Vorlagen
admin-bots-dialogs = Dialoge
admin-bots-knowledge-base = Wissensdatenbank

# Bot Details
admin-bot-details = Bot-Details
admin-bot-name = Bot-Name
admin-bot-description = Beschreibung
admin-bot-avatar = Bot-Avatar
admin-bot-language = Sprache
admin-bot-timezone = Zeitzone
admin-bot-greeting = Begrüßungsnachricht
admin-bot-fallback = Fallback-Nachricht
admin-bot-channels = Kanäle
admin-bot-channel-web = Web-Chat
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = Telegramm
admin-bot-channel-slack = Locker
admin-bot-channel-teams = Microsoft-Teams
admin-bot-channel-email = E-Mail
admin-bot-model = KI-Modell
admin-bot-temperature = Temperatur
admin-bot-max-tokens = Maximale Token
admin-bot-system-prompt = Systemaufforderung

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = Mietermanagement
admin-tenants-list = Mieterliste
admin-tenants-add = Mieter hinzufügen
admin-tenants-edit = Mieter bearbeiten
admin-tenants-delete = Mieter löschen
admin-tenants-search = Mieter suchen...
admin-tenants-total = Gesamtzahl der Mieter
admin-tenants-active = Aktive Mieter
admin-tenants-suspended = Suspendierte Mieter
admin-tenants-trial = Probemieter
admin-tenants-no-tenants = Keine Mieter gefunden
admin-tenants-confirm-delete = Sind Sie sicher, dass Sie diesen Mandanten löschen möchten?
admin-tenants-deleted = Mieter erfolgreich gelöscht
admin-tenants-saved = Mieter erfolgreich gespeichert

# Tenant Details
admin-tenant-details = Angaben zum Mieter
admin-tenant-name = Name des Mieters
admin-tenant-domain = Domäne
admin-tenant-plan = Planen
admin-tenant-plan-free = Kostenlos
admin-tenant-plan-starter = Anlasser
admin-tenant-plan-professional = Professionell
admin-tenant-plan-enterprise = Unternehmen
admin-tenant-users = Benutzer
admin-tenant-bots = Bots
admin-tenant-storage = Verwendeter Speicher
admin-tenant-api-calls = API-Aufrufe
admin-tenant-limits = Nutzungsbeschränkungen
admin-tenant-billing = Rechnungsinformationen

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = Systemeinstellungen
admin-settings-general = Allgemeine Einstellungen
admin-settings-security = Sicherheitseinstellungen
admin-settings-email = E-Mail-Einstellungen
admin-settings-storage = Speichereinstellungen
admin-settings-integrations = Integrationen
admin-settings-api = API-Einstellungen
admin-settings-appearance = Aussehen
admin-settings-localization = Lokalisierung
admin-settings-notifications = Benachrichtigungen
admin-settings-backup = Sichern und Wiederherstellen
admin-settings-maintenance = Wartungsmodus
admin-settings-saved = Einstellungen erfolgreich gespeichert
admin-settings-reset = Auf Standardeinstellungen zurücksetzen
admin-settings-confirm-reset = Sind Sie sicher, dass Sie alle Einstellungen auf die Standardeinstellungen zurücksetzen möchten?

# General Settings
admin-settings-site-name = Site-Name
admin-settings-site-url = Site-URL
admin-settings-admin-email = Admin-E-Mail
admin-settings-support-email = Support-E-Mail
admin-settings-default-language = Standardsprache
admin-settings-default-timezone = Standardzeitzone
admin-settings-date-format = Datumsformat
admin-settings-time-format = Zeitformat
admin-settings-currency = Währung

# Email Settings
admin-settings-smtp-host = SMTP-Host
admin-settings-smtp-port = SMTP-Port
admin-settings-smtp-user = SMTP-Benutzername
admin-settings-smtp-password = SMTP-Passwort
admin-settings-smtp-encryption = Verschlüsselung
admin-settings-smtp-from-name = Von Name
admin-settings-smtp-from-email = Von E-Mail
admin-settings-smtp-test = Test-E-Mail senden
admin-settings-smtp-test-success = Test-E-Mail erfolgreich gesendet
admin-settings-smtp-test-failed = Test-E-Mail konnte nicht gesendet werden

# Storage Settings
admin-settings-storage-provider = Speicheranbieter
admin-settings-storage-local = Lokaler Speicher
admin-settings-storage-s3 = Amazon S3
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Google Cloud-Speicher
admin-settings-storage-azure = Azure Blob Storage
admin-settings-storage-bucket = Bucket-Name
admin-settings-storage-region = Region
admin-settings-storage-access-key = Zugriffsschlüssel
admin-settings-storage-secret-key = Geheimer Schlüssel
admin-settings-storage-endpoint = Endpunkt-URL

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = Systemprotokolle
admin-logs-search = Protokolle durchsuchen...
admin-logs-filter-level = Nach Ebene filtern
admin-logs-filter-source = Nach Quelle filtern
admin-logs-filter-date = Nach Datum filtern
admin-logs-level-all = Alle Ebenen
admin-logs-level-debug = Debuggen
admin-logs-level-info = Infos
admin-logs-level-warning = Warnung
admin-logs-level-error = Fehler
admin-logs-level-critical = Kritisch
admin-logs-export = Protokolle exportieren
admin-logs-clear = Protokolle löschen
admin-logs-confirm-clear = Sind Sie sicher, dass Sie alle Protokolle löschen möchten?
admin-logs-cleared = Protokolle erfolgreich gelöscht
admin-logs-no-logs = Keine Protokolle gefunden
admin-logs-refresh = Aktualisieren
admin-logs-auto-refresh = Automatische Aktualisierung
admin-logs-timestamp = Zeitstempel
admin-logs-level = Ebene
admin-logs-source = Quelle
admin-logs-message = Nachricht
admin-logs-details = Einzelheiten

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = Analytik
admin-analytics-overview = Übersicht
admin-analytics-users = Benutzeranalyse
admin-analytics-bots = Bot-Analyse
admin-analytics-conversations = Konversationsanalyse
admin-analytics-performance = Leistung
admin-analytics-period = Zeitraum
admin-analytics-period-today = Heute
admin-analytics-period-week = Diese Woche
admin-analytics-period-month = Diesen Monat
admin-analytics-period-quarter = Dieses Quartal
admin-analytics-period-year = Dieses Jahr
admin-analytics-period-custom = Benutzerdefinierter Bereich
admin-analytics-export = Bericht exportieren
admin-analytics-total-users = Gesamtzahl der Benutzer
admin-analytics-new-users = Neue Benutzer
admin-analytics-active-users = Aktive Benutzer
admin-analytics-total-bots = Insgesamt Bots
admin-analytics-active-bots = Aktive Bots
admin-analytics-total-conversations = Gesamtzahl der Gespräche
admin-analytics-avg-response-time = Durchschnittliche Reaktionszeit
admin-analytics-satisfaction-rate = Zufriedenheitsrate
admin-analytics-resolution-rate = Auflösungsrate

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = Sicherheit
admin-security-overview = Sicherheitsübersicht
admin-security-audit-log = Audit-Protokoll
admin-security-login-attempts = Anmeldeversuche
admin-security-blocked-ips = Blockierte IPs
admin-security-api-keys = API-Schlüssel
admin-security-webhooks = Webhooks
admin-security-cors = CORS-Einstellungen
admin-security-rate-limiting = Ratenbegrenzung
admin-security-encryption = Verschlüsselung
admin-security-2fa = Zwei-Faktor-Authentifizierung
admin-security-sso = Single Sign-On
admin-security-password-policy = Passwortrichtlinie

# API Keys
admin-api-keys-title = API-Schlüssel
admin-api-keys-add = API-Schlüssel erstellen
admin-api-keys-name = Schlüsselname
admin-api-keys-key = API-Schlüssel
admin-api-keys-secret = Geheimer Schlüssel
admin-api-keys-created = Erstellt
admin-api-keys-last-used = Zuletzt verwendet
admin-api-keys-expires = Läuft ab
admin-api-keys-never = Niemals
admin-api-keys-revoke = Widerrufen
admin-api-keys-confirm-revoke = Sind Sie sicher, dass Sie diesen API-Schlüssel widerrufen möchten?
admin-api-keys-revoked = API-Schlüssel erfolgreich widerrufen
admin-api-keys-created-success = API-Schlüssel erfolgreich erstellt
admin-api-keys-copy = In die Zwischenablage kopieren
admin-api-keys-copied = Kopiert!
admin-api-keys-warning = Stellen Sie sicher, dass Sie jetzt Ihren API-Schlüssel kopieren. Du wirst es nie wieder sehen können!

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = Abrechnung
admin-billing-overview = Abrechnungsübersicht
admin-billing-current-plan = Aktueller Plan
admin-billing-usage = Nutzung
admin-billing-invoices = Rechnungen
admin-billing-payment-methods = Zahlungsmethoden
admin-billing-upgrade = Upgrade-Plan
admin-billing-downgrade = Downgrade-Plan
admin-billing-cancel = Abonnement kündigen
admin-billing-invoice-date = Rechnungsdatum
admin-billing-invoice-amount = Betrag
admin-billing-invoice-status = Status
admin-billing-invoice-paid = Bezahlt
admin-billing-invoice-pending = Ausstehend
admin-billing-invoice-overdue = Überfällig
admin-billing-invoice-download = Rechnung herunterladen

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = Sichern und Wiederherstellen
admin-backup-create = Backup erstellen
admin-backup-restore = Sicherung wiederherstellen
admin-backup-schedule = Planen Sie Backups
admin-backup-list = Sicherungsverlauf
admin-backup-name = Sicherungsname
admin-backup-size = Größe
admin-backup-created = Erstellt
admin-backup-download = Herunterladen
admin-backup-delete = Löschen
admin-backup-confirm-restore = Sind Sie sicher, dass Sie dieses Backup wiederherstellen möchten? Dadurch werden die aktuellen Daten überschrieben.
admin-backup-confirm-delete = Sind Sie sicher, dass Sie dieses Backup löschen möchten?
admin-backup-in-progress = Sicherung läuft...
admin-backup-completed = Sicherung erfolgreich abgeschlossen
admin-backup-failed = Die Sicherung ist fehlgeschlagen
admin-backup-restore-in-progress = Wiederherstellung läuft...
admin-backup-restore-completed = Wiederherstellung erfolgreich abgeschlossen
admin-backup-restore-failed = Wiederherstellung fehlgeschlagen

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = Wartungsmodus
admin-maintenance-enable = Aktivieren Sie den Wartungsmodus
admin-maintenance-disable = Deaktivieren Sie den Wartungsmodus
admin-maintenance-status = Aktueller Status
admin-maintenance-active = Der Wartungsmodus ist aktiv
admin-maintenance-inactive = Der Wartungsmodus ist inaktiv
admin-maintenance-message = Wartungsmeldung
admin-maintenance-default-message = Wir führen derzeit planmäßige Wartungsarbeiten durch. Bitte schauen Sie bald wieder vorbei.
admin-maintenance-allowed-ips = Zulässige IP-Adressen
admin-maintenance-confirm-enable = Sind Sie sicher, dass Sie den Wartungsmodus aktivieren möchten? Benutzer können nicht auf das System zugreifen.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = Erforderlich
admin-optional = Optional
admin-loading = Laden...
admin-saving = Sparen...
admin-deleting = Löschen...
admin-confirm = Bestätigen
admin-cancel = Abbrechen
admin-save = Speichern
admin-create = Erstellen
admin-update = Aktualisieren
admin-delete = Löschen
admin-edit = Bearbeiten
admin-view = Ansicht
admin-close = Schließen
admin-back = Zurück
admin-next = Als nächstes
admin-previous = Zurück
admin-refresh = Aktualisieren
admin-export = Exportieren
admin-import = Importieren
admin-search = Suchen
admin-filter = Filtern
admin-clear = Klar
admin-select = Auswählen
admin-select-all = Wählen Sie „Alle“ aus
admin-deselect-all = Alle abwählen
admin-actions = Aktionen
admin-more-actions = Weitere Aktionen
admin-no-data = Keine Daten verfügbar
admin-error = Es ist ein Fehler aufgetreten
admin-success = Erfolg
admin-warning = Warnung
admin-info = Informationen

# Table Pagination
admin-showing = Es werden { $from } bis { $to } von { $total } Ergebnissen angezeigt
admin-page = Seite { $current } von { $total }
admin-items-per-page = Artikel pro Seite
admin-go-to-page = Gehe zur Seite

# Bulk Actions
admin-bulk-delete = Ausgewählte löschen
admin-bulk-export = Ausgewählte exportieren
admin-bulk-activate = Ausgewählte aktivieren
admin-bulk-deactivate = Ausgewählte deaktivieren
admin-selected-count = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
