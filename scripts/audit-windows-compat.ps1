param(
    [string]$OutputPath = (Join-Path ([IO.Path]::GetTempPath()) "generalbots-windows-compatibility.md"),
    [switch]$RunBuild
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$extensions = @(".rs", ".toml", ".ps1", ".js", ".html", ".py")
$sourceRoots = @("botserver\src", "botserver\crates", "botlib\src", "botui\src", "botui\ui", "botapp\src", "botmodels\src")
$rules = @(
    [pscustomobject]@{ Id = "PATH-TEMP"; Pattern = '/tmp/'; Description = "Unix temporary path" },
    [pscustomobject]@{ Id = "PATH-PROD"; Pattern = '(?<![A-Za-z0-9_.-])/(opt|usr|bin)/'; Description = "Unix absolute path" },
    [pscustomobject]@{ Id = "PROC-UNIX"; Pattern = '\b(pkill|pgrep|killall|systemctl|nohup|chmod|chown)\b'; Description = "Unix process or permission command" },
    [pscustomobject]@{ Id = "PKG-LINUX"; Pattern = '(build/bin|\.so\b|LD_LIBRARY_PATH)'; Description = "Linux package or loader layout" },
    [pscustomobject]@{ Id = "SHELL-UNIX"; Pattern = '(/bin/(ba)?sh|SafeCommand::new\("(ba)?sh"\))'; Description = "Unix shell assumption" },
    [pscustomobject]@{ Id = "PROCESS-DIRECT"; Pattern = '(std::process::Command::new|tokio::process::Command::new)'; Description = "Direct process execution requiring platform and security review" }
)

function Get-FindingClassification {
    param(
        [string]$RelativePath,
        [string]$SourceText,
        [int]$LineNumber,
        [int]$TestModuleStart
    )

    if ($RelativePath -match '[\\/]vendor[\\/]') {
        return "Bundled third-party asset"
    }
    if ($TestModuleStart -gt 0 -and $LineNumber -ge $TestModuleStart) {
        return "Test-only"
    }
    if ($SourceText -match '^\s*(//|/\*|\*|#(?!\[))') {
        return "Documentation or comment"
    }
    if ($RelativePath -match 'botcorepkg[\\/]src[\\/]installer_regs(2)?\.rs$') {
        return "Linux package definition with Windows capability metadata"
    }
    if ($RelativePath -match '(facade_container|package_manager[\\/](container|facade)|incus[\\/]cloud|botvibe[\\/]src[\\/]vm_incus|code_sandbox|botdeployment[\\/]src[\\/]forgejo)\.rs$') {
        return "Linux guest or container command"
    }
    if ($RelativePath -match 'botsecurity-protection[\\/]src[\\/]protection[\\/]') {
        return "Linux-only protection capability"
    }
    if ($RelativePath -match 'os_abstraction[\\/](linux|macos|windows)\.rs$') {
        return "Platform implementation"
    }
    if ($RelativePath -match '(botlib[\\/]src[\\/](work_path|branding|os[\\/]linux_command|security[\\/]utils|security[\\/]command_guard)|botllm[\\/]src[\\/]local|botserver[\\/]src[\\/]api[\\/]terminal[\\/].*|botserver[\\/]src[\\/](console[\\/]status_panel|console[\\/]wizard[\\/]wizard_core|main_module[\\/]drive_utils|main_module[\\/]routes[\\/]feature_routers)|botcorepkg[\\/]src[\\/](installer|facade_download|cli)|botcoresecrets[\\/]src[\\/]manager|botsources[\\/]src[\\/]state|botcore[\\/]src[\\/]bootstrap[\\/].*|botcore[\\/]src[\\/]shared[\\/]utils|botcore[\\/]src[\\/]package_manager[\\/](installer|certs_utils|cli)|botapi[\\/]src[\\/]terminal|botautotask[\\/]src[\\/]container_session|botdeployment[\\/]src[\\/]gateway_server|botvibe[\\/]src[\\/]harness[\\/](mod|cmd)|botmonitoring[\\/]src[\\/]real_time|botvideo[\\/]src[\\/]safe_command|botsecurity-core[\\/]src[\\/](antivirus|file_validation)|botmodelsbridge[\\/]src[\\/]opencv|botkb[\\/]src[\\/]face_api[\\/]opencv|botsecurity-crypto[\\/]src[\\/]tls)\.rs$') {
        return "Platform-gated host implementation"
    }
    return "Requires review"
}

$files = foreach ($root in $sourceRoots) {
    $fullRoot = Join-Path $repoRoot $root
    if (Test-Path -LiteralPath $fullRoot) {
        Get-ChildItem -LiteralPath $fullRoot -Recurse -File | Where-Object {
            $extensions -contains $_.Extension -and
            $_.FullName -notmatch '[\\/](target|node_modules|gen)[\\/]'
        }
    }
}

$findings = New-Object System.Collections.Generic.List[object]
foreach ($file in $files) {
    $relative = $file.FullName.Substring($repoRoot.Length).TrimStart("\\")
    $testModuleStart = 0
    if ($file.Extension -eq ".rs") {
        $testMarker = Select-String -LiteralPath $file.FullName -Pattern '^\s*#\[cfg\(test\)\]'
        if ($testMarker) {
            $testModuleStart = ($testMarker | Select-Object -First 1).LineNumber
        }
    }
    foreach ($rule in $rules) {
        foreach ($match in (Select-String -LiteralPath $file.FullName -Pattern $rule.Pattern -AllMatches)) {
            $findings.Add([pscustomobject]@{
                Rule = $rule.Id
                Description = $rule.Description
                File = $relative
                Line = $match.LineNumber
                Text = $match.Line.Trim()
                Classification = Get-FindingClassification -RelativePath $relative -SourceText $match.Line -LineNumber $match.LineNumber -TestModuleStart $testModuleStart
            })
        }
    }
}

function Test-LocalPort {
    param([int]$Port)
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $pending = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
        if (-not $pending.AsyncWaitHandle.WaitOne(500, $false)) { return $false }
        $client.EndConnect($pending)
        return $true
    } catch {
        return $false
    } finally {
        $client.Close()
    }
}

