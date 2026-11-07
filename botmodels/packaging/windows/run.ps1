# Start BotModels. Installs the virtual environment on first run.
$ErrorActionPreference = 'Stop'

$Root = $PSScriptRoot
Set-Location $Root
$VenvPython = Join-Path $Root '.venv\Scripts\python.exe'

if (-not (Test-Path $VenvPython)) {
    & (Join-Path $Root 'install.ps1')
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$BindHost = if ($env:BOTMODELS_HOST) { $env:BOTMODELS_HOST } else { '0.0.0.0' }
$Port = if ($env:BOTMODELS_PORT) { $env:BOTMODELS_PORT } else { '8085' }

& $VenvPython -m uvicorn src.main:app --host $BindHost --port $Port
exit $LASTEXITCODE
