$ErrorActionPreference = "Continue"

Write-Host "=== Fast Restart: botserver + botui + botmodels ==="
$processPath = [Environment]::GetEnvironmentVariable("Path", "Process")
if (-not $processPath) {
    $processPath = [Environment]::GetEnvironmentVariable("PATH", "Process")
}
[Environment]::SetEnvironmentVariable("PATH", $null, "Process")
[Environment]::SetEnvironmentVariable("Path", $processPath, "Process")
$repoRoot = (Resolve-Path $PSScriptRoot).Path
$stackRoot = Join-Path $repoRoot "botserver\botserver-stack"

function Test-LocalPort {
    param(
        [Parameter(Mandatory = $true)][int]$Port,
        [int]$TimeoutMilliseconds = 1000
    )

    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $connection = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
        if (-not $connection.AsyncWaitHandle.WaitOne($TimeoutMilliseconds, $false)) {
            return $false
        }
        $client.EndConnect($connection)
        return $true
    } catch {
        return $false
    } finally {
        $client.Close()
    }
}

Write-Host "Stopping..."
Stop-Process -Name "botserver" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "botui" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "rustc" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

Write-Host "Cleaning..."
Remove-Item -Path "botserver.log", "botui.log", "botmodels.log" -Force -ErrorAction SilentlyContinue

Write-Host "Building..."
cargo build -p botserver
if ($LASTEXITCODE -ne 0) { Write-Host "Failed to build botserver"; exit 1 }

cargo build -p botui
if ($LASTEXITCODE -ne 0) { Write-Host "Failed to build botui"; exit 1 }

$botmodelsReady = Test-LocalPort -Port 8085
if ($botmodelsReady) {
    Write-Host "Reusing botmodels already listening on port 8085"
}

if (-not $botmodelsReady) {
    Write-Host "Starting botmodels..."
    Push-Location botmodels
    $uvicorn = (Get-Command uvicorn -ErrorAction SilentlyContinue).Source
    if (-not $uvicorn) { $uvicorn = "python"; $uvicornArgs = @("-m", "uvicorn") } else { $uvicornArgs = @() }
    $botmodelsProcess = Start-Process -PassThru -NoNewWindow -FilePath $uvicorn -ArgumentList ($uvicornArgs + @("src.main:app", "--host", "0.0.0.0", "--port", "8085")) -RedirectStandardOutput "..\botmodels.log" -RedirectStandardError "..\botmodels-err.log"
    Write-Host "  PID: $($botmodelsProcess.Id)"
    Pop-Location
}

Write-Host "Waiting for botmodels..."
for ($i = 1; $i -le 20; $i++) {
    if (Test-LocalPort -Port 8085) {
        Write-Host "  botmodels ready"
        $botmodelsReady = $true
        break
    }
    Start-Sleep -Seconds 1
}
if (-not $botmodelsReady) { Write-Host "  WARNING: botmodels did not become ready within 20s" }

$cacheReady = Test-LocalPort -Port 6379
if (-not $cacheReady) {
    $cacheBin = Join-Path $stackRoot "bin\cache\redis-server.exe"
    $cacheData = Join-Path $stackRoot "data\cache"
    $cacheLogs = Join-Path $stackRoot "logs\cache"
    if (Test-Path -LiteralPath $cacheBin) {
        New-Item -ItemType Directory -Path $cacheData, $cacheLogs -Force | Out-Null
        Write-Host "Starting cache..."
        $cacheProcess = Start-Process -PassThru -WindowStyle Hidden -FilePath $cacheBin `
            -ArgumentList @("--port", "6379", "--bind", "127.0.0.1", "::1", "--dir", $cacheData, "--logfile", (Join-Path $cacheLogs "valkey.log")) `
            -WorkingDirectory (Split-Path $cacheBin)
        Write-Host "  PID: $($cacheProcess.Id)"
        for ($i = 1; $i -le 10; $i++) {
            if (Test-LocalPort -Port 6379) {
                $cacheReady = $true
                break
            }
            Start-Sleep -Seconds 1
        }
    }
}
if (-not $cacheReady) { Write-Host "  WARNING: cache did not become ready within 10s" }

$envFile = "botserver\.env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        if ($_ -match "^\s*([^#=]+)=(.*)$") {
            [Environment]::SetEnvironmentVariable($matches[1].Trim(), $matches[2].Trim(), "Process")
        }
    }
}

