# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Autenticazione
auth-login = Accedi
auth-logout = Esci
auth-signup = Iscriviti
auth-welcome = Benvenuto
auth-welcome-back = Bentornato, { $name }!
auth-session-timeout = Tempo di sessione in minuti { $minutes }

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Accedi al tuo account
auth-login-subtitle = Inserisca le credenziali per continuare
auth-login-email = Indirizzo email
auth-login-username = Nome utente
auth-login-password = Password
auth-login-remember = Ricordati di me
auth-login-forgot = Hai dimenticato la password?
auth-login-submit = Accedi
auth-login-loading = Firma...
auth-login-or = o continuare con
auth-login-no-account = Non ha un account?
auth-login-create-account = Crei un account

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Passaggi
passkey-subtitle = Autenticazione sicura senza password
passkey-description = Le chiavi di accesso utilizzano la biometria del dispositivo o il PIN per un accesso sicuro e resistente al phishing
passkey-what-is = Che cos'è una chiave di accesso?
passkey-benefits = Vantaggi delle passkey
passkey-benefit-secure = Più sicuro delle password
passkey-benefit-easy = Facile da usare - nessuna password da ricordare
passkey-benefit-fast = Accesso rapido con biometria
passkey-benefit-phishing = Resistente agli attacchi di phishing

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Impostare Passkey
passkey-register-subtitle = Crei una passkey per un accesso più rapido e sicuro
passkey-register-description = Il dispositivo vi chiederà di verificare la vostra identità utilizzando le vostre impronte digitali, faccia, o blocco schermo
passkey-register-button = Crei una passkey
passkey-register-name = Nome della chiave di accesso
passkey-register-name-placeholder = Ad esempio, MacBook Pro, iPhone
passkey-register-name-hint = Dare alla tua passkey un nome per identificarlo in seguito
passkey-register-loading = Prepara la chiave di accesso...
passkey-register-verifying = Verifica con il tuo dispositivo...
passkey-register-success = Passkey creato con successo
passkey-register-error = Impossibile creare la passkey
passkey-register-cancelled = Impostazione passkey annullata
passkey-register-not-supported = Il tuo browser non supporta le passkey

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Accedi con Passkey
passkey-login-subtitle = Usa la tua passkey per accedere senza password e senza password
passkey-login-button = Accedi con Passkey
passkey-login-loading = Autenticazione in corso...
passkey-login-verifying = Verifica della chiave di accesso...
passkey-login-success = Firmato con successo
passkey-login-error = Autenticazione non riuscita
passkey-login-cancelled = Autenticazione annullata
passkey-login-no-passkeys = Nessuna chiavetta trovata per questo account
passkey-login-try-another = Prova un altro metodo

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Gestisci le chiavi di accesso
passkey-manage-subtitle = Visualizza e gestisci le tue passkey registrate
passkey-manage-count = { $count ->
    [one] passkey { $count } registrato
    *[other] Registrati i passkey { $count }
    }
