<#
.SYNOPSIS
Installs runtime dependencies for General Bots on Windows.

.DESCRIPTION
This script downloads and configures the system libraries required to build
and run BotServer on Windows. It downloads PostgreSQL binaries (for libpq),
sets PQ_LIB_DIR, and verifies the Rust Windows target is installed.

.EXAMPLE
PS> .\DEPENDENCIES.ps1
#>

$ErrorActionPreference = 'Stop'

# ─── COLORS ───
function Write-Step { param($msg) Write-Host "  * $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "  ! $msg" -ForegroundColor Yellow }
function Write-Err  { param($msg) Write-Host "  x $msg" -ForegroundColor Red }

Write-Host "========================================" -ForegroundColor Green
Write-Host "  General Bots Runtime Dependencies"     -ForegroundColor Green
Write-Host "  (Windows)"                             -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# ─── 1. Rust Windows Target ───
Write-Host "`n--- Rust Windows Target ---" -ForegroundColor Cyan
$targets = rustup target list --installed
if ($targets -match "x86_64-pc-windows-gnu") {
    Write-Step "x86_64-pc-windows-gnu target already installed."
} else {
    Write-Host "Installing x86_64-pc-windows-gnu target..."
    rustup target add x86_64-pc-windows-gnu
    if ($LASTEXITCODE -eq 0) {
        Write-Step "Windows target installed successfully."
    } else {
        Write-Err "Failed to install Windows target!"
        exit 1
    }
}

# ─── 2. PostgreSQL binaries (libpq.lib for Diesel ORM) ───
Write-Host "`n--- PostgreSQL libpq ---" -ForegroundColor Cyan
$PgsqlDir = "C:\pgsql\pgsql"
$PgsqlLib = "$PgsqlDir\lib\libpq.lib"
$PgsqlZipUrl = "https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip"
$PgsqlZip = "$env:TEMP\pgsql.zip"

if (Test-Path $PgsqlLib) {
    Write-Step "PostgreSQL binaries already present at $PgsqlDir"
} else {
    Write-Host "Downloading PostgreSQL binaries..." -ForegroundColor Cyan
    Write-Host "  URL: $PgsqlZipUrl"
    Write-Host "  This may take a few minutes (~300MB)...`n"

    Invoke-WebRequest -Uri $PgsqlZipUrl -OutFile $PgsqlZip -UseBasicParsing

    Write-Host "Extracting to C:\pgsql ..."
    if (Test-Path "C:\pgsql") { Remove-Item "C:\pgsql" -Recurse -Force }
    Expand-Archive -Path $PgsqlZip -DestinationPath "C:\pgsql" -Force
    Remove-Item $PgsqlZip -Force -ErrorAction SilentlyContinue

    if (Test-Path $PgsqlLib) {
        Write-Step "PostgreSQL binaries installed successfully."
    } else {
        Write-Err "Failed to find libpq.lib after extraction!"
        exit 1
    }
}

# Set PQ_LIB_DIR permanently for the current user
$CurrentPqDir = [System.Environment]::GetEnvironmentVariable("PQ_LIB_DIR", "User")
if ($CurrentPqDir -ne "$PgsqlDir\lib") {
    [System.Environment]::SetEnvironmentVariable("PQ_LIB_DIR", "$PgsqlDir\lib", "User")
    $env:PQ_LIB_DIR = "$PgsqlDir\lib"
    Write-Step "PQ_LIB_DIR set to '$PgsqlDir\lib' (User environment variable)"
} else {
    Write-Step "PQ_LIB_DIR already configured."
}

# ─── Summary ───
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Dependencies installed!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "You can now build:" -ForegroundColor Cyan
Write-Host "  cargo build -p botserver --target x86_64-pc-windows-gnu"
Write-Host "  cargo build -p botui --target x86_64-pc-windows-gnu"
Write-Host "  .\restart.ps1"
Write-Host ""
Write-Host "NOTE: If this is the first time, restart your terminal" -ForegroundColor Yellow
Write-Host "      so PQ_LIB_DIR takes effect." -ForegroundColor Yellow
