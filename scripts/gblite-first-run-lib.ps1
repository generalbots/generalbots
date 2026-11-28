# Helpers of scripts/gblite-first-run.ps1 (dot-sourced by it; not run alone).
# They read $Stack and $AppDir of that script: the stack folder under test and
# the staged app folder, the only two places whose programs are ever stopped.

function Add-Check([string]$Name, [bool]$Ok, [string]$Detail = '') {
    $word = 'FAIL'; if ($Ok) { $word = 'PASS' }
    if ($Detail.Length -gt 420) { $Detail = $Detail.Substring(0, 420) + '...' }
    $line = "$word  $Name"; if ($Detail) { $line = "$line  ($Detail)" }
    Write-Host $line
    [void]$script:checks.Add([pscustomobject]@{ Ok = $Ok; Name = $Name; Line = $line })
}
function Add-Note([string]$Text) { Write-Host "NOTE  $Text" }
function Test-Under([string]$Path, [string]$Root) {
    if (-not $Path) { return $false }
    return $Path.StartsWith($Root.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)
}
function Test-Port([int]$Port) {
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $pending = $client.BeginConnect('127.0.0.1', $Port, $null, $null)
        if (-not $pending.AsyncWaitHandle.WaitOne(500)) { return $false }
        $client.EndConnect($pending); return $true
    } catch { return $false } finally { $client.Close() }
}
function Get-PortOwner([int]$Port) {
    try {
        $socket = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction Stop | Select-Object -First 1
        return Get-CimInstance Win32_Process -Filter "ProcessId = $($socket.OwningProcess)" -ErrorAction Stop
    } catch { return $null }
}
function Get-HttpStatus([string]$Url) {
    try { return [int](Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 5).StatusCode }
    catch { if ($_.Exception.Response) { return [int]$_.Exception.Response.StatusCode }; return 0 }
}
# Lines of a log another process may still be writing.
function Read-Lines([string]$Path) {
    $text = ''
    try {
        $file = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
        try { $text = (New-Object IO.StreamReader($file, [Text.Encoding]::UTF8)).ReadToEnd() } finally { $file.Dispose() }
    } catch { return @() }
    return @($text -split "`r?`n" | Where-Object { $_.Trim() })
}
# `GBWIZARD:{json}` progress lines; anything else is plain log output
# (components.rs:43-48, wizard_provision.rs:36-41).
function Get-WizardEvents([string[]]$Lines) {
    foreach ($line in $Lines) {
        if (-not $line.StartsWith('GBWIZARD:')) { continue }
        try { $item = $line.Substring(9) | ConvertFrom-Json } catch { continue }
        if ($item -and $item.id) { $item }
    }
}
function Get-Owned {
    foreach ($process in @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)) {
        if ((Test-Under $process.ExecutablePath $Stack) -or (Test-Under $process.ExecutablePath $AppDir)) { $process }
    }
}
function Invoke-Tool([string]$File, [string[]]$Arguments, [int]$TimeoutSec) {
    $out = [IO.Path]::GetTempFileName(); $err = [IO.Path]::GetTempFileName()
    try {
        $tool = Start-Process -FilePath $File -ArgumentList $Arguments -PassThru -WindowStyle Hidden `
            -RedirectStandardOutput $out -RedirectStandardError $err
        $null = $tool.Handle
        if (-not $tool.WaitForExit($TimeoutSec * 1000)) { try { $tool.Kill() } catch { }; return $null }
        return $tool.ExitCode
    } catch { return $null } finally { Remove-Item -LiteralPath $out, $err -Force -ErrorAction SilentlyContinue }
}
# Through the Env: drive: a null handed to .NET becomes an empty string, and
# newer runtimes then set an empty variable instead of removing it.
function Set-Env([string]$Name, $Value) {
    if ($null -eq $Value -or $Value -eq '') { Remove-Item -LiteralPath "Env:\$Name" -ErrorAction SilentlyContinue }
    else { Set-Item -LiteralPath "Env:\$Name" -Value $Value }
}
function Set-ChildEnv([string]$Name, $Value) {
    if (-not $script:envBackup.ContainsKey($Name)) { $script:envBackup[$Name] = [Environment]::GetEnvironmentVariable($Name) }
    Set-Env $Name $Value
}
function Show-Tail([string]$Title, [string]$Path, [int]$Count = 60) {
    $lines = @(Read-Lines $Path)
    if ($lines.Count -eq 0) { return }
    Write-Host "----- $Title (last $Count lines) -----"
    $lines | Select-Object -Last $Count | ForEach-Object { Write-Host "    $_" }
}

function Stop-Everything {
    foreach ($process in @(Get-Owned | Where-Object { Test-Under $_.ExecutablePath $AppDir })) {
        Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue
    }
    $data = "$Stack\data\tables\pgdata"
    if (Test-Path -LiteralPath "$data\postmaster.pid") {
        [void](Invoke-Tool "$Stack\bin\tables\bin\pg_ctl.exe" @('-D', "`"$data`"", 'stop', '-m', 'fast', '-s') 30)
    }
    for ($round = 0; $round -lt 20; $round++) {
        $left = @(Get-Owned)
        if ($left.Count -eq 0) { return }
        foreach ($process in $left) { Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue }
        Start-Sleep -Seconds 1
    }
}

