# Materialise the virtual environment for this pythonbox.
#
# Usage: .\install.ps1 [-Online] [-Yes] [-OfflineWheels DIR] [-Python PATH]
#                      [-Site DIR] [-With NAME] [-Help]
#
# Offline by default: this bundle ships .\wheelhouse, so pip runs with
# --no-index --find-links .\wheelhouse. -Online opts into PyPI and requires
# -Yes (or BOTMODELS_INSTALL_YES=1) because a full resolve can download several
# gigabytes.
#
# Disk preflight: at least 6 GiB free is required. The wheelhouse estimate is
# wheel size x3 + 1 GiB; an online resolve uses 12 GiB.
# BOTMODELS_MIN_FREE_GIB raises or lowers the floor and
# BOTMODELS_SKIP_DISK_CHECK=1 bypasses the check (not recommended).
#
# -Python  interpreter used to create the venv (default: the bundled one)
# -Site    extra package tree to reuse (for example C:\gbo\pylibs)

[CmdletBinding()]
param(
    [switch]$Online,
    [switch]$Yes,
    [string]$OfflineWheels,
    [string]$Python,
    [string]$Site,
    [string[]]$With = @(),
    [switch]$Help
)

$ErrorActionPreference = 'Stop'
$Root = $PSScriptRoot
$Venv = Join-Path $Root '.venv'
if (-not $Python) { $Python = Join-Path $Root 'python\python.exe' }
$AssumedYes = $env:BOTMODELS_INSTALL_YES
$MinFreeGib = if ($env:BOTMODELS_MIN_FREE_GIB) { [int]$env:BOTMODELS_MIN_FREE_GIB } else { 6 }
$OnlineMinFreeGib = 12
$Wheelhouse = if ($OfflineWheels) { $OfflineWheels } else { Join-Path $Root 'wheelhouse' }

function Show-Usage {
    @'
Usage: .\install.ps1 [-Online] [-Yes] [-OfflineWheels DIR] [-Python PATH]
                     [-Site DIR] [-With NAME] [-Help]

  -With NAME          install requirements-NAME.txt (for example jupyter)
  -OfflineWheels DIR  install from DIR instead of .\wheelhouse
  -Online             resolve from PyPI instead of the offline wheelhouse
  -Yes                confirm the online download (or BOTMODELS_INSTALL_YES=1)
  -Python PATH        interpreter used to create the venv
  -Site DIR           extra package tree to reuse
  -Help               show this help
'@
}

function Fail([string]$Message) {
    [Console]::Error.WriteLine("error: $Message")
    exit 2
}

if ($Help) {
    Show-Usage
    exit 0
}

if (-not (Test-Path $Python)) {
    Fail "bundled interpreter not found: $Python"
}

$hashFiles = @((Join-Path $Root 'requirements.txt'))
$extraArgs = @()
foreach ($extra in $With) {
    $extraPath = Join-Path $Root "requirements-$extra.txt"
    if (-not (Test-Path $extraPath)) {
        Fail "unknown extra '$extra' (expected $extraPath)"
    }
    $extraArgs += @('-r', $extraPath)
    $hashFiles += $extraPath
}

if ($Online -and $OfflineWheels) {
    Fail 'choose either -Online or -OfflineWheels DIR, not both'
}

$pipArgs = @()
$requiredBytes = [int64]$MinFreeGib * 1GB
if ($Online) {
    Write-Host 'Online install: pip will resolve requirements from PyPI.'
    Write-Host '  approximate download: about 4 GiB'
    Write-Host "  required free space:  $OnlineMinFreeGib GiB"
    if ($Yes -or $AssumedYes -eq '1') {
        $requiredBytes = [int64]$OnlineMinFreeGib * 1GB
    } else {
        Fail 'refusing to download without confirmation. Re-run with -Yes or set BOTMODELS_INSTALL_YES=1.'
    }
} elseif (Test-Path $Wheelhouse) {
    $wheelBytes = (Get-ChildItem -Path $Wheelhouse -Recurse -File | Measure-Object -Property Length -Sum).Sum
    $requiredBytes = ([int64]$wheelBytes * 3) + 1GB
    if ($requiredBytes -lt ([int64]$MinFreeGib * 1GB)) {
        $requiredBytes = [int64]$MinFreeGib * 1GB
    }
    $pipArgs = @('--no-index', '--find-links', $Wheelhouse)
} else {
    Fail "no offline wheelhouse at $Wheelhouse. Re-download the pythonbox, pass -OfflineWheels DIR, or opt into PyPI with .\install.ps1 -Online -Yes."
}

$combined = ($hashFiles | ForEach-Object { (Get-FileHash -Algorithm SHA256 $_).Hash }) -join ''
$sha = [System.Security.Cryptography.SHA256]::Create()
$wanted = ([System.BitConverter]::ToString(
    $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($combined))
)).Replace('-', '').ToLowerInvariant()

$venvPython = Join-Path $Venv 'Scripts\python.exe'
$marker = Join-Path $Root '.installed'
if ((Test-Path $venvPython) -and (Test-Path $marker) -and
    ((Get-Content -Raw $marker).Trim() -eq $wanted)) {
    Write-Host "pythonbox already installed (requirements $wanted)"
    exit 0
}

if ($env:BOTMODELS_SKIP_DISK_CHECK -ne '1') {
    $drive = New-Object System.IO.DriveInfo -ArgumentList ([System.IO.Path]::GetPathRoot($Root))
    $free = $drive.AvailableFreeSpace
    if ($free -lt $requiredBytes) {
        $requiredGib = [math]::Round($requiredBytes / 1GB, 1)
        $freeGib = [math]::Round($free / 1GB, 1)
        Fail "not enough free disk space for the BotModels environment. required: $requiredGib GiB, available: $freeGib GiB on $Root. Free the difference and rerun (BOTMODELS_SKIP_DISK_CHECK=1 bypasses this check)."
    }
}

if (-not (Test-Path $venvPython)) {
    & $Python -m venv --system-site-packages $Venv
    if ($LASTEXITCODE -ne 0) { Fail 'could not create the virtual environment' }
}

if ($Site) {
    $siteDir = & $venvPython -c 'import sysconfig; print(sysconfig.get_paths()["purelib"])'
    if ($LASTEXITCODE -ne 0) { Fail 'could not resolve the venv site-packages directory' }
    Set-Content -Path (Join-Path $siteDir 'zzz-pythonbox-site.pth') -Value $Site -Encoding ascii
}

if ($Online) {
    & $venvPython -m pip install --upgrade pip --no-cache-dir
    if ($LASTEXITCODE -ne 0) { Fail 'could not upgrade pip' }
}

$requirements = Join-Path $Root 'requirements.txt'
& $venvPython -m pip install --no-cache-dir @pipArgs -r $requirements @extraArgs
if ($LASTEXITCODE -ne 0) { Fail 'pip install failed' }

& $venvPython -c 'import fastapi, uvicorn; print("pythonbox ready")'
if ($LASTEXITCODE -ne 0) { Fail 'the installed environment cannot import fastapi/uvicorn' }

Set-Content -Path $marker -Value $wanted -Encoding ascii
Write-Host "pythonbox installed (requirements $wanted)"