$stackRoot = Join-Path $repoRoot "botserver\botserver-stack"
$checks = @(
    [pscustomobject]@{ Name = "Debian WSL distribution"; Passed = [bool](wsl --list --quiet 2>$null | Where-Object { $_.Trim() -eq "Debian" }) },
    [pscustomobject]@{ Name = "Packaged llama-server.exe"; Passed = Test-Path -LiteralPath (Join-Path $stackRoot "bin\llm\llama-server.exe") },
    [pscustomobject]@{ Name = "Local chat model"; Passed = Test-Path -LiteralPath (Join-Path $stackRoot "data\llm\DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf") },
    [pscustomobject]@{ Name = "Local embedding model"; Passed = Test-Path -LiteralPath (Join-Path $stackRoot "data\llm\bge-small-en-v1.5-f32.gguf") },
    [pscustomobject]@{ Name = "Packaged mc.exe"; Passed = Test-Path -LiteralPath (Join-Path $stackRoot "bin\drive\mc.exe") },
    [pscustomobject]@{ Name = "Botserver port 8080"; Passed = Test-LocalPort 8080 },
    [pscustomobject]@{ Name = "Local LLM port 8081"; Passed = Test-LocalPort 8081 },
    [pscustomobject]@{ Name = "Botmodels port 8085"; Passed = Test-LocalPort 8085 }
)

$buildPassed = $null
if ($RunBuild) {
    Push-Location $repoRoot
    try {
        cargo check -p botserver
        $buildPassed = $LASTEXITCODE -eq 0
    } finally {
        Pop-Location
    }
}

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("# General Bots Windows Compatibility Scan")
$lines.Add("")
$lines.Add("Generated: $(Get-Date -Format o)")
$lines.Add("")
$lines.Add("## Runtime and package checks")
$lines.Add("")
$lines.Add("| Check | Result |")
$lines.Add("|---|---|")
foreach ($check in $checks) {
    $lines.Add("| $($check.Name) | $(if ($check.Passed) { 'PASS' } else { 'FAIL' }) |")
}
if ($null -ne $buildPassed) {
    $lines.Add("| cargo check -p botserver | $(if ($buildPassed) { 'PASS' } else { 'FAIL' }) |")
}
$lines.Add("")
$lines.Add("## Static review candidates")
$lines.Add("")
$lines.Add("Raw matches are classified before review. Only `Requires review` entries are unresolved audit work; the other categories are retained as trace evidence.")
$lines.Add("")
$lines.Add("| Classification | Count |")
$lines.Add("|---|---:|")
foreach ($group in ($findings | Group-Object Classification | Sort-Object Name)) {
    $lines.Add("| $($group.Name) | $($group.Count) |")
}
$lines.Add("")
$reviewFindings = @($findings | Where-Object { $_.Classification -eq "Requires review" })
$lines.Add("Unresolved review candidates: $($reviewFindings.Count)")
$lines.Add("")
$lines.Add("### Unresolved candidates")
$lines.Add("")
$lines.Add("| Rule | File | Line | Source |")
$lines.Add("|---|---|---:|---|")
foreach ($finding in ($reviewFindings | Sort-Object Rule, File, Line)) {
    $safeText = $finding.Text.Replace("|", "\|")
    $lines.Add("| $($finding.Rule) | $($finding.File) | $($finding.Line) | ``$safeText`` |")
}
$lines.Add("")
$lines.Add("### Classified evidence")
$lines.Add("")
$lines.Add("| Classification | Rule | File | Line | Source |")
$lines.Add("|---|---|---|---:|---|")
foreach ($finding in ($findings | Sort-Object Rule, File, Line)) {
    $safeText = $finding.Text.Replace("|", "\|")
    $lines.Add("| $($finding.Classification) | $($finding.Rule) | $($finding.File) | $($finding.Line) | ``$safeText`` |")
}

$outputDirectory = Split-Path -Parent $OutputPath
if ($outputDirectory -and -not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}
$lines | Set-Content -LiteralPath $OutputPath -Encoding utf8
Write-Host "Windows compatibility report: $OutputPath"
Write-Host "Static review candidates: $($findings.Count)"
Write-Host "Unresolved review candidates: $($reviewFindings.Count)"