passkey-manage-add = Aggiungi nuova passkey
passkey-manage-rename = Rinomina
passkey-manage-delete = Elimina
passkey-manage-created = Creato { $date }
passkey-manage-last-used = Ultimo { $date } usato
passkey-manage-never-used = Non usato
passkey-manage-this-device = Questo dispositivo
passkey-manage-cross-platform = Multi-piattaforma
passkey-manage-platform = Autenticatore piattaforma
passkey-manage-security-key = Chiave di sicurezza
passkey-manage-empty = Nessuna chiave d'accesso registrata
passkey-manage-empty-description = Aggiungi una passkey per un accesso più rapido e sicuro

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Elimina Passkey
passkey-delete-confirm = È sicuro di voler eliminare questa passkey?
passkey-delete-warning = Non potrai più usare questa chiavetta per accedere
passkey-delete-last-warning = Questa è la tua unica chiave di accesso. Dovrai usare l'autenticazione della password dopo averlo eliminato.
passkey-delete-success = Passkey eliminato con successo
passkey-delete-error = Impossibile eliminare la passkey

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Usare invece la password
passkey-fallback-description = Se non puoi usare la tua password, puoi accedere con la tua password
passkey-fallback-button = Usa password
passkey-fallback-or-passkey = Oppure accedi con la chiavetta
passkey-fallback-setup-prompt = Impostare una chiave di accesso per un accesso più veloce la prossima volta
passkey-fallback-setup-later = Magari dopo.
passkey-fallback-setup-now = Imposta ora
passkey-fallback-locked = Account temporaneamente bloccato
passkey-fallback-locked-description = Troppi tentativi falliti. Riprovare in minuti { $minutes }.
passkey-fallback-attempts = tentativi di { $remaining } rimanenti

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Autenticazione a due fattori
mfa-subtitle = Aggiungi un livello di sicurezza aggiuntivo al tuo account
mfa-enabled = L'autenticazione a due fattori è abilitata
mfa-disabled = L'autenticazione a due fattori è disabilitata
mfa-enable = Abilita 2FA
mfa-disable = Disabilita 2FA
mfa-setup = Impostazione 2FA
mfa-verify = Verifica codice
mfa-code = Codice di verifica
mfa-code-placeholder = Inserisca il codice a 6 cifre.
mfa-code-sent = Codice inviato a { $destination }
mfa-code-expired = Il codice è scaduto
mfa-code-invalid = Codice non valido
mfa-resend = Rimanda codice
mfa-resend-in = Rimanda in { $seconds }s
mfa-methods = Metodi di autenticazione
mfa-method-app = App di autenticazione
mfa-method-sms = SMS
mfa-method-email = Email
mfa-method-passkey = Password
mfa-backup-codes = Codici di backup
mfa-backup-codes-description = Salvare questi codici in un luogo sicuro. Ogni codice può essere utilizzato una sola volta.
mfa-backup-codes-remaining = Codici di backup { $count } rimanenti
mfa-backup-codes-generate = Genera nuovi codici
mfa-backup-codes-download = Scarica codici
mfa-backup-codes-copy = Copi codici

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Password
password-change = Modifica password
password-current = Password corrente
password-new = Nuova password
password-confirm = Conferma nuova password
password-requirements = Requisiti della password
password-requirement-length = Almeno { $length } caratteri
password-requirement-uppercase = Almeno una lettera maiuscola
password-requirement-lowercase = Almeno una lettera minuscola
password-requirement-number = Almeno un numero
password-requirement-special = Almeno un carattere speciale
password-strength = Forza password
password-strength-weak = Debole
password-strength-fair = Equo
password-strength-good = Bene.
password-strength-strong = Forte
password-match = Corrispondenza password
password-mismatch = Le password non corrispondono
password-changed = Password cambiata con successo
password-change-error = Impossibile modificare la password

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Reimposta password
password-reset-subtitle = Inserisca la sua email per ricevere un link di reimpostazione
password-reset-email-sent = Reimpostare la password email inviata
password-reset-email-sent-description = Controlla la tua email per le istruzioni per reimpostare la password
password-reset-invalid-token = Collegamento di reset non valido o scaduto
password-reset-success = Reimpostare la password con successo
password-reset-error = Impossibile reimpostare la password

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Sessioni attive
session-subtitle = Gestisci le sessioni attive tra i dispositivi
session-current = Sessione corrente
session-device = Dispositivo
session-location = Ubicazione
session-last-active = Ultima attività
session-ip-address = Indirizzo IP
session-browser = Navigatore
session-os = Sistema operativo
session-sign-out = Esci
session-sign-out-all = Esci tutte le altre sessioni
session-sign-out-confirm = È sicuro di voler disconnettersi da questa sessione?
session-sign-out-all-confirm = È sicuro di voler disconnettersi da tutte le altre sessioni?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Sicurezza
security-subtitle = Gestisci le impostazioni di sicurezza del tuo account
security-overview = Panoramica sulla sicurezza
security-last-login = Ultimo accesso
security-password-last-changed = Password modificata ultima volta
security-security-checkup = Controllo di sicurezza
security-checkup-description = Rivedere le impostazioni di sicurezza
security-recommendation = Raccomandazione
security-add-passkey = Aggiungi una passkey per un accesso più sicuro
security-enable-mfa = Abilita autenticazione a due fattori
security-update-password = Aggiorna la tua password regolarmente

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Email o password non valide
auth-error-account-locked = Account bloccato. Contatti il supporto.
auth-error-account-disabled = Account è stato disabilitato
auth-error-email-not-verified = Verifica il tuo indirizzo email
auth-error-too-many-attempts = Troppi tentativi falliti. Riprova più tardi.
auth-error-network = Errore di rete. Controllare la connessione.
auth-error-server = Errore del server. Riprova più tardi.
auth-error-unknown = Si è verificato un errore sconosciuto
auth-error-session-invalid = Sessione non valida. Accedere di nuovo.
auth-error-token-expired = La tua sessione è scaduta. Per favore, registrati di nuovo.
auth-error-unauthorized = Non sei autorizzato a eseguire questa azione

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Firmato con successo
auth-success-logout = Esci con successo
auth-success-signup = Account creato con successo
auth-success-password-changed = Password cambiata con successo
auth-success-email-verified = Email verificata con successo
auth-success-mfa-enabled = Autenticazione a due fattori abilitata
auth-success-mfa-disabled = Autenticazione a due fattori disabilitata
auth-success-session-terminated = Sessione terminata con successo

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Nuovo accesso da { $device } in { $location }
auth-notify-password-changed = La password è stata cambiata
auth-notify-mfa-enabled = L'autenticazione a due fattori è stata abilitata
auth-notify-passkey-added = Nuova passkey è stata aggiunta al tuo account
auth-notify-suspicious-activity = Attività sospetta rilevata sul tuo account
