# Server starts of scripts/gblite-first-run.ps1 (dot-sourced by it; not run
# alone). The first start runs on the stack the provisioner has just made.
# The server is then stopped with Ctrl+C, every program of the stack is
# stopped as a restart of the computer would stop it, and the server starts a
# second time. Nothing is handed to the server that a new install lacks: no
# secret, no token, no folder, no development switch (the caller has removed
# those variables, and the server gets only what the desktop shell sets).
#
# What each check reads (path:line when written):
#   environment   botapp/src/desktop/sidecar.rs:253-290, sidecar/agent_env.rs:19-40
#   posture       botserver/src/main_module/server/run_axum_server.rs:153
#   CSRF secret   run_axum_server.rs:192 (source; the names are in
#                 botserver/crates/botsecurity-auth/src/csrf/secret.rs:56-61),
#                 :197 (generated and persisted), and
#                 botserver/crates/botcoresecrets/src/sync.rs:196 (persisted by
#                 the background retry)
#   refusal       run_axum_server.rs:34, :291; main_module/listen_impl.rs:87
#   credentials   botserver/crates/botcorepkg/src/installer_vault/secret_location.rs:11-14
#                 (folder inside the stack on Windows), secret_store.rs:28-30 (names)
#   unsealed      Vault's own /v1/sys/seal-status
#   token         botserver/crates/botcorepkg/src/installer_vault/token_recovery.rs:107
#   Ctrl+C        botserver/src/main_module/shutdown.rs:13-45

$Credentials = 'vault-unseal-keys', 'vault-token'
$CredentialDir = "$Stack\conf\vault\credentials"

# The first line of the server logs that matches, waiting up to $WaitSec.
function Find-Line([string[]]$Paths, [string]$Pattern, [int]$WaitSec = 0) {
    $deadline = (Get-Date).AddSeconds($WaitSec)
    while ($true) {
        foreach ($path in $Paths) {
            $hit = @(Read-Lines $path | Where-Object { $_ -match $Pattern }) | Select-Object -First 1
            if ($hit) { return "$hit".Trim() }
        }
        if ((Get-Date) -ge $deadline) { return '' }
        Start-Sleep -Seconds 2
    }
}

function Get-SealStatus {
    foreach ($url in 'http://127.0.0.1:8200/v1/sys/seal-status', 'https://127.0.0.1:8200/v1/sys/seal-status') {
        $options = @{ Uri = $url; TimeoutSec = 5 }
        if ($url.StartsWith('https') -and $PSVersionTable.PSVersion.Major -ge 6) { $options.SkipCertificateCheck = $true }
        try { return Invoke-RestMethod @options } catch { continue }
    }
    return $null
}

function Get-CredentialHashes {
    $hashes = @{}
    foreach ($name in $Credentials) {
        $file = Get-Item -LiteralPath "$CredentialDir\$name" -Force -ErrorAction SilentlyContinue
        if (-not $file -or $file.Length -eq 0) { continue }
        try { $hashes[$name] = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256 -ErrorAction Stop).Hash }
        catch { $hashes[$name] = "unreadable: $($_.Exception.Message)" }
    }
    return $hashes
}

