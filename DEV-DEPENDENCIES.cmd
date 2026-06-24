@echo off
:: General Bots Dev Dependencies - Windows Environment
:: ==================================================

setlocal enabledelayedexpansion

echo ========================================
echo  General Bots Development Dependencies
echo  (Windows)
echo ========================================
echo.

:: 1. Check for Git
where git >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Git is not installed. Please install from https://git-scm.com/
    exit /b 1
) else (
    echo [OK] Git detected.
)

:: 2. Check for Rustup/Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed. Please install via https://rustup.rs/
    exit /b 1
) else (
    echo [OK] Cargo detected.
)

:: 3. Install/verify Windows cross-compilation target
echo.
echo [INFO] Checking Rust Windows target...
rustup target list --installed | find "x86_64-pc-windows-gnu" >nul 2>nul
if %errorlevel% neq 0 (
    echo [INFO] Installing x86_64-pc-windows-gnu target...
    rustup target add x86_64-pc-windows-gnu
    if !errorlevel! neq 0 (
        echo [ERROR] Failed to install Windows target.
        exit /b 1
    )
    echo [OK] Windows target installed.
) else (
    echo [OK] x86_64-pc-windows-gnu target already installed.
)

:: 4. Check for MinGW-w64 linker
where x86_64-w64-mingw32-gcc >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARNING] MinGW-w64 cross-compiler not found in PATH.
    echo          Install with: choco install mingw
    echo          Or download from: https://www.mingw-w64.org/
) else (
    echo [OK] MinGW-w64 detected.
)

:: 5. Check for PostgreSQL development libraries (libpq)
echo.
set PGSQL_DIR=C:\pgsql\pgsql
set PGSQL_LIB=%PGSQL_DIR%\lib\libpq.lib

if exist "%PGSQL_LIB%" (
    echo [OK] PostgreSQL libpq detected at %PGSQL_LIB%
) else (
    echo [INFO] PostgreSQL libpq not found at %PGSQL_LIB%
    echo.
    echo [INFO] Downloading PostgreSQL 17.4 binaries for Windows...
    echo        URL: https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip
    echo.
    set PGZIP=%TEMP%\pgsql.zip
    powershell -Command "& { Write-Host 'Downloading PostgreSQL (~300MB)...' -ForegroundColor Cyan; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri 'https://get.enterprisedb.com/postgresql/postgresql-17.4-1-windows-x64-binaries.zip' -OutFile '%TEMP%\pgsql.zip' -UseBasicParsing; if (!(Test-Path '%TEMP%\pgsql.zip')) { Write-Host '[ERROR] Download failed!' -ForegroundColor Red; exit 1 }; Write-Host 'Extracting to C:\pgsql ...'; Expand-Archive -Path '%TEMP%\pgsql.zip' -DestinationPath 'C:\pgsql' -Force; Remove-Item '%TEMP%\pgsql.zip' -Force -ErrorAction SilentlyContinue; if (Test-Path '%PGSQL_LIB%') { Write-Host '[OK] PostgreSQL installed.' -ForegroundColor Green } else { Write-Host '[ERROR] Extraction failed!' -ForegroundColor Red; exit 1 } }"
    if !errorlevel! neq 0 (
        echo [ERROR] Failed to download/extract PostgreSQL.
        echo        Install manually from: https://www.enterprisedb.com/download-postgresql-binaries
        exit /b 1
    )
)

:: 6. Set PQ_LIB_DIR permanently (user level)
echo.
setx PQ_LIB_DIR "%PGSQL_DIR%\lib" >nul 2>nul
echo [OK] PQ_LIB_DIR set to %PGSQL_DIR%\lib (permanent user env var)

:: 7. Create temp directories
if not exist "%TEMP%\gbo" mkdir "%TEMP%\gbo"
echo [OK] Temporary directory created at %TEMP%\gbo

:: 8. Configure .cargo/config.toml for Windows target
echo.
if not exist ".cargo" mkdir ".cargo"
if exist ".cargo\config.toml" (
    find "x86_64-pc-windows-gnu" ".cargo\config.toml" >nul 2>nul
    if !errorlevel! neq 0 (
        echo [INFO] Adding Windows linker configuration to .cargo\config.toml
        echo.>> ".cargo\config.toml"
        echo [target.x86_64-pc-windows-gnu]>> ".cargo\config.toml"
        echo linker = "x86_64-w64-mingw32-gcc">> ".cargo\config.toml"
        echo rustflags = ["-C", "link-arg=-lwinpthread"]>> ".cargo\config.toml"
    )
) else (
    echo [target.x86_64-pc-windows-gnu]>> ".cargo\config.toml"
    echo linker = "x86_64-w64-mingw32-gcc">> ".cargo\config.toml"
    echo rustflags = ["-C", "link-arg=-lwinpthread"]>> ".cargo\config.toml"
)
echo [OK] .cargo\config.toml configured for Windows target.

echo.
echo ========================================
echo  All dependencies installed!
echo ========================================
echo.
echo To build for Windows:
echo   cargo build -p botserver --target x86_64-pc-windows-gnu
echo.
echo To run locally:
echo   .\restart.ps1
echo.
echo NOTE: Restart your terminal so PQ_LIB_DIR takes effect.

exit /b 0
