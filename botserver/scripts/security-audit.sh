#!/bin/bash

# General Bots Security Audit Script
# This script helps identify critical security issues in the codebase

set -e

echo "🔒 General Bots Security Audit"
echo "=============================="
echo ""

# Check for hardcoded secrets
echo "1. Checking for hardcoded secrets..."
if grep -r "password\s*=\s*\"" --include="*.rs" --include="*.toml" --include="*.json" . 2>/dev/null | grep -v "test" | grep -v "example" | head -10; then
    echo "⚠️  WARNING: Found potential hardcoded passwords"
else
    echo "✅ No obvious hardcoded passwords found"
fi

echo ""

# Check for unwrap/expect calls
echo "2. Checking for unwrap/expect calls..."
UNWRAP_COUNT=$(grep -r "\.unwrap()\|\.expect(" --include="*.rs" . 2>/dev/null | wc -l)
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    echo "⚠️  WARNING: Found $UNWRAP_COUNT unwrap/expect calls"
    echo "   Sample locations:"
    grep -r "\.unwrap()\|\.expect(" --include="*.rs" . 2>/dev/null | head -5
else
    echo "✅ No unwrap/expect calls found"
fi

echo ""

# Check for Command::new usage
echo "3. Checking for unsafe command execution..."
if grep -r "Command::new" --include="*.rs" . 2>/dev/null | grep -v "SafeCommand" | head -5; then
    echo "⚠️  WARNING: Found potential unsafe command execution"
    echo "   Should use SafeCommand instead"
else
    echo "✅ No unsafe Command::new calls found"
fi

echo ""

# Check for SQL injection patterns
echo "4. Checking for SQL injection patterns..."
if grep -r "format!.*SELECT\|format!.*INSERT\|format!.*UPDATE\|format!.*DELETE" --include="*.rs" . 2>/dev/null | grep -v "sanitize" | head -5; then
    echo "⚠️  WARNING: Found potential SQL injection patterns"
    echo "   Should use sql_guard functions"
else
    echo "✅ No obvious SQL injection patterns found"
fi

echo ""

# Check security headers in routes
echo "5. Checking for security middleware usage..."
if grep -r "security_headers_middleware\|csrf_middleware\|rate_limit_middleware" --include="*.rs" . 2>/dev/null | head -5; then
    echo "✅ Security middleware found"
else
    echo "⚠️  WARNING: No security middleware found in routes"
fi

echo ""

# Check for SecurityManager usage
echo "6. Checking for SecurityManager initialization..."
if grep -r "SecurityManager::new\|SecurityManager::initialize" --include="*.rs" . 2>/dev/null; then
    echo "✅ SecurityManager usage found"
else
    echo "⚠️  WARNING: SecurityManager not initialized"
fi

echo ""

# Check dependencies
echo "7. Checking dependencies..."
if command -v cargo-audit &> /dev/null; then
    echo "Running cargo audit..."
    cargo audit
else
    echo "⚠️  Install cargo-audit: cargo install cargo-audit"
fi

echo ""

# Check for .env files in git
echo "8. Checking for secrets in git..."
if find . -name ".env" -type f | grep -v node_modules | grep -v target; then
    echo "⚠️  WARNING: .env files found in repository"
    echo "   Secrets should be in /tmp/ only"
else
    echo "✅ No .env files in repository"
fi

echo ""

# Check file permissions
echo "9. Checking critical file permissions..."
if [ -f "botserver-stack/conf/vault/init.json" ]; then
    PERMS=$(stat -c "%a" "botserver-stack/conf/vault/init.json")
    if [ "$PERMS" -gt 600 ]; then
        echo "⚠️  WARNING: Vault init file permissions too open: $PERMS"
        echo "   Should be 600 or 400"
    else
        echo "✅ Vault init file permissions OK: $PERMS"
    fi
fi

echo ""

# Summary
echo "📊 Security Audit Summary"
echo "========================"
echo ""
echo "Critical Issues to Address:"
echo "1. $UNWRAP_COUNT unwrap/expect calls need replacement"
echo "2. SecurityManager initialization missing"
echo "3. Security middleware may not be applied to all routes"
echo ""
echo "Next Steps:"
echo "1. Review TASKS.md for detailed remediation plan"
echo "2. Fix P1 issues first (SecurityManager, error handling)"
echo "3. Run cargo clippy and fix all warnings"
echo "4. Implement security testing"
echo ""
echo "For detailed tasks, see: TASKS.md"
echo "For quick checklist, see: SECURITY_CHECKLIST.md"
