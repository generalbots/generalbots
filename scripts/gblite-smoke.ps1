# Smoke test for the government desktop edition (Chat + Drive profile).
#
# Boots the `gblite` build of botserver against an isolated stack directory and
# a local deterministic mock model, then checks that the product profile is
# Chat + Drive only, that the suite shell is served, that the agent workspace
# is compiled in and routed, and that a chat turn is answered by the mock.
#
# Usage (CI and local):
#   pwsh scripts/gblite-smoke.ps1 -Binary target\debug\botserver.exe
#
# Needs: PostgreSQL reachable through DATABASE_URL, Python on PATH. No request
# leaves the machine: the model endpoint is 127.0.0.1 and the secrets address is
# an unresolvable name, so the server skips its service bootstrap.

param(
    [string]$Binary = 'target\debug\botserver.exe',
    [string]$Evidence = 'smoke-artifacts',
    [int]$Port = 8080,
    [int]$MockPort = 18080
)

$ErrorActionPreference = 'Stop'
$root = (Get-Location).Path
$stack = if ($env:BOTSERVER_STACK_PATH) { $env:BOTSERVER_STACK_PATH } else { Join-Path $env:TEMP 'gblite-smoke-stack' }
$base = "http://127.0.0.1:$Port"
$mockBase = "http://127.0.0.1:$MockPort"

New-Item -ItemType Directory -Force $Evidence, "$stack\conf\system", "$stack\conf\vault" | Out-Null

# Isolated stack: an empty .env plus init.json mark the bootstrap as done, and
# the unresolvable secrets address makes starting the bundled services a no-op.
'' | Set-Content "$stack\.env"
'{}' | Set-Content "$stack\conf\vault\init.json"
'{"issuer_url":"http://127.0.0.1:1","issuer":"http://127.0.0.1:1","client_id":"smoke","client_secret":"smoke","redirect_uri":"/callback","project_id":"default","api_url":"http://127.0.0.1:1","allow_insecure_http":true}' |
    Set-Content "$stack\conf\system\directory_config.json"

# The smoke run has no identity service, so sign-in is switched off in its
# copy of the product file. The shipped file keeps no_security=false.
$product = Get-Content 'botserver\.product.gblite'
if (-not ($product -match '^no_security=false$')) { throw '.product.gblite must ship with no_security=false' }
$product -replace '^no_security=false$', 'no_security=true' | Set-Content "$stack\.product.gblite"

$env:BOTSERVER_STACK_PATH = $stack
$env:VAULT_ADDR = 'https://vault.smoke.invalid:8200'
$env:BOTSERVER_DISABLE_TLS = '1'
$env:BOT_TEMPLATES_REPO = 'https://example.invalid/templates.git'
$env:GBO_NO_DOTENV = '1'
$env:LLM_URL = "$mockBase/v1/chat/completions"
$env:LLM_API_URL = "$mockBase/v1"
$env:OPENAI_API_BASE = "$mockBase/v1"
$env:LLM_KEY = 'test-mock-key'
$env:LLM_API_KEY = 'test-mock-key'
$env:OPENAI_API_KEY = 'test-mock-key'
$env:LLM_MODEL = 'mock-gpt'
$env:LLM_PROVIDER = 'openai'
$env:MOCK_HOST = '127.0.0.1'
$env:MOCK_PORT = "$MockPort"
$env:MOCK_MODEL = 'mock-gpt'
$env:MOCK_REPLY = 'Hello from mock LLM.'
$env:MOCK_MODE = 'answer'
$env:PYTHONUNBUFFERED = '1'
$env:PYTHONIOENCODING = 'utf-8'

$procs = @()
function Stop-All { foreach ($p in $script:procs) { if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } } }
function Show-Logs {
    foreach ($f in 'botserver-smoke.log', 'botserver-smoke.log.err', 'mock-llm.log.err') {
        Write-Host "=== $f (tail) ==="
        Get-Content (Join-Path $root $f) -Tail 120 -ErrorAction SilentlyContinue
    }
}
function Get-Json([string]$Url) {
    $r = Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 10
    if ($r.StatusCode -ne 200) { throw "$Url expected 200, got $($r.StatusCode)" }
    return $r
}

