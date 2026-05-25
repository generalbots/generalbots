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
if not exist "C:\pgsql\pgsql\lib\libpq.lib" (
    echo.
    echo [INFO] PostgreSQL libpq not found. Run DEPENDENCIES.ps1 to download.
    echo        Or install manually from: https://www.enterprisedb.com/download-postgresql-binaries
) else (
    echo [OK] PostgreSQL libpq detected at C:\pgsql\pgsql\lib\
)

:: 6. Set PQ_LIB_DIR if PostgreSQL is present
if exist "C:\pgsql\pgsql\lib\libpq.lib" (
    set "PQ_LIB_DIR=C:\pgsql\pgsql\lib"
    echo [OK] PQ_LIB_DIR set to !PQ_LIB_DIR!
)

:: 7. Create temp directories
if not exist "%TEMP%\gbo" mkdir "%TEMP%\gbo"
echo [OK] Temporary directory created at %TEMP%\gbo

echo.
echo ========================================
echo  All checks complete.
echo ========================================
echo.
echo To build for Windows natively:
echo   cargo build -p botserver --target x86_64-pc-windows-gnu
echo   cargo build -p botui --target x86_64-pc-windows-gnu
echo.
echo To build and run locally:
echo   .\restart.ps1

exit /b 0
