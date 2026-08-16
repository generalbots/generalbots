# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Authentifizierung
auth-login = Anmelden
auth-logout = Abmelden
auth-signup = Melden Sie sich an
auth-welcome = Willkommen
auth-welcome-back = Willkommen zurück, { $name }!
auth-session-expired = Ihre Sitzung ist abgelaufen
auth-session-timeout = Sitzungs-Timeout in { $minutes } Minuten

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Melden Sie sich bei Ihrem Konto an
auth-login-subtitle = Geben Sie Ihre Anmeldedaten ein, um fortzufahren
auth-login-email = E-Mail-Adresse
auth-login-username = Benutzername
auth-login-password = Passwort
auth-login-remember = Erinnere dich an mich
auth-login-forgot = Passwort vergessen?
auth-login-submit = Anmelden
auth-login-loading = Anmelden...
auth-login-or = oder weitermachen
auth-login-no-account = Sie haben noch kein Konto?
auth-login-create-account = Erstellen Sie ein Konto

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Passschlüssel
passkey-subtitle = Sichere, passwortlose Authentifizierung
passkey-description = Passkeys nutzen die Biometrie oder PIN Ihres Geräts für eine sichere, Phishing-resistente Anmeldung
passkey-what-is = Was ist ein Passkey?
passkey-benefits = Vorteile von Passkeys
passkey-benefit-secure = Sicherer als Passwörter
passkey-benefit-easy = Einfach zu bedienen – Sie müssen sich keine Passwörter merken
passkey-benefit-fast = Schnelle Anmeldung mit Biometrie
passkey-benefit-phishing = Resistent gegen Phishing-Angriffe

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Passkey einrichten
passkey-register-subtitle = Erstellen Sie einen Passkey für eine schnellere und sicherere Anmeldung
passkey-register-description = Ihr Gerät fordert Sie auf, Ihre Identität mithilfe Ihres Fingerabdrucks, Ihres Gesichts oder Ihrer Bildschirmsperre zu bestätigen
passkey-register-button = Passkey erstellen
passkey-register-name = Passkey-Name
passkey-register-name-placeholder = z. B. MacBook Pro, iPhone
passkey-register-name-hint = Geben Sie Ihrem Passkey einen Namen, um ihn später identifizieren zu können
passkey-register-loading = Passkey einrichten...
passkey-register-verifying = Überprüfung mit Ihrem Gerät...
passkey-register-success = Passkey erfolgreich erstellt
passkey-register-error = Der Passkey konnte nicht erstellt werden
passkey-register-cancelled = Passkey-Einrichtung abgebrochen
passkey-register-not-supported = Ihr Browser unterstützt keine Passkeys

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Melden Sie sich mit Passkey an
passkey-login-subtitle = Verwenden Sie Ihren Passschlüssel für eine sichere, passwortlose Anmeldung
passkey-login-button = Melden Sie sich mit Passkey an
passkey-login-loading = Authentifizierung...
passkey-login-verifying = Passschlüssel wird überprüft...
passkey-login-success = Erfolgreich angemeldet
passkey-login-error = Die Authentifizierung ist fehlgeschlagen
passkey-login-cancelled = Authentifizierung abgebrochen
passkey-login-no-passkeys = Für dieses Konto wurden keine Passkeys gefunden
passkey-login-try-another = Versuchen Sie es mit einer anderen Methode

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Passkeys verwalten
passkey-manage-subtitle = Zeigen Sie Ihre registrierten Passkeys an und verwalten Sie sie
passkey-manage-count = { $count ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = Neuen Passkey hinzufügen
passkey-manage-rename = Umbenennen
passkey-manage-delete = Löschen
passkey-manage-created = Erstellt { $date }
passkey-manage-last-used = Zuletzt verwendet { $date }
passkey-manage-never-used = Nie benutzt
passkey-manage-this-device = Dieses Gerät
passkey-manage-cross-platform = Plattformübergreifend
passkey-manage-platform = Plattform-Authentifikator
passkey-manage-security-key = Sicherheitsschlüssel
passkey-manage-empty = Keine Passkeys registriert
passkey-manage-empty-description = Fügen Sie einen Passkey für eine schnellere und sicherere Anmeldung hinzu

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Passkey löschen
passkey-delete-confirm = Sind Sie sicher, dass Sie diesen Passkey löschen möchten?
passkey-delete-warning = Sie können sich mit diesem Passkey nicht mehr anmelden
passkey-delete-last-warning = Dies ist Ihr einziger Hauptschlüssel. Nach dem Löschen müssen Sie die Passwortauthentifizierung verwenden.
passkey-delete-success = Passkey erfolgreich gelöscht
passkey-delete-error = Der Kennwortschlüssel konnte nicht gelöscht werden

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Verwenden Sie stattdessen ein Passwort
passkey-fallback-description = Wenn Sie Ihren Passkey nicht verwenden können, können Sie sich mit Ihrem Passwort anmelden
passkey-fallback-button = Passwort verwenden
passkey-fallback-or-passkey = Oder melden Sie sich mit dem Passwort an
passkey-fallback-setup-prompt = Richten Sie einen Passkey ein, um die Anmeldung beim nächsten Mal zu beschleunigen
passkey-fallback-setup-later = Vielleicht später
passkey-fallback-setup-now = Jetzt einrichten
passkey-fallback-locked = Konto vorübergehend gesperrt
passkey-fallback-locked-description = Zu viele gescheiterte Versuche. Versuchen Sie es in { $minutes } Minuten erneut.
passkey-fallback-attempts = { $remaining } verbleibende Versuche

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Zwei-Faktor-Authentifizierung
mfa-subtitle = Fügen Sie Ihrem Konto eine zusätzliche Sicherheitsebene hinzu
mfa-enabled = Die Zwei-Faktor-Authentifizierung ist aktiviert
mfa-disabled = Die Zwei-Faktor-Authentifizierung ist deaktiviert
mfa-enable = Aktivieren Sie 2FA
mfa-disable = Deaktivieren Sie 2FA
mfa-setup = Richten Sie 2FA ein
mfa-verify = Code überprüfen
mfa-code = Bestätigungscode
mfa-code-placeholder = Geben Sie den 6-stelligen Code ein
mfa-code-sent = Code gesendet an { $destination }
mfa-code-expired = Code ist abgelaufen
mfa-code-invalid = Ungültiger Code
mfa-resend = Code erneut senden
mfa-resend-in = In { $seconds }s erneut senden
mfa-methods = Authentifizierungsmethoden
mfa-method-app = Authentifizierungs-App
mfa-method-sms = SMS
mfa-method-email = E-Mail
mfa-method-passkey = Hauptschlüssel
mfa-backup-codes = Backup-Codes
mfa-backup-codes-description = Bewahren Sie diese Codes an einem sicheren Ort auf. Jeder Code kann nur einmal verwendet werden.
mfa-backup-codes-remaining = { $count } verbleibende Backup-Codes
mfa-backup-codes-generate = Generieren Sie neue Codes
mfa-backup-codes-download = Codes herunterladen
mfa-backup-codes-copy = Codes kopieren

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Passwort
password-change = Passwort ändern
password-current = Aktuelles Passwort
password-new = Neues Passwort
password-confirm = Bestätigen Sie das neue Passwort
password-requirements = Passwortanforderungen
password-requirement-length = Mindestens { $length } Zeichen
password-requirement-uppercase = Mindestens ein Großbuchstabe
password-requirement-lowercase = Mindestens ein Kleinbuchstabe
password-requirement-number = Mindestens eine Nummer
password-requirement-special = Mindestens ein Sonderzeichen
password-strength = Passwortstärke
password-strength-weak = Schwach
password-strength-fair = Fair
password-strength-good = Gut
password-strength-strong = Stark
password-match = Passwörter stimmen überein
password-mismatch = Passwörter stimmen nicht überein
password-changed = Passwort erfolgreich geändert
password-change-error = Das Passwort konnte nicht geändert werden

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Passwort zurücksetzen
password-reset-subtitle = Geben Sie Ihre E-Mail-Adresse ein, um einen Link zum Zurücksetzen zu erhalten
password-reset-email-sent = E-Mail zum Zurücksetzen des Passworts gesendet
password-reset-email-sent-description = Überprüfen Sie Ihre E-Mails auf Anweisungen zum Zurücksetzen Ihres Passworts
password-reset-invalid-token = Ungültiger oder abgelaufener Link zum Zurücksetzen
password-reset-success = Passwort erfolgreich zurückgesetzt
password-reset-error = Passwort konnte nicht zurückgesetzt werden

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Aktive Sitzungen
session-subtitle = Verwalten Sie Ihre aktiven Sitzungen geräteübergreifend
session-current = Aktuelle Sitzung
session-device = Gerät
session-location = Standort
session-last-active = Zuletzt aktiv
session-ip-address = IP-Adresse
session-browser = Browser
session-os = Betriebssystem
session-sign-out = Abmelden
session-sign-out-all = Melden Sie sich von allen anderen Sitzungen ab
session-sign-out-confirm = Möchten Sie sich wirklich von dieser Sitzung abmelden?
session-sign-out-all-confirm = Sind Sie sicher, dass Sie sich von allen anderen Sitzungen abmelden möchten?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Sicherheit
security-subtitle = Verwalten Sie die Sicherheitseinstellungen Ihres Kontos
security-overview = Sicherheitsübersicht
security-last-login = Letzte Anmeldung
security-password-last-changed = Zuletzt geändertes Passwort
security-security-checkup = Sicherheitsüberprüfung
security-checkup-description = Überprüfen Sie Ihre Sicherheitseinstellungen
security-recommendation = Empfehlung
security-add-passkey = Fügen Sie einen Passkey hinzu, um die Anmeldung sicherer zu machen
security-enable-mfa = Aktivieren Sie die Zwei-Faktor-Authentifizierung
security-update-password = Aktualisieren Sie Ihr Passwort regelmäßig

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Ungültige E-Mail-Adresse oder ungültiges Passwort
auth-error-account-locked = Konto ist gesperrt. Bitte wenden Sie sich an den Support.
auth-error-account-disabled = Das Konto wurde deaktiviert
auth-error-email-not-verified = Bitte überprüfen Sie Ihre E-Mail-Adresse
auth-error-too-many-attempts = Zu viele gescheiterte Versuche. Bitte versuchen Sie es später noch einmal.
auth-error-network = Netzwerkfehler. Bitte überprüfen Sie Ihre Verbindung.
auth-error-server = Serverfehler. Bitte versuchen Sie es später noch einmal.
auth-error-unknown = Es ist ein unbekannter Fehler aufgetreten
auth-error-session-invalid = Ungültige Sitzung. Bitte melden Sie sich erneut an.
auth-error-token-expired = Ihre Sitzung ist abgelaufen. Bitte melden Sie sich erneut an.
auth-error-unauthorized = Sie sind nicht berechtigt, diese Aktion durchzuführen

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Erfolgreich angemeldet
auth-success-logout = Erfolgreich abgemeldet
auth-success-signup = Konto erfolgreich erstellt
auth-success-password-changed = Passwort erfolgreich geändert
auth-success-email-verified = E-Mail erfolgreich bestätigt
auth-success-mfa-enabled = Zwei-Faktor-Authentifizierung aktiviert
auth-success-mfa-disabled = Zwei-Faktor-Authentifizierung deaktiviert
auth-success-session-terminated = Sitzung erfolgreich beendet

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Neue Anmeldung von { $device } im { $location }
auth-notify-password-changed = Ihr Passwort wurde geändert
auth-notify-mfa-enabled = Die Zwei-Faktor-Authentifizierung wurde aktiviert
auth-notify-passkey-added = Ihrem Konto wurde ein neuer Passkey hinzugefügt
auth-notify-suspicious-activity = Auf Ihrem Konto wurden verdächtige Aktivitäten festgestellt