function Invoke-Server([string]$Boot) {
    foreach ($name in $Wizard.Keys) { Set-ChildEnv $name $null }
    Set-ChildEnv 'AGENT_ALLOW_UNREGISTERED_IDENTITIES' '1'
    # Agent mode variables, set only when the payload carries the wrapper
    # (botapp/src/desktop/sidecar/agent_env.rs:19-40).
    if (Test-Path -LiteralPath "$Installers\incus.exe") {
        Set-ChildEnv 'AGENT_INCUS_BIN' "$Installers\incus.exe"
        Set-ChildEnv 'GBO_WSL_DISTRO' 'GeneralBotsLite'
        Set-ChildEnv 'GBO_AGENT_RUNTIME_MARKER' "$Stack\agent-runtime\ready.json"
        Set-ChildEnv 'AGENT_VM_BRIDGE' 'gblbr0'
    }
    if (Test-Path -LiteralPath "$AppDir\.product.gblite") { Copy-Item -LiteralPath "$AppDir\.product.gblite" -Destination "$Stack\.product.gblite" -Force }
    $suffix = ''; if ($Boot -ne 'first') { $suffix = "-$Boot" }
    $out = "$Evidence\server$suffix.out.log"; $err = "$Evidence\server$suffix.err.log"
    $script:serverLogs = @($out, $err)
    Enable-CtrlC
    $started = Get-Date
    # Start-Process with redirected output gives the server a console of its
    # own (hidden), as the shell does (no_window.rs), so Ctrl+C reaches it alone.
    $server = Start-Process -FilePath $AppBinary -ArgumentList '--noconsole' -WorkingDirectory $Stack -PassThru `
        -WindowStyle Hidden -RedirectStandardOutput $out -RedirectStandardError $err
    $null = $server.Handle
    $script:server = $server
    $ready = $false
    while (-not $ready -and -not $server.HasExited -and ((Get-Date) - $started).TotalSeconds -lt $ServerTimeoutSec) {
        Start-Sleep -Seconds 2
        $ready = (Get-HttpStatus $Api) -eq 200
    }
    $script:serverReady = $ready
    $seconds = [int]((Get-Date) - $started).TotalSeconds
    $detail = "after $seconds s"
    if (-not $ready) { $detail = "no answer in $seconds s"; if ($server.HasExited) { $detail = "the server ended with exit code $($server.ExitCode) after $seconds s" } }
    Add-Check "$Boot start: server answered /api/product" $ready $detail
    if ($ready) {
        $body = (Invoke-WebRequest -Uri $Api -UseBasicParsing -TimeoutSec 10).Content
        Set-Content -LiteralPath "$Evidence\api-product$suffix.json" -Value $body
        $got = @(($body | ConvertFrom-Json).services_off | ForEach-Object { [string]$_ } | Sort-Object)
        $want = @($Off | ForEach-Object { $_.Id } | Sort-Object)
        Add-Check "$Boot start: /api/product lists the services switched off" (($got -join ',') -eq ($want -join ',')) "services_off=$($got -join ',')"
    }
    $wait = 0; if ($ready) { $wait = 20 }
    $cache = Find-Line $script:serverLogs 'Cache initialized - built-in cache' $wait
    Add-Check "$Boot start: server log has the built-in cache line" ([bool]$cache) (Find-Line $script:serverLogs 'Built-in cache (could not start|did not answer|check failed)')
    $loaded = Find-Line $script:serverLogs 'Loaded product configuration'
    Add-Check "$Boot start: server log shows the product configuration was loaded" ([bool]$loaded) $loaded
    $panic = Find-Line $script:serverLogs 'panicked at'
    Add-Check "$Boot start: server did not panic" (-not $panic) $panic
    $other = Find-Line $script:serverLogs "$FetchLine.*$OtherSystem|$OtherSystem.*$FetchLine"
    $detail = ''; if ($other) { $detail = [regex]::Match($other, $OtherSystem).Value }
    Add-Check "$Boot start: server asked for no archive of another system" (-not $other) $detail
    $offline = Find-Line $script:serverLogs 'Offline mode:'
    if ($offline) { Add-Note "the server wanted something the payload does not carry: $offline" }
    Test-Secrets $Boot $ready
    if (-not $ready) { Show-Tail 'server output' $out; Show-Tail 'server errors' $err }
}

function Test-Secrets([string]$Boot, [bool]$Ready) {
    $logs = $script:serverLogs
    $posture = Find-Line $logs 'Security posture: '
    Add-Check "$Boot start: production posture, no development switch" ($posture -match 'Security posture: production \(BOTSERVER_ENV=<unset>\)') $posture
    $refused = Find-Line $logs 'Refusing to start'
    Add-Check "$Boot start: server did not refuse to start" (-not $refused) $refused
    $source = Find-Line $logs 'CSRF secret source: '
    if ($Boot -eq 'first') {
        Add-Check 'first start: CSRF secret generated on this machine' ($source -match 'CSRF secret source: desktop-generated') $source
        $wait = 0; if ($Ready) { $wait = 60 }
        $kept = Find-Line $logs 'CSRF signing secret (generated and )?persisted to Vault' $wait
        $detail = $kept
        if (-not $kept) { $detail = Find-Line $logs 'could not persist it|CSRF secret persist'; if (-not $detail) { $detail = "no ""persisted to Vault"" line in $wait s" } }
        Add-Check 'first start: CSRF secret persisted to Vault' ([bool]$kept) $detail
        $script:credentialHashes = Get-CredentialHashes
        $missing = @($Credentials | Where-Object { -not $script:credentialHashes.ContainsKey($_) })
        $detail = "$($Credentials -join ', ') in conf\vault\credentials"
        if ($missing.Count -gt 0) { $detail = "missing or empty in conf\vault\credentials: $($missing -join ', ')" }
        Add-Check 'first start: unseal keys and server token stored in the stack' ($missing.Count -eq 0) $detail
        return
    }
    Add-Check "$Boot start: CSRF secret read from Vault" ($source -match 'CSRF secret source: vault(\s|$)') $source
    $seal = Get-SealStatus
    $detail = 'Vault did not answer /v1/sys/seal-status'
    if ($seal) { $detail = "initialized=$($seal.initialized) sealed=$($seal.sealed)" }
    $owner = Get-PortOwner 8200
    if ($owner -and $owner.ExecutablePath -and -not (Test-Under $owner.ExecutablePath $Stack)) { $seal = $null; $detail = "port 8200 is held by $($owner.ExecutablePath)" }
    Add-Check "$Boot start: Vault unsealed with the stored keys" ($seal -and $seal.initialized -eq $true -and $seal.sealed -eq $false) $detail
    $renewed = Find-Line $logs 'Scoped Vault token renewed' 20
    $detail = $renewed; if (-not $renewed) { $detail = Find-Line $logs 'Vault token' }
    Add-Check "$Boot start: stored Vault token renewed" ([bool]$renewed) $detail
    $now = Get-CredentialHashes
    $changed = @($Credentials | Where-Object { $now[$_] -ne $script:credentialHashes[$_] })
    if ($changed.Count -gt 0) { Add-Note "the $Boot start rewrote or removed: $($changed -join ', ')" }
}

# A program inherits "ignore Ctrl+C" from the one that starts it, and a
# runner or a terminal often has it set; the server would then never see the
# Ctrl+C of Stop-Server. This process takes Ctrl+C again before it starts the
# server, so the server inherits the default.
function Enable-CtrlC {
    if (-not ('GbFirstRun.Parent' -as [type])) {
        Add-Type -Namespace GbFirstRun -Name Parent -MemberDefinition `
            '[DllImport("kernel32.dll")] public static extern bool SetConsoleCtrlHandler(System.IntPtr handler, bool add);'
    }
    [void][GbFirstRun.Parent]::SetConsoleCtrlHandler([IntPtr]::Zero, $false)
}

