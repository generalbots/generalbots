<#
.SYNOPSIS
Checks the Windows runtime prerequisites for General Bots.

.DESCRIPTION
Read-only prerequisite report for the desktop shell and the agent backend
(WSL2 + Incus). The script never downloads or installs anything.

Exit codes:
  0  desktop shell and agent mode can run
  1  desktop shell cannot run (WebView2, VC++ 2015-2022 x64 runtime or
     libpq.dll missing)
  2  desktop shell can run, but agent mode cannot (WSL2, distro, systemd,
     Incus or its storage pool missing)
  3  this is not Windows

-Install prints the remediation commands. It performs no installation.

.EXAMPLE
PS> .\PROD-DEPENDENCIES.ps1 -Check

.EXAMPLE
PS> .\PROD-DEPENDENCIES.ps1 -Install
#>
[CmdletBinding()]
param(
    [switch]$Check,
    [switch]$Install
)

$ErrorActionPreference = 'Continue'

function Write-Ok {
    param([string]$Message)
    Write-Host "  [ok]      $Message" -ForegroundColor Green
}

function Write-Missing {
    param([string]$Message)
    Write-Host "  [missing] $Message" -ForegroundColor Red
}

function Write-Note {
    param([string]$Message)
    Write-Host "  [info]    $Message" -ForegroundColor Cyan
}

function Write-Caution {
    param([string]$Message)
    Write-Host "  [warn]    $Message" -ForegroundColor Yellow
}

