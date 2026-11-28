# Stages the offline payload of the government desktop edition outside a
# release build, for checks that need one (scripts/gblite-first-run.ps1).
#
# The files are the ones the release installer carries
# (.github/workflows/gbgov-release.yml, "Stage the offline payload"): OpenBao,
# the PostgreSQL server, OtterIO and the OC client. Address, file name and SHA-256
# of each come from the Windows pins in botserver/3rdparty.toml (`url_win`,
# `filename_win`, `sha256_win`), so a pin changed there is followed here. A
# component with no Windows build pinned stops the script.
#
# The PostgreSQL client libraries the desktop shell stages beside the server
# (botapp/src/desktop/sidecar.rs) are copied into the payload from one of:
#   -RuntimeDllsPin <component>  the EDB binaries zip pinned in the manifest
#                                (the release uses `libpq_win`), downloaded and
#                                SHA-256 checked into -WorkDir, never into the
#                                payload;
#   -RuntimeDllsFrom <folder>    a `bin` folder already on the machine.
#
# Usage (Windows PowerShell 5.1 or PowerShell 7, from the repository root):
#   scripts\gbgov-stage-payload.ps1 -Destination $env:TEMP\gbgov-payload -RuntimeDllsPin libpq_win

param(
    [Parameter(Mandatory = $true)][string]$Destination,
    [string]$Manifest = 'botserver\3rdparty.toml',
    [string[]]$Components = @('openbao', 'tables', 'drive', 'mc'),
    [string]$RuntimeDllsFrom = '',
    [string]$RuntimeDllsPin = '',
    [string]$WorkDir = (Join-Path ([System.IO.Path]::GetTempPath()) 'gbgov-stage-work')
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$pins = @{}
$section = ''
foreach ($line in Get-Content -LiteralPath $Manifest) {
    if ($line -match '^\s*\[components\.([A-Za-z0-9_\-]+)\]\s*$') { $section = $Matches[1]; $pins[$section] = @{}; continue }
    if ($line -match '^\s*\[') { $section = ''; continue }
    if ($section -and $line -match '^\s*([a-z0-9_]+)\s*=\s*"([^"]*)"') { $pins[$section][$Matches[1]] = $Matches[2] }
}

function Save-Pinned([string]$id, [string]$folder) {
    $pin = $pins[$id]
    if (-not $pin) { throw "$Manifest has no [components.$id]" }
    $url = $pin['url_win']
    $expected = $pin['sha256_win']
    if (-not $url -or -not $expected) { throw "component '$id' has no Windows build pinned in $Manifest (url_win, sha256_win)" }
    $name = $pin['filename_win']
    if (-not $name) { $name = $pin['filename'] }
    New-Item -ItemType Directory -Force $folder | Out-Null
    $target = Join-Path $folder $name
    if ((Test-Path -LiteralPath $target) -and (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -eq $expected) {
        Write-Host "already staged: $name (SHA-256 verified)"
        return $target
    }
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        try { Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing; break }
        catch {
            if ($attempt -eq 3) { throw }
            Write-Host "download of $name failed (attempt $attempt): $($_.Exception.Message)"
            Start-Sleep -Seconds 10
        }
    }
    $actual = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected.ToLowerInvariant()) { throw "checksum mismatch for ${name}: expected $expected, got $actual" }
    Write-Host ('staged {0} ({1:N1} MB, SHA-256 verified)' -f $name, ((Get-Item -LiteralPath $target).Length / 1MB))
    return $target
}

if ($RuntimeDllsFrom -and $RuntimeDllsPin) { throw 'pass -RuntimeDllsFrom or -RuntimeDllsPin, not both' }

New-Item -ItemType Directory -Force $Destination | Out-Null
foreach ($id in $Components) { Save-Pinned $id $Destination | Out-Null }

if ($RuntimeDllsPin) {
    $zip = Save-Pinned $RuntimeDllsPin $WorkDir
    $expanded = Join-Path $WorkDir $RuntimeDllsPin
    if (Test-Path -LiteralPath $expanded) { Remove-Item -LiteralPath $expanded -Recurse -Force }
    Expand-Archive -LiteralPath $zip -DestinationPath $expanded -Force
    $RuntimeDllsFrom = Join-Path $expanded 'pgsql\bin'
}

if ($RuntimeDllsFrom) {
    $dlls = 'libpq.dll', 'libcrypto-3-x64.dll', 'libssl-3-x64.dll', 'libiconv-2.dll', 'libintl-9.dll',
        'libwinpthread-1.dll', 'liblz4.dll', 'libzstd.dll', 'zlib1.dll'
    foreach ($dll in $dlls) {
        $source = Join-Path $RuntimeDllsFrom $dll
        if (-not (Test-Path -LiteralPath $source)) { throw "PostgreSQL runtime file $dll is not in $RuntimeDllsFrom" }
        Copy-Item -LiteralPath $source -Destination $Destination -Force
    }
    Write-Host "staged $($dlls.Count) PostgreSQL runtime files from $RuntimeDllsFrom"
}
