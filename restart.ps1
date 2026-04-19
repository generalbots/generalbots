$ErrorActionPreference = "Continue"

Write-Host "Stopping..."
Stop-Process -Name "botserver" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "botui" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "rustc" -Force -ErrorAction SilentlyContinue

Write-Host "Cleaning..."
Remove-Item -Path "botserver.log", "botui.log" -Force -ErrorAction SilentlyContinue

Write-Host "Building..."
cargo build -p botserver
if ($LASTEXITCODE -ne 0) { Write-Host "Failed to build botserver"; exit 1 }

cargo build -p botui
if ($LASTEXITCODE -ne 0) { Write-Host "Failed to build botui"; exit 1 }

Write-Host "Starting botserver..."
$env:PORT = "8080"
$env:RUST_LOG = "debug"
$env:PATH += ";C:\pgsql\pgsql\bin;C:\pgsql\pgsql\lib"
$botserverProcess = Start-Process -PassThru -NoNewWindow -FilePath ".\target\debug\botserver.exe" -ArgumentList "--noconsole" -RedirectStandardOutput "botserver.log" -RedirectStandardError "botserver.log"
Write-Host "  PID: $($botserverProcess.Id)"

Write-Host "Starting botui..."
$env:BOTSERVER_URL = "http://localhost:8080"
$env:PORT = "3000"
$botuiProcess = Start-Process -PassThru -NoNewWindow -FilePath ".\target\debug\botui.exe" -RedirectStandardOutput "botui.log" -RedirectStandardError "botui.log"
Write-Host "  PID: $($botuiProcess.Id)"

Write-Host "Done. Logs are being written to botserver.log and botui.log"
Write-Host "To view logs, you can use: Get-Content botserver.log -Wait"