# A console control event reaches every program attached to one console, so
# it is sent by a separate PowerShell that leaves its own console, joins the
# server's and ignores the event itself. Exit code 0 when it was sent.
function Send-CtrlC([int]$ProcessId) {
    $members = @(
        '[DllImport("kernel32.dll")] public static extern bool FreeConsole();'
        '[DllImport("kernel32.dll")] public static extern bool AttachConsole(uint processId);'
        '[DllImport("kernel32.dll")] public static extern bool SetConsoleCtrlHandler(System.IntPtr handler, bool add);'
        '[DllImport("kernel32.dll")] public static extern bool GenerateConsoleCtrlEvent(uint ctrlEvent, uint processGroupId);'
    ) -join ' '
    $code = @(
        "Add-Type -Namespace GbFirstRun -Name Console -MemberDefinition '$members'"
        '[void][GbFirstRun.Console]::FreeConsole()'
        "if (-not [GbFirstRun.Console]::AttachConsole($ProcessId)) { exit 2 }"
        '[void][GbFirstRun.Console]::SetConsoleCtrlHandler([IntPtr]::Zero, $true)'
        'if (-not [GbFirstRun.Console]::GenerateConsoleCtrlEvent(0, 0)) { exit 3 }'
        'Start-Sleep -Seconds 1'
        'exit 0'
    ) -join "`n"
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($code))
    $shell = (Get-Process -Id $PID).Path
    return Invoke-Tool $shell @('-NoLogo', '-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded) 60
}

function Stop-Server {
    $server = $script:server
    if (-not $server -or $server.HasExited) {
        $detail = 'the server was not running'
        if ($server) { $detail = "the server had already ended with exit code $($server.ExitCode)" }
        Add-Check 'first start: server stopped on Ctrl+C' $false $detail
        return
    }
    $sent = Send-CtrlC $server.Id
    # The server forces its exit 15 s after the signal (shutdown.rs:54-58).
    $ended = $server.WaitForExit(45000)
    $said = Find-Line $script:serverLogs 'Received SIGINT'
    $detail = "Ctrl+C helper exit code $sent; still running after 45 s"
    if ($ended) { $detail = "exit code $($server.ExitCode); $said" }
    Add-Check 'first start: server stopped on Ctrl+C' ($ended -and [bool]$said) $detail
    if (-not $ended) { Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue; [void]$server.WaitForExit(10000) }
}

# Every program of the stack stops, as when the computer restarts: the
# database through pg_ctl, the others are ended. Returns whether all did.
function Restart-Stack {
    Stop-Everything
    $left = @(Get-Owned | ForEach-Object { $_.Name })
    $held = @()
    for ($try = 0; $try -lt 20; $try++) {
        $held = @($NeedFree | Where-Object { Test-Port $_ })
        if ($held.Count -eq 0) { break }
        Start-Sleep -Seconds 1
    }
    $done = $left.Count -eq 0 -and $held.Count -eq 0
    $detail = 'no program of the stack left, ports free'
    if (-not $done) { $detail = "still running: $($left -join ', '); ports in use: $($held -join ', ')" }
    Add-Check 'every program of the stack stopped before the second start' $done $detail
    return $done
}
