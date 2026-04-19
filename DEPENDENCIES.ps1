<#
.SYNOPSIS
Installs runtime dependencies for General Bots on Windows.

.DESCRIPTION
This script downloads and configures the system libraries required to build
and run BotServer on Windows. It downloads PostgreSQL binaries (for libpq)
and sets the PQ_LIB_DIR environment variable permanently.

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

# ─── PostgreSQL binaries (libpq.lib for Diesel ORM) ───
$PgsqlDir = "C:\pgsql\pgsql"
$PgsqlLib = "$PgsqlDir\lib\libpq.lib"
$PgsqlZipUrl = "https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip"
$PgsqlZip = "$env:TEMP\pgsql.zip"

if (Test-Path $PgsqlLib) {
    Write-Step "PostgreSQL binaries already present at $PgsqlDir"
} else {
    Write-Host "`nDownloading PostgreSQL binaries..." -ForegroundColor Cyan
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
Write-Host "You can now build and run:" -ForegroundColor Cyan
Write-Host "  cargo build -p botserver"
Write-Host "  cargo build -p botui"
Write-Host "  .\restart.ps1"
Write-Host ""
Write-Host "NOTE: If this is the first time, restart your terminal" -ForegroundColor Yellow
Write-Host "      so PQ_LIB_DIR takes effect." -ForegroundColor Yellow