# -ReadOnlyApp: the staged app folder is locked as a per-machine install is for
# the person who runs the app. The read-only attribute and an ACL that only drops
# Allow entries do not stop an administrator (the CI runner is one); an explicit
# Deny of every write right does. It is set for Everyone and for the account
# running the check, on the app folder and again on installers\ (in case the
# inherited entry did not reach it), and the lock is measured by creating a file
# in each folder. A probe file that got through is deleted at once and is left
# out of the payload comparison.
$WriteProbe = 'gblite-check-write-probe.tmp'
function Get-LockSids { return @('*S-1-1-0', "*$([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)") }
function Get-WritableFolders {
    $writable = @()
    foreach ($folder in $AppDir, $Installers) {
        $probe = Join-Path $folder $WriteProbe
        try { [IO.File]::WriteAllText($probe, 'x'); $writable += $folder } catch { Write-Verbose "write refused in $folder" }
        if (Test-Path -LiteralPath $probe) { Remove-Item -LiteralPath $probe -Force -ErrorAction SilentlyContinue }
    }
    return $writable
}
function Lock-AppFolder {
    $script:locked = $true
    $problems = @()
    foreach ($folder in $AppDir, $Installers) {
        foreach ($sid in Get-LockSids) {
            $out = & icacls $folder /deny "${sid}:(OI)(CI)(DE,DC,WD,AD,WEA,WA)" /Q
            if ($LASTEXITCODE -ne 0) { $problems += "icacls /deny $sid on $folder exited $LASTEXITCODE $(($out | Out-String).Trim())" }
        }
    }
    $writable = @(Get-WritableFolders)
    $detail = "a new file was refused in $AppDir and in installers\"
    if ($writable.Count -gt 0) {
        $privileges = @(& whoami /priv | Where-Object { $_ -match 'SeBackupPrivilege|SeRestorePrivilege|SeTakeOwnershipPrivilege' } | ForEach-Object { ($_ -replace '\s+', ' ').Trim() })
        $detail = "a file could still be created in $($writable -join ', '); $($problems -join '; '); $($privileges -join '; ')"
    }
    Add-Check 'app folder is read-only, as a per-machine install is' ($writable.Count -eq 0 -and $problems.Count -eq 0) $detail
}
function Unlock-AppFolder {
    foreach ($folder in $Installers, $AppDir) {
        if (-not (Test-Path -LiteralPath $folder)) { continue }
        foreach ($sid in Get-LockSids) { & icacls $folder /remove:d $sid /Q | Out-Null }
    }
    $script:locked = $false
    if (Test-Path -LiteralPath $Installers) {
        $writable = @(Get-WritableFolders)
        Add-Check 'app folder writable again after the run (lock removed)' ($writable.Count -eq 2) "writable: $($writable -join ', ')"
    }
}