function Get-WebView2Runtime {
    $keys = @(
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
        'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
        'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    )
    foreach ($key in $keys) {
        $entry = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
        if ($null -ne $entry -and $entry.pv -and $entry.pv -ne '0.0.0.0') {
            return "WebView2 runtime $($entry.pv)"
        }
    }
    $roots = @()
    if (${env:ProgramFiles(x86)}) {
        $roots += (Join-Path ${env:ProgramFiles(x86)} 'Microsoft\EdgeWebView\Application')
    }
    if ($env:ProgramFiles) {
        $roots += (Join-Path $env:ProgramFiles 'Microsoft\EdgeWebView\Application')
    }
    foreach ($root in $roots) {
        if (Test-Path -LiteralPath $root) {
            $exe = Get-ChildItem -LiteralPath $root -Filter 'msedgewebview2.exe' -Recurse -ErrorAction SilentlyContinue `
                | Select-Object -First 1
            if ($exe) {
                return "WebView2 runtime at $($exe.FullName)"
            }
        }
    }
    return $null
}

function Get-VcRuntime {
    $keys = @(
        'HKLM:\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64',
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64'
    )
    foreach ($key in $keys) {
        $entry = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
        if ($null -ne $entry -and $entry.Installed -eq 1) {
            $version = if ($entry.Version) { $entry.Version } else { 'version unknown' }
            return "VC++ 2015-2022 x64 runtime $version"
        }
    }
    $system32 = if ($env:WINDIR) { Join-Path $env:WINDIR 'System32' } else { $null }
    if ($system32) {
        $required = @('vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll')
        $missing = @($required | Where-Object { -not (Test-Path -LiteralPath (Join-Path $system32 $_)) })
        if ($missing.Count -eq 0) {
            return 'VC++ 2015-2022 x64 runtime (System32 DLLs present)'
        }
    }
    return $null
}

function Find-LibpqDll {
    $pathCandidates = @()
    foreach ($dir in ($env:Path -split ';')) {
        $trimmed = $dir.Trim().Trim('"')
        if ($trimmed) {
            $pathCandidates += (Join-Path $trimmed 'libpq.dll')
        }
    }
    foreach ($candidate in $pathCandidates) {
        if (Test-Path -LiteralPath $candidate) {
            return [pscustomobject]@{ Path = $candidate; Usable = $true; Reason = 'on PATH' }
        }
    }

    $localDirs = @()
    if ($PSScriptRoot) {
        $localDirs += $PSScriptRoot
    }
    $localDirs += (Get-Location).Path
    foreach ($dir in ($localDirs | Select-Object -Unique)) {
        $candidate = Join-Path $dir 'libpq.dll'
        if (Test-Path -LiteralPath $candidate) {
            $besideServer = Test-Path -LiteralPath (Join-Path $dir 'botserver.exe')
            $reason = if ($besideServer) { 'beside botserver.exe' } else { 'not on PATH' }
            return [pscustomobject]@{ Path = $candidate; Usable = $besideServer; Reason = $reason }
        }
    }

    foreach ($candidate in @('C:\pgsql\pgsql\bin\libpq.dll', 'C:\pgsql\pgsql\lib\libpq.dll')) {
        if (Test-Path -LiteralPath $candidate) {
            return [pscustomobject]@{ Path = $candidate; Usable = $false; Reason = 'not on PATH' }
        }
    }
    return $null
}

function Find-LibpqLib {
    $dirs = @()
    $pqDir = $env:PQ_LIB_DIR
    if (-not $pqDir) {
        $pqDir = [System.Environment]::GetEnvironmentVariable('PQ_LIB_DIR', 'User')
    }
    if (-not $pqDir) {
        $pqDir = [System.Environment]::GetEnvironmentVariable('PQ_LIB_DIR', 'Machine')
    }
    if ($pqDir) {
        $dirs += $pqDir.Trim().Trim('"')
    }
    $dirs += 'C:\pgsql\pgsql\lib'
    foreach ($dir in $dirs) {
        $candidate = Join-Path $dir 'libpq.lib'
        if (Test-Path -LiteralPath $candidate) {
            return $candidate
        }
    }
    return $null
}

function Invoke-Native {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )
    $lines = & $FilePath @Arguments 2>&1
    $code = $LASTEXITCODE
    $text = (@($lines) | ForEach-Object { "$_" }) -join "`n"
    return [pscustomobject]@{
        ExitCode = $code
        Text     = $text.Trim()
    }
}

function Test-AgentBackend {
    $distro = $env:GBO_WSL_DISTRO
    if (-not $distro) {
        $distro = 'Debian'
    }
    $distro = $distro.Trim()

    $wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
    if (-not $wsl) {
        Write-Missing 'wsl.exe not found: WSL2 is not installed.'
        Write-Note 'Agent mode cannot run. Desktop Chat/Drive does not need WSL2.'
        return 1
    }

    $status = Invoke-Native -FilePath 'wsl.exe' -Arguments @('--status')
    if ($status.ExitCode -ne 0) {
        Write-Missing "WSL2 is not ready (wsl --status exited $($status.ExitCode))."
        if ($status.Text) {
            Write-Note $status.Text
        }
        return 1
    }
    Write-Ok 'WSL2 is installed.'

    $probe = Invoke-Native -FilePath 'wsl.exe' -Arguments @('-d', $distro, '-u', 'root', '--', 'true')
    if ($probe.ExitCode -ne 0) {
        Write-Missing "WSL distro '$distro' is not available (set GBO_WSL_DISTRO to another installed distro)."
        return 1
    }
    Write-Ok "WSL distro '$distro' starts as root."

    $kernel = Invoke-Native -FilePath 'wsl.exe' -Arguments @('-d', $distro, '-u', 'root', '--', 'uname', '-r')
    if ($kernel.ExitCode -eq 0 -and $kernel.Text) {
        if ($kernel.Text -match 'WSL2|microsoft-standard') {
            Write-Ok "WSL2 kernel: $($kernel.Text)"
        } elseif ($kernel.Text -match 'Microsoft') {
            Write-Missing "distro '$distro' runs WSL1 ($($kernel.Text)); Incus requires WSL2."
            return 1
        } else {
            Write-Caution "could not classify the WSL kernel: $($kernel.Text)"
        }
    }

    $systemd = Invoke-Native -FilePath 'wsl.exe' -Arguments @('-d', $distro, '-u', 'root', '--', 'systemctl', 'is-system-running')
    $systemdState = $systemd.Text.Trim()
    $systemdStates = @($systemd.Text -split "`n" | ForEach-Object { $_.Trim() })
    if ($systemdStates -contains 'running' -or $systemdStates -contains 'degraded' -or $systemdStates -contains 'starting') {
        Write-Ok "systemd is running in '$distro'."
    } else {
        Write-Missing "systemd is not running in '$distro' (state: $systemdState)."
        Write-Note "Add '[boot] systemd=true' to /etc/wsl.conf inside '$distro', then run: wsl --shutdown"
        return 1
    }

    $incus = Invoke-Native -FilePath 'wsl.exe' -Arguments @('-d', $distro, '-u', 'root', '--', 'incus', 'version')
    if ($incus.ExitCode -ne 0) {
        Write-Missing "Incus is not installed or its daemon is not running in '$distro'."
        $firstLine = @($incus.Text -split "`n")[0]
        if ($firstLine) {
            Write-Note $firstLine
        }
        return 1
    }
    $versionLine = @($incus.Text -split "`n" | Where-Object { $_ })[0]
    Write-Ok "Incus reachable in '$distro' ($versionLine)."

    $storage = Invoke-Native -FilePath 'wsl.exe' -Arguments @('-d', $distro, '-u', 'root', '--', 'incus', 'storage', 'list', '--format', 'csv')
    if ($storage.ExitCode -ne 0 -or -not $storage.Text) {
        Write-Missing "Incus has no storage pool in '$distro'."
        Write-Note "Run: wsl -d $distro -u root -- incus admin init --minimal"
        return 1
    }
    Write-Ok 'Incus storage pool is configured.'
    return 0
}

function Show-InstallGuidance {
    Write-Host ''
    Write-Host '========================================' -ForegroundColor Cyan
    Write-Host '  Remediation (guidance only)' -ForegroundColor Cyan
    Write-Host '========================================' -ForegroundColor Cyan
    Write-Host ''
    Write-Host 'This script does not download or install anything. Run these commands yourself:' -ForegroundColor Yellow
    Write-Host ''
    Write-Host 'Desktop shell:' -ForegroundColor Cyan
    Write-Host '  winget install --id Microsoft.EdgeWebView2Runtime -e'
    Write-Host '  winget install --id Microsoft.VCRedist.2015+.x64 -e'
    Write-Host '  .\DEPENDENCIES.ps1        # PostgreSQL binaries + PQ_LIB_DIR (build helper)'
    Write-Host '  Add C:\pgsql\pgsql\bin to PATH, then open a new terminal.'
    Write-Host ''
    Write-Host 'Agent mode (WSL2 + Incus):' -ForegroundColor Cyan
    Write-Host '  wsl --install -d Debian'
    Write-Host '  wsl --set-default-version 2'
    Write-Host '  wsl -d Debian -u root -- apt-get update'
    Write-Host '  wsl -d Debian -u root -- apt-get install -y incus'
    Write-Host '  wsl -d Debian -u root -- systemctl enable --now incus'
    Write-Host '  wsl -d Debian -u root -- incus admin init --minimal'
    Write-Host ''
    Write-Host "  Add '[boot] systemd=true' to /etc/wsl.conf inside the distro, then: wsl --shutdown" -ForegroundColor Yellow
    Write-Host '  For another distro, set GBO_WSL_DISTRO before starting botserver.' -ForegroundColor Yellow
    Write-Host '  botagent needs a WSL wrapper in AGENT_INCUS_BIN; see botbook/src/07-user-interface/first-run-wizard.md.' -ForegroundColor Yellow
    Write-Host ''
    Write-Host 'Media features:' -ForegroundColor Cyan
    Write-Host '  The Windows BotModels pythonbox (botmodels-pythonbox-windows-x86_64.zip)' -ForegroundColor Yellow
    Write-Host '  is bundled in installers/; the first use builds its venv from the offline' -ForegroundColor Yellow
    Write-Host '  wheelhouse, so vision/STT/TTS need no network access.' -ForegroundColor Yellow
    Write-Host ''
    Write-Host 'Offline components:' -ForegroundColor Cyan
    Write-Host '  Copy the component archives into installers/ next to generalbots.exe;' -ForegroundColor Yellow
    Write-Host '  desktop-local builds do not download components at runtime.' -ForegroundColor Yellow
    Write-Host ''
}

if ([System.Environment]::OSVersion.Platform -ne [System.PlatformID]::Win32NT) {
    Write-Host 'This script checks Windows prerequisites only.' -ForegroundColor Red
    Write-Host 'Linux: ./DEPENDENCIES.sh    macOS: ./PROD-DEPENDENCIES.sh --check' -ForegroundColor Yellow
    exit 3
}

$runCheck = [bool]$Check
$runInstall = [bool]$Install
if (-not $runCheck -and -not $runInstall) {
    $runCheck = $true
}

$result = 0
if ($runCheck) {
    Write-Host ''
    Write-Host '========================================' -ForegroundColor Green
    Write-Host '  General Bots runtime prerequisites' -ForegroundColor Green
    Write-Host '  (Windows)' -ForegroundColor Green
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ''
    Write-Note "Windows $([System.Environment]::OSVersion.Version) / $env:PROCESSOR_ARCHITECTURE"
    Write-Host ''

    $desktopMissing = 0
    Write-Host '--- Desktop shell ---' -ForegroundColor Cyan

    $webview = Get-WebView2Runtime
    if ($webview) {
        Write-Ok $webview
    } else {
        Write-Missing 'WebView2 runtime not found (required by the desktop shell).'
        $desktopMissing++
    }

    $vc = Get-VcRuntime
    if ($vc) {
        Write-Ok $vc
    } else {
        Write-Missing 'VC++ 2015-2022 x64 runtime not found.'
        $desktopMissing++
    }

    $libpqDll = Find-LibpqDll
    if ($null -ne $libpqDll) {
        if ($libpqDll.Usable) {
            Write-Ok "libpq.dll ($($libpqDll.Reason)): $($libpqDll.Path)"
        } else {
            Write-Missing "libpq.dll found at $($libpqDll.Path) but $($libpqDll.Reason); add its directory to PATH or copy the DLL next to botserver.exe."
            $desktopMissing++
        }
    } else {
        Write-Missing 'libpq.dll not found on PATH, next to the script or in C:\pgsql\pgsql\bin.'
        $desktopMissing++
    }

    $libpqLib = Find-LibpqLib
    if ($libpqLib) {
        Write-Ok "libpq.lib for source builds: $libpqLib"
    } else {
        Write-Note 'libpq.lib not found (only needed to build from source; run .\DEPENDENCIES.ps1).'
    }

    $installerDirs = @()
    if ($PSScriptRoot) {
        $installerDirs += $PSScriptRoot
    }
    $installerDirs += (Get-Location).Path
    $installers = $null
    foreach ($dir in ($installerDirs | Select-Object -Unique)) {
        $candidate = Join-Path $dir 'installers'
        if (Test-Path -LiteralPath $candidate) {
            $installers = $candidate
            break
        }
    }
    if ($installers) {
        Write-Ok "offline component cache: $installers"
    } else {
        Write-Caution "no installers/ directory here; the desktop-local package manager refuses runtime downloads ('not found in bundled installers')."
    }

    if ($installers -and (Test-Path (Join-Path $installers 'botmodels-pythonbox-windows-x86_64.zip'))) {
        Write-Ok 'BotModels Windows pythonbox is bundled; vision/STT/TTS install offline on first use.'
    } else {
        Write-Caution 'no BotModels Windows pythonbox in installers/; vision/STT/TTS need a first-run download.'
    }

    Write-Host ''
    Write-Host '--- Agent mode (WSL2 + Incus) ---' -ForegroundColor Cyan
    $agentMissing = Test-AgentBackend

    Write-Host ''
    if ($desktopMissing -gt 0) {
        Write-Host 'Result: DESKTOP BLOCKED - fix the [missing] items above.' -ForegroundColor Red
        $result = 1
    } elseif ($agentMissing -gt 0) {
        Write-Host 'Result: AGENT BLOCKED - desktop Chat/Drive can run; agent mode cannot.' -ForegroundColor Yellow
        $result = 2
    } else {
        Write-Host 'Result: OK - desktop shell and agent mode can run.' -ForegroundColor Green
        $result = 0
    }
    Write-Host 'Run .\PROD-DEPENDENCIES.ps1 -Install for the remediation commands.' -ForegroundColor Cyan
}

if ($runInstall) {
    Show-InstallGuidance
}

exit $result
