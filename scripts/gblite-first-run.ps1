# First-run check for the government desktop edition (General Bots Lite).
#
# Does what a brand-new install does on its first start, without the desktop
# shell: lays out an app folder the way the installer does, runs
# `botserver --provision-stack` on an EMPTY stack with the arguments, working
# directory and environment the first-run wizard gives it, downloads forbidden,
# then starts the server once on that stack. Whatever the base app needs and
# the payload does not carry is a failure: that is the point of the check.
#
# Usage (Windows PowerShell 5.1 or PowerShell 7, from the repository root):
#   scripts\gblite-first-run.ps1 -Binary target\release\botserver.exe `
#       -Payload botserver\botserver-installers -Stack $env:TEMP\gb-first-run
#   -ReadOnlyApp     lock the app folder, as a per-machine install is for the
#                    person who runs the app (Program Files)
#   -ProductFile ''  leave the product file out (shows what a provisioner that
#                    cannot find it does)
#
# Needs ports 8200, 5432, 9100, 9101 and 8080 free, and refuses a stack folder
# that is not empty. It stops only what it started (programs whose file is
# under the stack folder or the staged app folder) and removes both folders.
# One line per check (PASS/FAIL); exit code 1 when any check failed.
#
# Where each step comes from (path:line when written):
#   app folder    botapp/tauri.gblite.conf.json:13-24 (.product.gblite, installers/
#                 and botserver.exe side by side); the provisioner finds the
#                 product file beside its own executable, the stack has none
#                 yet (botserver/crates/botcore/src/product/load.rs:124-178)
#   provisioning  botapp/src/desktop/wizard/provision/components.rs:101-159
#                 (program, --provision-stack, cwd, environment), :52-54 and
#                 :232 (last event), :241 (30 s for the exit), :257 (exit code)
#   runtime DLLs  botapp/src/desktop/sidecar.rs:83-116
#   offline       GB_DESKTOP_ALLOW_DOWNLOAD=0 is what the wizard passes under
#                 GB_DESKTOP_OFFLINE=1 (components.rs:150-157); the installer
#                 then refuses a download (botcore package_manager
#                 facade/install_system_packages.rs:91-98)
#   plan          botserver/src/main_module/wizard_provision/component_plan.rs:27-79
#                 and :119-132 (texts of a switched-off service)
#   server start  botapp/src/desktop/sidecar.rs:160-164 (product file into the
#                 stack), :253-279 (--noconsole, cwd, environment)
#   checks        health commands and ports of each service:
#                 botserver/crates/botcorepkg/src/installer_regs/register_secrets.rs:130,
#                 register_drive.rs:42, :119, :133, register_databases.rs:8,
#                 register_directory.rs:8; `services_off` in /api/product:
#                 botcore/src/product/get_product_name.rs:145; built-in cache
#                 line: botserver/src/main_module/cache.rs:200
# Not checked: the `.initialized` marker. The desktop shell writes it when the
# wizard ends (botapp/src/desktop/wizard/handoff.rs:194), not the provisioner.

