# Password Security

General Bots delegates all password security to the Directory Service (currently Zitadel, can be migrated to Keycloak), an enterprise-grade identity management platform. No passwords are ever stored, hashed, or managed within General Bots itself.

## Overview

Password security is handled entirely by Zitadel, which provides:
- Industry-standard password hashing (Argon2/bcrypt)
- Configurable password policies
- Password history and rotation
- Breach detection
- Self-service password recovery

## No Internal Password Management

### What General Bots Does NOT Do

- **No password storage**: No password or hash columns in database
- **No hashing implementation**: No Argon2/bcrypt code in botserver
- **No password validation**: All validation done by Zitadel
- **No password reset logic**: Handled through Zitadel workflows
- **No password policies**: Configured in Zitadel admin console

### What General Bots DOES Do

- Redirects to Zitadel for authentication
- Stores Zitadel user IDs
- Manages local session tokens
- Caches user profile information
- Validates sessions locally for performance

## Zitadel Password Security

### Hashing Algorithm

Zitadel uses industry-standard algorithms:
- **Default**: Argon2id (recommended)
- **Alternative**: bcrypt (for compatibility)
- **Configurable**: Parameters can be adjusted
- **Automatic**: Rehashing on algorithm updates

### Password Policies

Configured in Zitadel admin console:
- Minimum length (default: 8 characters)
- Maximum length (configurable)
- Character requirements (uppercase, lowercase, numbers, symbols)
- Complexity rules
- Common password blacklist
- Password history (prevent reuse)
- Expiration policies

### Password Storage in Zitadel

Zitadel stores:
- Hashed passwords (never plaintext)
- Salt per password
- Algorithm identifier
- Hash parameters
- Password history
- Last changed timestamp

## Configuration

### Setting Password Policies

Access Zitadel admin console:
1. Navigate to Settings → Password Complexity
2. Configure requirements:
   - Min/max length
   - Required character types
   - Expiry settings
3. Save changes (applies immediately)

### Example Policy Configuration

In Zitadel UI or API:
```json
{
  "minLength": 12,
  "maxLength": 128,
  "hasUppercase": true,
  "hasLowercase": true,
  "hasNumber": true,
  "hasSymbol": true,
  "passwordHistory": 5,
  "expiryDays": 90
}
```

## Password Reset Flow

### User-Initiated Reset

1. User clicks "Forgot Password" on Zitadel login
2. Email sent with reset link
3. User clicks link (time-limited)
4. New password entered in Zitadel UI
5. Password validated against policy
6. Hash updated in Zitadel database
7. User can login with new password

### Admin-Initiated Reset

1. Admin accesses Zitadel console
2. Navigates to user management
3. Triggers password reset
4. User receives reset email
5. Same flow as user-initiated

## Security Features

### Breach Detection

Zitadel includes:
- Have I Been Pwned integration
- Checks passwords against breach databases
- Warns users of compromised passwords
- Forces reset if detected in breach

### Multi-Factor Authentication

Additional security beyond passwords:
- TOTP (Google Authenticator, etc.)
- WebAuthn/FIDO2 keys
- SMS OTP (if configured)
- Email verification codes

### Account Protection

- Account lockout after failed attempts
- CAPTCHA after threshold
- IP-based rate limiting
- Suspicious activity detection
- Passwordless options available

## Integration Points

### Bootstrap Process

During setup, General Bots:
1. Installs Directory Service (Zitadel)
2. Configures database connection
3. Creates admin account with randomly generated password
4. Password is displayed once during initial setup

### Authentication Flow

1. User enters credentials in Directory Service UI
2. Directory Service validates password
3. OIDC tokens issued
4. General Bots receives tokens
5. No password ever touches General Bots

### Session Management

After Directory Service authentication:
- General Bots creates local session
- Session token generated (not password-related)
- User ID linked to Directory Service ID
- No password data stored

## Default Credentials

### Initial Admin Account

Created during bootstrap:
- Username: `admin`
- Password: **Randomly generated**
- Displayed once during initial setup
- Should be stored securely or changed immediately

### Initial User Account

Created during bootstrap:
- Username: `user`
- Password: **Randomly generated**
- Displayed once during initial setup
- Must be changed on first login

## Best Practices

### For Administrators

1. **Secure Initial Passwords**: Store or change randomly generated passwords immediately
2. **Configure Policies**: Set appropriate password requirements
3. **Enable MFA**: Require for admin accounts
4. **Monitor Logs**: Review authentication attempts
5. **Update Regularly**: Keep Zitadel updated
6. **Test Recovery**: Verify password reset works through Directory Service

### For Developers

1. **Never Touch Passwords**: Let Zitadel handle everything
2. **Use OIDC Flow**: Standard OAuth2/OpenID Connect
3. **Validate Tokens**: Check with Zitadel when needed
4. **Cache Carefully**: Don't cache sensitive data
5. **Log Safely**: Never log authentication details

### For Users

1. **Use Strong Passwords**: Follow policy requirements
2. **Enable MFA**: Add extra security layer
3. **Unique Passwords**: Don't reuse across services
4. **Regular Updates**: Change periodically if required
5. **Report Issues**: Alert admins of problems

## Compliance

Zitadel's password handling helps meet:
- **GDPR**: Data protection requirements
- **NIST 800-63B**: Modern password guidelines
- **OWASP**: Security best practices
- **PCI DSS**: Payment card standards
- **HIPAA**: Healthcare requirements
- **SOC 2**: Security controls

## Troubleshooting

### Common Password Issues

1. **Password Reset Not Working**
   - Check email configuration
   - Verify SMTP settings in Zitadel
   - Check spam folders

2. **Policy Not Enforced**
   - Review Zitadel configuration
   - Check policy is active
   - Verify user's organization settings

3. **Account Locked**
   - Check lockout policy
   - Admin can unlock via console
   - Wait for timeout period

4. **MFA Issues**
   - Verify time sync for TOTP
   - Check backup codes
   - Admin can reset MFA

## Security Benefits

### Delegated Security

- **Professional Implementation**: Security experts maintain Zitadel
- **Regular Updates**: Security patches applied by Zitadel team
- **Compliance**: Certifications maintained by Zitadel
- **No Liability**: Password breaches not botserver's responsibility

### Reduced Attack Surface

- No password code to exploit
- No hashing vulnerabilities
- No timing attacks possible
- No password database to breach

### Advanced Features

Available through Zitadel:
- Passwordless authentication
- Biometric support
- Hardware key support
- Risk-based authentication
- Adaptive security

## Migration Guide

### From Internal Passwords

If migrating from a system with internal passwords:

1. **Export Users**: Username and email only (no passwords)
2. **Import to Zitadel**: Create accounts
3. **Force Reset**: All users must set new passwords
4. **Remove Old Code**: Delete password-related code
5. **Update Docs**: Reflect new authentication flow

### Password Policy Migration

1. Document existing policy
2. Configure equivalent in Zitadel
3. Test with sample accounts
4. Communicate changes to users
5. Provide support during transition

## Summary

General Bots achieves enterprise-grade password security by not handling passwords at all. The Directory Service provides professional identity management with all the security features needed for production deployments. This separation of concerns allows General Bots to focus on bot functionality while delegating security to a specialized platform.