try {
    $mock = Start-Process -FilePath 'python' -ArgumentList @('-u', (Join-Path $root 'scripts\mock_llm.py')) -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput (Join-Path $root 'mock-llm.log') -RedirectStandardError (Join-Path $root 'mock-llm.log.err')
    $procs += $mock
    $mockReady = $false
    for ($i = 0; $i -lt 30 -and -not $mockReady; $i++) {
        Start-Sleep -Milliseconds 500
        try { if ((Invoke-WebRequest -Uri "$mockBase/health" -UseBasicParsing -TimeoutSec 2).StatusCode -eq 200) { $mockReady = $true } } catch {}
    }
    if (-not $mockReady) { throw "mock model did not answer on $mockBase/health" }

    $server = Start-Process -FilePath (Join-Path $root $Binary) -ArgumentList '--noconsole' -WorkingDirectory $stack -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput (Join-Path $root 'botserver-smoke.log') -RedirectStandardError (Join-Path $root 'botserver-smoke.log.err')
    $procs += $server

    $ready = $false
    for ($i = 0; $i -lt 150 -and -not $ready; $i++) {
        Start-Sleep -Seconds 2
        if ($server.HasExited) { throw "botserver exited with code $($server.ExitCode) before it was ready" }
        try { if ((Invoke-WebRequest -Uri "$base/api/product" -UseBasicParsing -TimeoutSec 5).StatusCode -eq 200) { $ready = $true } } catch {}
    }
    if (-not $ready) { throw 'botserver did not answer /api/product within 5 minutes' }

    Get-Json "$base/health" | Out-Null

    $r = Get-Json "$base/api/product"
    $r.Content | Set-Content "$Evidence/api-product.json"
    $p = $r.Content | ConvertFrom-Json
    $apps = @($p.apps | Sort-Object)
    if (($apps -join ',') -ne 'chat,drive') { throw "apps must be exactly chat,drive; got '$($apps -join ',')'" }
    if (@($p.preview_apps).Count -ne 0) { throw "the edition has no preview apps; got '$(@($p.preview_apps) -join ',')'" }
    if ($p.suite_default_app -ne 'chat') { throw "suite_default_app expected 'chat', got '$($p.suite_default_app)'" }
    if ($p.edition -ne 'desktop') { throw "edition expected 'desktop', got '$($p.edition)'" }
    $compiled = @($p.compiled_features)
    # The agent workspace is part of the edition (it reports itself unavailable
    # where its backend is missing); the office and data apps are not.
    if ($compiled -notcontains 'agent-vm') { throw "feature 'agent-vm' must be compiled into the edition" }
    foreach ($absent in 'terminal', 'database', 'research', 'sheet', 'editor', 'mail') {
        if ($compiled -contains $absent) { throw "feature '$absent' must not be compiled into the edition" }
    }
    Write-Host "product OK: apps=$($apps -join ',') compiled=$($compiled -join ',')"

    $d = Get-Json "$base/suite/desktop.html"
    if ($d.Content.Length -lt 256 -or $d.Content -notmatch '<html|<!DOCTYPE html') { throw '/suite/desktop.html is not the suite shell' }
    Write-Host "suite shell OK ($($d.Content.Length) bytes)"

    # The agent workspace is part of the edition: its routes must exist (any
    # answer but "no such route"; an empty request is rightly refused).
    $agentStatus = 0
    try {
        $agentStatus = (Invoke-WebRequest -Uri "$base/api/agent/drive/save" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10).StatusCode
    } catch { if ($_.Exception.Response) { $agentStatus = [int]$_.Exception.Response.StatusCode } }
    if ($agentStatus -eq 404 -or $agentStatus -eq 0) { throw "POST /api/agent/drive/save: the agent workspace route is missing (status $agentStatus)" }
    Write-Host "agent workspace routes present OK (status $agentStatus)"

    $reply = ''
    for ($attempt = 1; $attempt -le 3 -and -not $reply; $attempt++) {
        try {
            $body = @{ message = 'hi'; prompt = 'hi' } | ConvertTo-Json -Compress
            $c = Invoke-WebRequest -Uri "$base/api/chat" -Method POST -Body $body -ContentType 'application/json' -UseBasicParsing -TimeoutSec 45
            $c.Content | Set-Content "$Evidence/chat.json"
            $j = $c.Content | ConvertFrom-Json
            foreach ($k in 'reply', 'message', 'content', 'response') { if (-not $reply -and $j.$k) { $reply = [string]$j.$k } }
        } catch { Write-Host "chat attempt $attempt failed: $($_.Exception.Message)" }
        if (-not $reply) { Start-Sleep -Seconds 3 }
    }
    if ($reply -notmatch 'Hello from mock LLM') { throw "chat reply did not come from the mock model: '$reply'" }
    Write-Host 'chat turn OK (answered by the local mock)'

    "status=ok`napps=$($apps -join ',')`nreply=$reply" | Set-Content "$Evidence/smoke-summary.txt"
    Write-Host 'gblite smoke OK'
} catch {
    Write-Host "gblite smoke FAILED: $($_.Exception.Message)"
    Show-Logs
    throw
} finally {
    Stop-All
}