param(
    [string]$Binary = 'target\release\botserver.exe',
    [string]$Payload = 'botserver\botserver-installers',
    [string]$Stack = '',
    [string]$ProductFile = 'botserver\.product.gblite',
    [string]$Evidence = 'first-run-artifacts',
    [int]$ProvisionTimeoutSec = 600,
    # The wizard waits 300 s, restarts the server once and waits 180 s more
    # (components.rs:319, :384); this check starts it once and waits the sum.
    [int]$ServerTimeoutSec = 480,
    [switch]$ReadOnlyApp,
    [switch]$KeepFolders
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Resolve-Full([string]$Path) { return [IO.Path]::GetFullPath([IO.Path]::Combine((Get-Location).Path, $Path)) }
if (-not $Stack) {
    $tempRoot = $env:RUNNER_TEMP; if (-not $tempRoot) { $tempRoot = $env:TEMP }
    $Stack = Join-Path $tempRoot 'gblite-first-run-stack'
}
$Stack = (Resolve-Full $Stack).TrimEnd('\')
$AppDir = "$Stack-app"; $Installers = "$AppDir\installers"; $AppBinary = "$AppDir\botserver.exe"
$Binary = Resolve-Full $Binary; $Payload = Resolve-Full $Payload; $Evidence = Resolve-Full $Evidence
if ($ProductFile) { $ProductFile = Resolve-Full $ProductFile }
$Api = 'http://127.0.0.1:8080/api/product'

# The services of the base app, and the ones the product file switches off.
$Base = @(
    @{ Id = 'vault'; Label = 'secrets store'; Program = 'bin\vault\bao.exe'; Port = 8200 },
    @{ Id = 'tables'; Label = 'database'; Program = 'bin\tables\bin\postgres.exe'; Port = 5432 },
    @{ Id = 'drive'; Label = 'file store'; Program = 'bin\drive\minio.exe'; Port = 9100 }
)
$Off = @(
    @{ Id = 'cache'; Ports = @(6379) },
    @{ Id = 'vector_db'; Ports = @(6333, 6334) },
    @{ Id = 'directory'; Ports = @(8300) }
)
$OffPrograms = '^(valkey|redis|qdrant|zitadel)'
$NeedFree = @(8200, 5432, 9100, 9101, 8080)
$RuntimeDlls = 'libpq.dll', 'libcrypto-3-x64.dll', 'libssl-3-x64.dll', 'libiconv-2.dll', 'libintl-9.dll',
    'libwinpthread-1.dll', 'liblz4.dll', 'libzstd.dll', 'zlib1.dll'
# Variables a new machine does not have and the server would obey: the two the
# wizard removes, then the secrets address and token, tenant, services switched
# back on, dotenv switch, stack alias, external cache and database password.
$Cleared = 'DATABASE_URL', 'PG_URL', 'VAULT_ADDR', 'VAULT_TOKEN', 'VAULT_CACERT', 'GB_TENANT', 'GB_SERVICES_ON',
    'GBO_NO_DOTENV', 'GBO_STACK_PATH', 'CACHE_URL', 'REDIS_URL', 'VALKEY_URL', 'BOOTSTRAP_DB_PASSWORD'
# The wizard always starts on the free engine (botui/ui/wizard/wizard-state.js:63-86).
$Wizard = @{
    GB_WIZARD_WORKSPACE = "$Stack\workspace"; GB_WIZARD_MODEL_MODE = 'free'; GB_WIZARD_LLM_PROVIDER = 'kilo'
    GB_WIZARD_LLM_URL = 'https://api.kilo.ai/api/gateway/chat/completions'
    GB_WIZARD_LLM_MODEL = 'nvidia/nemotron-3-ultra-550b-a55b:free'; GB_WIZARD_LLM_KEY = $null
    GB_WIZARD_SKIP_AGENTIC_WORK = '0'
}
# A file name that belongs to another system (the cache has no Windows build:
# botserver/3rdparty.toml `[components.cache]` has no `url_win`).
$OtherSystem = '(?i)[\w.\-]*(linux|jammy|ubuntu|musl|darwin|macos)[\w.\-]*'
$FetchLine = '(?i)(Downloading |Offline mode:|Expected file at|could not be installed)'

$script:checks = New-Object System.Collections.ArrayList
$script:envBackup = @{}
$script:owned = $false; $script:locked = $false; $script:staged = $false; $script:provisioned = $false
$script:bundled = @(); $script:skipPorts = @()
. (Join-Path $PSScriptRoot 'gblite-first-run-lib.ps1')

function Invoke-Preflight {
    $ok = Test-Path -LiteralPath $Binary -PathType Leaf
    Add-Check 'server binary found' $ok $Binary
    $files = @(); if (Test-Path -LiteralPath $Payload -PathType Container) { $files = @(Get-ChildItem -LiteralPath $Payload -File) }
    $size = 0; foreach ($file in $files) { $size += $file.Length }
    Add-Check 'payload folder found' ($files.Count -gt 0) ('{0}: {1} files, {2:N0} MB' -f $Payload, $files.Count, ($size / 1MB))
    $ok = $ok -and $files.Count -gt 0
    if ($ProductFile) {
        $found = Test-Path -LiteralPath $ProductFile -PathType Leaf
        $listed = @()
        if ($found) {
            $line = @(Get-Content -LiteralPath $ProductFile | Where-Object { $_ -match '^\s*services_off\s*=' }) | Select-Object -Last 1
            if ($line) { $listed = @(($line -split '=', 2)[1] -split ',' | ForEach-Object { $_.Trim().ToLower() } | Where-Object { $_ }) }
        }
        $notOff = @($Off | Where-Object { $listed -notcontains $_.Id } | ForEach-Object { $_.Id })
        Add-Check 'product file switches off cache, vector_db and directory' ($found -and $notOff.Count -eq 0) "services_off=$($listed -join ',') in $ProductFile"
        $ok = $ok -and $found
    } else { Add-Note 'no product file beside the server in this run (-ProductFile empty)' }
    $used = (Test-Path -LiteralPath $AppDir) -or ((Test-Path -LiteralPath $Stack) -and @(Get-ChildItem -LiteralPath $Stack -Force).Count -gt 0)
    # A `.product` one level above the stack would be read first (load.rs:130),
    # and a path with "botserver-installers" in it is copied from instead of
    # moved out of (botcore package_manager facade/install_binary.rs:20-24).
    $shadow = (Test-Path -LiteralPath (Join-Path (Split-Path $Stack -Parent) '.product')) -or ($Installers -match 'botserver-installers')
    Add-Check 'stack folder is new or empty and stands like a real one' (-not $used -and -not $shadow) $Stack
    $ok = $ok -and -not $used -and -not $shadow
    $busy = @()
    for ($try = 0; $try -lt 20; $try++) {
        $busy = @($NeedFree | Where-Object { Test-Port $_ })
        if ($busy.Count -eq 0) { break }
        Start-Sleep -Seconds 1
    }
    $detail = $NeedFree -join ', '
    if ($busy.Count -gt 0) { $detail = 'in use: ' + (@($busy | ForEach-Object { "$_ by $((Get-PortOwner $_).Name)" }) -join ', ') }
    Add-Check 'ports of the base services and of the server are free' ($busy.Count -eq 0) $detail
    if (-not ($ok -and $busy.Count -eq 0)) { return }
    $script:skipPorts = @($Off | ForEach-Object { $_.Ports } | Where-Object { Test-Port $_ })
    if ($script:skipPorts.Count -gt 0) { Add-Note "ports $($script:skipPorts -join ', ') were already in use before the run and are left out of the checks" }

    New-Item -ItemType Directory -Force $Stack, "$Stack\logs", "$Stack\workspace", $Installers | Out-Null
    $script:owned = $true
    Copy-Item -LiteralPath $Binary -Destination $AppBinary
    if ($ProductFile) { Copy-Item -LiteralPath $ProductFile -Destination "$AppDir\.product.gblite" }
    foreach ($file in $files) { Copy-Item -LiteralPath $file.FullName -Destination $Installers }
    $script:bundled = @($files | ForEach-Object { $_.Name })
    $absent = @()
    foreach ($dll in $RuntimeDlls) {
        if (Test-Path -LiteralPath "$Installers\$dll") { Copy-Item -LiteralPath "$Installers\$dll" -Destination "$Stack\$dll" } else { $absent += $dll }
    }
    $detail = "botserver.exe, .product.gblite, installers\ with $($files.Count) files"
    if ($absent.Count -gt 0) { $detail = "PostgreSQL runtime files missing from the payload: $($absent -join ', ')" }
    Add-Check 'app folder laid out as the installer does, runtime files beside the stack' ($absent.Count -eq 0) $detail
    if ($absent.Count -gt 0) { return }
    if ($ReadOnlyApp) {
        & icacls $AppDir /deny '*S-1-1-0:(OI)(CI)(DE,DC,WD,AD,WEA,WA)' /Q | Out-Null
        $script:locked = $true
        $writable = $true
        try { [IO.File]::WriteAllText("$Installers\write-test.tmp", 'x') } catch { $writable = $false }
        Add-Check 'app folder is read-only, as a per-machine install is' (-not $writable) $AppDir
    }
    $script:staged = $true
}

function Invoke-Provisioning {
    foreach ($name in $Cleared) { Set-ChildEnv $name $null }
    foreach ($name in $Wizard.Keys) { Set-ChildEnv $name $Wizard[$name] }
    Set-ChildEnv 'PATH' "$Stack;$env:PATH"
    Set-ChildEnv 'BOTSERVER_STACK_PATH' $Stack
    Set-ChildEnv 'BOTSERVER_INSTALLERS_PATH' $Installers
    Set-ChildEnv 'GB_DESKTOP_ALLOW_DOWNLOAD' '0'
    $out = "$Evidence\provision.out.log"; $err = "$Evidence\provision.err.log"
    $started = Get-Date
    $child = Start-Process -FilePath $AppBinary -ArgumentList '--provision-stack' -WorkingDirectory $Stack -PassThru `
        -WindowStyle Hidden -RedirectStandardOutput $out -RedirectStandardError $err
    $null = $child.Handle
    # The services the provisioner starts keep its output open after it ends,
    # so the last event ends the wait, as in the wizard (components.rs:225-234).
    $events = @(); $shown = 0; $timedOut = $false; $settled = $false
    while ($true) {
        $events = @(Get-WizardEvents (Read-Lines $out))
        for (; $shown -lt $events.Count; $shown++) {
            Write-Host ('      {0,6:N1} s  {1}: {2} - {3}' -f ($events[$shown].elapsed_ms / 1000), $events[$shown].id, $events[$shown].state, $events[$shown].message)
        }
        if (@($events | Where-Object { $_.state -eq 'failed' -or ($_.id -eq 'components' -and $_.state -eq 'ready') }).Count -gt 0) { break }
        if ($child.HasExited) { if ($settled) { break }; $settled = $true; Start-Sleep -Seconds 2; continue }
        if (((Get-Date) - $started).TotalSeconds -ge $ProvisionTimeoutSec) { $timedOut = $true; break }
        Start-Sleep -Seconds 1
    }
    if ($timedOut) { Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue }
    [void]$child.WaitForExit(30000)
    $exit = $null; if ($child.HasExited) { $exit = $child.ExitCode }
    $seconds = [int]((Get-Date) - $started).TotalSeconds
    $text = @(Read-Lines $out) + @(Read-Lines $err)
    $failed = @($events | Where-Object { $_.state -eq 'failed' })
    $allReady = @($events | Where-Object { $_.id -eq 'components' -and $_.state -eq 'ready' }).Count -gt 0
    # A provisioner still running 30 s after its last event does not stop the
    # wizard (components.rs:241-246); a known exit code must be 0 (:257).
    $exitOk = ($exit -eq 0) -or ($null -eq $exit -and -not $timedOut)
    Add-Check "provisioning ended inside $ProvisionTimeoutSec s" (-not $timedOut) "$seconds s"
    $detail = 'All components are ready'
    if ($failed.Count -gt 0) { $detail = '{0}: {1}' -f $failed[0].id, $failed[0].message }
    elseif (-not $allReady) { $detail = 'no "components ready" event; ' + $events.Count + ' events seen' }
    Add-Check 'provisioning completed: every component ready' ($allReady -and $failed.Count -eq 0) $detail
    $detail = "exit code $exit"; if ($null -eq $exit) { $detail = 'still running 30 s after its last event' }
    Add-Check 'provisioning process ended without an error' $exitOk $detail
    $script:provisioned = $allReady -and $failed.Count -eq 0 -and $exitOk -and -not $timedOut
    if (-not $script:provisioned) {
        Show-Tail 'provisioning output' $out; Show-Tail 'provisioning errors' $err
        $last = $events | Select-Object -Last 1
        $where = 'vault'; if ($failed.Count -gt 0) { $where = $failed[0].id } elseif ($last) { $where = $last.id }
        if (Test-Path -LiteralPath "$Stack\logs\$where") {
            Get-ChildItem -LiteralPath "$Stack\logs\$where" -File | ForEach-Object { Show-Tail "stack log $where\$($_.Name)" $_.FullName 30 }
        }
    }
    foreach ($service in $Base) {
        $last = $events | Where-Object { $_.id -eq $service.Id } | Select-Object -Last 1
        $ready = $last -and $last.state -eq 'ready' -and $last.message -notmatch '^Skipped'
        $detail = 'not reached'; if ($last) { $detail = 'last event: {0} - {1}' -f $last.state, $last.message }
        Add-Check "provisioning made the $($service.Label) ready ($($service.Id))" ([bool]$ready) $detail
    }
    # A switched-off service is reported ready at once with one of two fixed
    # texts and is never installed or started (component_plan.rs:119-132). Only
    # a loaded product configuration gives that, so it stands for the log line
    # when the provisioner prints none (it sets no logger: main_body.rs:25-27).
    $attempts = @(); $said = 0; $reached = 0
    foreach ($service in $Off) {
        $mine = @($events | Where-Object { $_.id -eq $service.Id })
        if ($mine.Count -gt 0) { $reached++ }
        $attempts += @($mine | Where-Object { $_.state -eq 'starting' -or $_.state -eq 'failed' } | ForEach-Object { '{0}: {1}' -f $_.id, $_.message })
        $last = $mine | Select-Object -Last 1
        if ($last -and $last.message -match '^(Built into the assistant|Not used in this edition)$') { $said++ }
    }
    $loaded = @($text | Where-Object { $_ -match 'Loaded product configuration' }) | Select-Object -First 1
    $notFound = @($text | Where-Object { $_ -match 'No \.product file found' }).Count -gt 0
    $against = $notFound -or $attempts.Count -gt 0
    $for = [bool]$loaded -or ($said -gt 0 -and $said -eq $reached) -or ($reached -eq 0 -and $script:provisioned)
    $detail = "$said of $($Off.Count) switched-off services reported as such"; if ($loaded) { $detail = $loaded.Trim() }
    if ($notFound) { $detail = 'No .product file found, using default configuration' }
    elseif ($attempts.Count -gt 0) { $detail = "services_off was empty for the provisioner: $($attempts[0])" }
    if ($against -or $for) { Add-Check 'provisioning loaded the product configuration' ($for -and -not $against) $detail }
    else { Add-Note 'provisioning stopped before any switched-off service: no sign yet of the product configuration' }
    Add-Check 'provisioning never installed or started cache, vector store or identity provider' ($attempts.Count -eq 0) ($attempts -join ' | ')
    $now = @(); if (Test-Path -LiteralPath $Installers) { $now = @(Get-ChildItem -LiteralPath $Installers -File | ForEach-Object { $_.Name }) }
    # "Downloading <address>" is printed before a fetch and "Offline mode: ..."
    # is the refusal of one (facade/install_system_packages.rs:91-100); a file
    # that is new in the installers folder was fetched into it
    # (package_manager/cache.rs:58-59).
    $fetched = @($now | Where-Object { $script:bundled -notcontains $_ } | ForEach-Object { "new file $_" })
    $fetched += @($text | Where-Object { $_ -match '^\s*Downloading\s' -or $_ -match 'Offline mode:' } |
            ForEach-Object { ($_ -replace '^.*?(Offline mode:|Downloading\s)', '$1' -replace '","elapsed_ms.*$', '' -replace '\\\\', '\').Trim() })
    Add-Check 'provisioning needed nothing outside the payload' ($fetched.Count -eq 0) (@($fetched | Select-Object -Unique) -join ' | ')
    $other = @($text | Where-Object { $_ -match $FetchLine -and $_ -match $OtherSystem })
    $detail = ''; if ($other.Count -gt 0) { $detail = [regex]::Match($other[0], $OtherSystem).Value }
    Add-Check 'provisioning asked for no archive of another system' ($other.Count -eq 0) $detail
    Add-Check 'secrets store initialised' (Test-Path -LiteralPath "$Stack\conf\vault\init.json") 'conf\vault\init.json'
    $moved = @($script:bundled | Where-Object { $now -notcontains $_ })
    if ($moved.Count -gt 0) { Add-Note "the first run took these files out of the installers folder: $($moved -join ', ')" }
}

function Test-BaseServices {
    foreach ($service in $Base) {
        Add-Check "$($service.Label) installed" (Test-Path -LiteralPath (Join-Path $Stack $service.Program) -PathType Leaf) $service.Program
        $up = $false; $answer = ''
        if ($service.Id -eq 'tables') {
            $code = Invoke-Tool "$Stack\bin\tables\bin\pg_isready.exe" @('-h', 'localhost', '-p', '5432', '-t', '5') 20
            $up = $code -eq 0; $answer = "pg_isready on 5432: exit code $code"
            if ($null -eq $code) { $answer = 'pg_isready on 5432 could not run or gave no answer in 20 s' }
        } else {
            $url = 'http://127.0.0.1:8200/v1/sys/health?standbyok=true'
            if ($service.Id -eq 'drive') { $url = 'http://127.0.0.1:9100/minio/health/live' }
            $status = Get-HttpStatus $url
            $up = $status -ge 200 -and $status -lt 400; $answer = "$url -> $status"
        }
        $owner = Get-PortOwner $service.Port
        if ($owner -and $owner.ExecutablePath -and -not (Test-Under $owner.ExecutablePath $Stack)) {
            $up = $false; $answer = "port $($service.Port) is held by $($owner.ExecutablePath), not by this stack"
        }
        Add-Check "$($service.Label) answers" $up $answer
    }
}

function Test-OffServices([string]$When) {
    $files = @(); $open = @()
    foreach ($service in $Off) {
        $folder = "$Stack\bin\$($service.Id)"
        if (Test-Path -LiteralPath $folder) { $files += @(Get-ChildItem -LiteralPath $folder -Recurse -File -Force | ForEach-Object { $_.FullName }) }
        $open += @($service.Ports | Where-Object { $script:skipPorts -notcontains $_ -and (Test-Port $_) })
    }
    if (Test-Path -LiteralPath "$Stack\bin") {
        $files += @(Get-ChildItem -LiteralPath "$Stack\bin" -Recurse -File -Force | Where-Object { $_.Name -match $OffPrograms } | ForEach-Object { $_.FullName })
    }
    $files = @($files | Select-Object -Unique | ForEach-Object { $_.Substring($Stack.Length + 1) })
    $running = @(Get-Owned | Where-Object { $_.Name -match $OffPrograms } | ForEach-Object { $_.Name })
    Add-Check "cache, vector store and identity provider not installed $When" ($files.Count -eq 0) ($files -join ', ')
    Add-Check "their ports are closed $When" ($open.Count -eq 0) ($open -join ', ')
    Add-Check "none of them is running $When" ($running.Count -eq 0) ($running -join ', ')
}

function Invoke-Server {
    foreach ($name in $Wizard.Keys) { Set-ChildEnv $name $null }
    Set-ChildEnv 'AGENT_ALLOW_UNREGISTERED_IDENTITIES' '1'
    # Agent mode variables, set only when the payload carries the wrapper
    # (botapp/src/desktop/sidecar/agent_env.rs:19-40).
    if ((Test-Path -LiteralPath "$Installers\incus.exe") -and -not $env:AGENT_INCUS_BIN) {
        Set-ChildEnv 'AGENT_INCUS_BIN' "$Installers\incus.exe"
        if (-not $env:GBO_WSL_DISTRO) { Set-ChildEnv 'GBO_WSL_DISTRO' 'GeneralBotsLite' }
        if (-not $env:GBO_AGENT_RUNTIME_MARKER) { Set-ChildEnv 'GBO_AGENT_RUNTIME_MARKER' "$Stack\agent-runtime\ready.json" }
    }
    if (Test-Path -LiteralPath "$AppDir\.product.gblite") { Copy-Item -LiteralPath "$AppDir\.product.gblite" -Destination "$Stack\.product.gblite" -Force }
    $out = "$Evidence\server.out.log"; $err = "$Evidence\server.err.log"
    $started = Get-Date
    $server = Start-Process -FilePath $AppBinary -ArgumentList '--noconsole' -WorkingDirectory $Stack -PassThru `
        -WindowStyle Hidden -RedirectStandardOutput $out -RedirectStandardError $err
    $null = $server.Handle
    $ready = $false
    while (-not $ready -and -not $server.HasExited -and ((Get-Date) - $started).TotalSeconds -lt $ServerTimeoutSec) {
        Start-Sleep -Seconds 2
        $ready = (Get-HttpStatus $Api) -eq 200
    }
    $seconds = [int]((Get-Date) - $started).TotalSeconds
    $detail = "after $seconds s"
    if (-not $ready) { $detail = "no answer in $seconds s"; if ($server.HasExited) { $detail = "the server ended with exit code $($server.ExitCode) after $seconds s" } }
    Add-Check 'server answered /api/product' $ready $detail
    if ($ready) {
        $body = (Invoke-WebRequest -Uri $Api -UseBasicParsing -TimeoutSec 10).Content
        Set-Content -LiteralPath "$Evidence\api-product.json" -Value $body
        $got = @(($body | ConvertFrom-Json).services_off | ForEach-Object { [string]$_ } | Sort-Object)
        $want = @($Off | ForEach-Object { $_.Id } | Sort-Object)
        Add-Check '/api/product lists the services switched off' (($got -join ',') -eq ($want -join ',')) "services_off=$($got -join ',')"
    }
    $log = @(); $cacheLine = $false
    for ($try = 0; $try -lt 10; $try++) {
        $log = @(Read-Lines $out) + @(Read-Lines $err)
        $cacheLine = @($log | Where-Object { $_ -match 'Cache initialized - built-in cache' }).Count -gt 0
        if ($cacheLine -or -not $ready) { break }
        Start-Sleep -Seconds 2
    }
    $trouble = @($log | Where-Object { $_ -match 'Built-in cache (could not start|did not answer|check failed)' }) | Select-Object -First 1
    Add-Check 'server log has the built-in cache line' $cacheLine ("$trouble".Trim())
    $loaded = @($log | Where-Object { $_ -match 'Loaded product configuration' }) | Select-Object -First 1
    Add-Check 'server log shows the product configuration was loaded' ([bool]$loaded) ("$loaded".Trim())
    $panic = @($log | Where-Object { $_ -match 'panicked at' }) | Select-Object -First 1
    Add-Check 'server did not panic' (-not $panic) ("$panic".Trim())
    $other = @($log | Where-Object { $_ -match $FetchLine -and $_ -match $OtherSystem })
    $detail = ''; if ($other.Count -gt 0) { $detail = [regex]::Match($other[0], $OtherSystem).Value }
    Add-Check 'server asked for no archive of another system' ($other.Count -eq 0) $detail
    $offline = @($log | Where-Object { $_ -match 'Offline mode:' })
    if ($offline.Count -gt 0) { Add-Note "the server wanted $($offline.Count) thing(s) the payload does not carry; first: $($offline[0].Trim())" }
    if (-not $ready) { Show-Tail 'server output' $out; Show-Tail 'server errors' $err }
}

try {
    New-Item -ItemType Directory -Force $Evidence | Out-Null
    Remove-Item -LiteralPath "$Evidence\stack-logs" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "gbgov first run: server $Binary, payload $Payload, stack $Stack"
    Invoke-Preflight
    if ($script:staged) {
        Invoke-Provisioning
        Test-BaseServices
        Test-OffServices 'after provisioning'
        if ($script:provisioned) {
            Invoke-Server
            Test-OffServices 'after the first start of the server'
        } else {
            Add-Note 'the server was not started: the wizard stops when provisioning fails (provision/mod.rs:60-61)'
            if ($ReadOnlyApp) { Add-Note 'the app folder was read-only in this run; the run without -ReadOnlyApp tells whether that is the cause' }
        }
    }
} catch {
    Add-Check 'the check ran to its end' $false "$($_.Exception.Message) (line $($_.InvocationInfo.ScriptLineNumber))"
} finally {
    if ($script:owned) {
        try { Stop-Everything } catch { Add-Note "stopping failed: $($_.Exception.Message)" }
        $left = @(Get-Owned | ForEach-Object { $_.Name })
        Add-Check 'everything this check started is stopped' ($left.Count -eq 0) ($left -join ', ')
        $held = @($NeedFree | Where-Object { Test-Port $_ })
        Add-Check 'ports are free again' ($held.Count -eq 0) ($held -join ', ')
        if (Test-Path -LiteralPath "$Stack\logs") { Copy-Item -LiteralPath "$Stack\logs" -Destination "$Evidence\stack-logs" -Recurse -Force -ErrorAction SilentlyContinue }
    }
    foreach ($name in @($script:envBackup.Keys)) { Set-Env $name $script:envBackup[$name] }
    if ($script:locked) { & icacls $AppDir /remove:d '*S-1-1-0' /Q | Out-Null }
    if ($script:owned -and -not $KeepFolders) {
        foreach ($folder in $Stack, $AppDir) {
            for ($try = 0; $try -lt 5 -and (Test-Path -LiteralPath $folder); $try++) {
                Remove-Item -LiteralPath $folder -Recurse -Force -ErrorAction SilentlyContinue
                if (Test-Path -LiteralPath $folder) { Start-Sleep -Seconds 2 }
            }
        }
        Add-Check 'temporary folders removed' (-not (Test-Path -LiteralPath $Stack) -and -not (Test-Path -LiteralPath $AppDir)) "$Stack ; $AppDir"
    }
}

$failed = @($script:checks | Where-Object { -not $_.Ok })
Write-Host ''
Write-Host "gbgov first run: $($script:checks.Count) checks, $($failed.Count) failed"
$script:checks | ForEach-Object { $_.Line } | Set-Content -LiteralPath "$Evidence\first-run-summary.txt"
if ($env:GITHUB_STEP_SUMMARY) {
    $title = 'First run on an empty stack'; if ($ReadOnlyApp) { $title = "$title, app folder read-only" }
    @("### $title", '', '```') + @($script:checks | ForEach-Object { $_.Line }) + @('```') | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
}
if ($failed.Count -gt 0) {
    if ($env:GITHUB_ACTIONS -eq 'true') { Write-Host "::warning title=gbgov first run::$($failed.Count) check(s) failed: $(@($failed | ForEach-Object { $_.Name }) -join '; ')" }
    exit 1
}
exit 0