$initFile = Join-Path $stackRoot "conf\vault\init.json"
$vaultTokenFile = Join-Path ([IO.Path]::GetTempPath()) "vault-token-gb"
if (Test-Path -LiteralPath $vaultTokenFile) {
    $env:VAULT_TOKEN = (Get-Content -LiteralPath $vaultTokenFile -Raw).Trim()
} elseif (-not $env:VAULT_TOKEN -and (Test-Path -LiteralPath $initFile)) {
    try {
        $env:VAULT_TOKEN = (Get-Content -LiteralPath $initFile -Raw | ConvertFrom-Json).root_token
    } catch {
        Write-Host "  WARNING: could not recover the local Vault token from init.json"
    }
}
if (-not $env:VAULT_CACERT -or -not (Test-Path -LiteralPath $env:VAULT_CACERT)) {
    Remove-Item Env:VAULT_CACERT -ErrorAction SilentlyContinue
}
$env:BOTSERVER_STACK_PATH = $stackRoot
$env:GBO_STACK_PATH = $stackRoot
$env:GBO_SKIP_LOCAL_DIRECTORY = "1"
$env:GBO_WSL_DISTRO = "Debian"
$env:SAAS_DISABLE_CAPACITY_CHECK = "1"
$env:VIBE_VM_IMAGE = "images:debian/13"
$env:VIBE_WSL_APP_PORT = "39000"

$vaultBin = Join-Path $stackRoot "bin\vault\vault.exe"
$vaultAddr = if ($env:VAULT_ADDR) { $env:VAULT_ADDR } else { "http://127.0.0.1:8200" }
$minioAccessKey = & $vaultBin kv get -field=accesskey -tls-skip-verify -address=$vaultAddr secret/gbo/drive 2>$null
$minioSecretKey = & $vaultBin kv get -field=secret -tls-skip-verify -address=$vaultAddr secret/gbo/drive 2>$null
$databaseName = & $vaultBin kv get -field=database -tls-skip-verify -address=$vaultAddr secret/gbo/tables 2>$null
$databaseUser = & $vaultBin kv get -field=username -tls-skip-verify -address=$vaultAddr secret/gbo/tables 2>$null
$databasePassword = & $vaultBin kv get -field=password -tls-skip-verify -address=$vaultAddr secret/gbo/tables 2>$null
$databasePort = & $vaultBin kv get -field=port -tls-skip-verify -address=$vaultAddr secret/gbo/tables 2>$null
if ($databaseName -and $databaseUser -and $databasePort) {
    $encodedUser = [Uri]::EscapeDataString(($databaseUser | Out-String).Trim())
    $encodedPassword = [Uri]::EscapeDataString(($databasePassword | Out-String).Trim())
    $env:DATABASE_URL = "postgres://${encodedUser}:${encodedPassword}@127.0.0.1:$($databasePort.Trim())/$($databaseName.Trim())"
}

Write-Host "Starting botserver..."
$env:PORT = "8080"
$env:RUST_LOG = "info"
$env:MINIO_ACCESS_KEY = $minioAccessKey
$env:MINIO_SECRET_KEY = $minioSecretKey
$env:MINIO_ENDPOINT = "http://127.0.0.1:9100"
$env:MINIO_BUCKET = "default.gbai"
$env:MINIO_SERVER = "http://127.0.0.1:9100"
$env:BOTMODELS_HOST = "http://localhost:8085"
$env:BOTMODELS_API_KEY = "starter"
$processPath = [Environment]::GetEnvironmentVariable("Path", "Process")
[Environment]::SetEnvironmentVariable("Path", "$processPath;C:\pgsql\pgsql\bin;C:\pgsql\pgsql\lib", "Process")
$botserverProcess = Start-Process -PassThru -NoNewWindow -FilePath ".\target\debug\botserver.exe" -ArgumentList @("--noconsole", "--stack-path", $stackRoot) -WorkingDirectory "botserver" -RedirectStandardOutput "botserver.log" -RedirectStandardError "botserver-err.log"
Write-Host "  PID: $($botserverProcess.Id)"

Start-Sleep -Seconds 2

Write-Host "Starting botui..."
$env:BOTSERVER_URL = "http://localhost:8080"
$env:PORT = "3000"
$botuiProcess = Start-Process -PassThru -NoNewWindow -FilePath ".\target\debug\botui.exe" -RedirectStandardOutput "botui.log" -RedirectStandardError "botui-err.log"
Write-Host "  PID: $($botuiProcess.Id)"

Write-Host "Done. botserver=$($botserverProcess.Id) botui=$($botuiProcess.Id) botmodels=$($botmodelsProcess.Id)"
Write-Host "Logs: botserver.log, botui.log, botmodels.log"
Write-Host "To view logs, you can use: Get-Content botserver.log -Wait"
