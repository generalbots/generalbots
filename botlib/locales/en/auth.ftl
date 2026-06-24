# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Authentication
auth-login = Log In
auth-logout = Log Out
auth-signup = Sign Up
auth-welcome = Welcome
auth-welcome-back = Welcome back, { $name }!
auth-session-expired = Your session has expired
auth-session-timeout = Session timeout in { $minutes } minutes

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Sign in to your account
auth-login-subtitle = Enter your credentials to continue
auth-login-email = Email Address
auth-login-username = Username
auth-login-password = Password
auth-login-remember = Remember me
auth-login-forgot = Forgot password?
auth-login-submit = Sign In
auth-login-loading = Signing in...
auth-login-or = or continue with
auth-login-no-account = Don't have an account?
auth-login-create-account = Create an account

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Passkeys
passkey-subtitle = Secure, passwordless authentication
passkey-description = Passkeys use your device's biometrics or PIN for secure, phishing-resistant sign-in
passkey-what-is = What is a passkey?
passkey-benefits = Benefits of passkeys
passkey-benefit-secure = More secure than passwords
passkey-benefit-easy = Easy to use - no passwords to remember
passkey-benefit-fast = Fast sign-in with biometrics
passkey-benefit-phishing = Resistant to phishing attacks

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Set Up Passkey
passkey-register-subtitle = Create a passkey for faster, more secure sign-in
passkey-register-description = Your device will ask you to verify your identity using your fingerprint, face, or screen lock
passkey-register-button = Create Passkey
passkey-register-name = Passkey Name
passkey-register-name-placeholder = e.g., MacBook Pro, iPhone
passkey-register-name-hint = Give your passkey a name to identify it later
passkey-register-loading = Setting up passkey...
passkey-register-verifying = Verifying with your device...
passkey-register-success = Passkey created successfully
passkey-register-error = Failed to create passkey
passkey-register-cancelled = Passkey setup cancelled
passkey-register-not-supported = Your browser doesn't support passkeys

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Sign in with Passkey
passkey-login-subtitle = Use your passkey for secure, passwordless sign-in
passkey-login-button = Sign in with Passkey
passkey-login-loading = Authenticating...
passkey-login-verifying = Verifying passkey...
passkey-login-success = Signed in successfully
passkey-login-error = Authentication failed
passkey-login-cancelled = Authentication cancelled
passkey-login-no-passkeys = No passkeys found for this account
passkey-login-try-another = Try another method

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Manage Passkeys
passkey-manage-subtitle = View and manage your registered passkeys
passkey-manage-count = { $count ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = Add New Passkey
passkey-manage-rename = Rename
passkey-manage-delete = Delete
passkey-manage-created = Created { $date }
passkey-manage-last-used = Last used { $date }
passkey-manage-never-used = Never used
passkey-manage-this-device = This device
passkey-manage-cross-platform = Cross-platform
passkey-manage-platform = Platform authenticator
passkey-manage-security-key = Security key
passkey-manage-empty = No passkeys registered
passkey-manage-empty-description = Add a passkey for faster, more secure sign-in

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Delete Passkey
passkey-delete-confirm = Are you sure you want to delete this passkey?
passkey-delete-warning = You won't be able to use this passkey to sign in anymore
passkey-delete-last-warning = This is your only passkey. You'll need to use password authentication after deleting it.
passkey-delete-success = Passkey deleted successfully
passkey-delete-error = Failed to delete passkey

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Use Password Instead
passkey-fallback-description = If you can't use your passkey, you can sign in with your password
passkey-fallback-button = Use Password
passkey-fallback-or-passkey = Or sign in with passkey
passkey-fallback-setup-prompt = Set up a passkey for faster sign-in next time
passkey-fallback-setup-later = Maybe later
passkey-fallback-setup-now = Set up now
passkey-fallback-locked = Account temporarily locked
passkey-fallback-locked-description = Too many failed attempts. Try again in { $minutes } minutes.
passkey-fallback-attempts = { $remaining } attempts remaining

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Two-Factor Authentication
mfa-subtitle = Add an extra layer of security to your account
mfa-enabled = Two-factor authentication is enabled
mfa-disabled = Two-factor authentication is disabled
mfa-enable = Enable 2FA
mfa-disable = Disable 2FA
mfa-setup = Set Up 2FA
mfa-verify = Verify Code
mfa-code = Verification Code
mfa-code-placeholder = Enter 6-digit code
mfa-code-sent = Code sent to { $destination }
mfa-code-expired = Code has expired
mfa-code-invalid = Invalid code
mfa-resend = Resend code
mfa-resend-in = Resend in { $seconds }s
mfa-methods = Authentication Methods
mfa-method-app = Authenticator App
mfa-method-sms = SMS
mfa-method-email = Email
mfa-method-passkey = Passkey
mfa-backup-codes = Backup Codes
mfa-backup-codes-description = Save these codes in a safe place. Each code can only be used once.
mfa-backup-codes-remaining = { $count } backup codes remaining
mfa-backup-codes-generate = Generate New Codes
mfa-backup-codes-download = Download Codes
mfa-backup-codes-copy = Copy Codes

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Password
password-change = Change Password
password-current = Current Password
password-new = New Password
password-confirm = Confirm New Password
password-requirements = Password Requirements
password-requirement-length = At least { $length } characters
password-requirement-uppercase = At least one uppercase letter
password-requirement-lowercase = At least one lowercase letter
password-requirement-number = At least one number
password-requirement-special = At least one special character
password-strength = Password Strength
password-strength-weak = Weak
password-strength-fair = Fair
password-strength-good = Good
password-strength-strong = Strong
password-match = Passwords match
password-mismatch = Passwords do not match
password-changed = Password changed successfully
password-change-error = Failed to change password

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Reset Password
password-reset-subtitle = Enter your email to receive a reset link
password-reset-email-sent = Password reset email sent
password-reset-email-sent-description = Check your email for instructions to reset your password
password-reset-invalid-token = Invalid or expired reset link
password-reset-success = Password reset successfully
password-reset-error = Failed to reset password

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Active Sessions
session-subtitle = Manage your active sessions across devices
session-current = Current Session
session-device = Device
session-location = Location
session-last-active = Last Active
session-ip-address = IP Address
session-browser = Browser
session-os = Operating System
session-sign-out = Sign Out
session-sign-out-all = Sign Out All Other Sessions
session-sign-out-confirm = Are you sure you want to sign out of this session?
session-sign-out-all-confirm = Are you sure you want to sign out of all other sessions?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Security
security-subtitle = Manage your account security settings
security-overview = Security Overview
security-last-login = Last Sign In
security-password-last-changed = Password Last Changed
security-security-checkup = Security Checkup
security-checkup-description = Review your security settings
security-recommendation = Recommendation
security-add-passkey = Add a passkey for more secure sign-in
security-enable-mfa = Enable two-factor authentication
security-update-password = Update your password regularly

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Invalid email or password
auth-error-account-locked = Account is locked. Please contact support.
auth-error-account-disabled = Account has been disabled
auth-error-email-not-verified = Please verify your email address
auth-error-too-many-attempts = Too many failed attempts. Please try again later.
auth-error-network = Network error. Please check your connection.
auth-error-server = Server error. Please try again later.
auth-error-unknown = An unknown error occurred
auth-error-session-invalid = Invalid session. Please sign in again.
auth-error-token-expired = Your session has expired. Please sign in again.
auth-error-unauthorized = You are not authorized to perform this action

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Signed in successfully
auth-success-logout = Signed out successfully
auth-success-signup = Account created successfully
auth-success-password-changed = Password changed successfully
auth-success-email-verified = Email verified successfully
auth-success-mfa-enabled = Two-factor authentication enabled
auth-success-mfa-disabled = Two-factor authentication disabled
auth-success-session-terminated = Session terminated successfully

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = New sign-in from { $device } in { $location }
auth-notify-password-changed = Your password was changed
auth-notify-mfa-enabled = Two-factor authentication was enabled
auth-notify-passkey-added = New passkey was added to your account
auth-notify-suspicious-activity = Suspicious activity detected on your account
