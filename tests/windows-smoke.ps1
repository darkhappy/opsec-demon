param([switch]$SkipVulkan)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$binary = Join-Path $repoRoot 'target/release/opsec-demon.exe'
function Invoke-Check([string]$command, [string]$expected) {
    $output = (& $binary --plain -c $command | Out-String)
    if ($LASTEXITCODE -ne 0 -or -not $output.Contains($expected)) {
        throw "Windows command check failed: $command"
    }
}
Invoke-Check 'echo WINDOWS_NATIVE_OK|findstr NATIVE' 'WINDOWS_NATIVE_OK'
Invoke-Check 'mission next' '[STORY MODE]'
Invoke-Check 'opsecular desktop' 'SIM'
Invoke-Check 'discord preview' 'SIM'
Invoke-Check 'rips list' '[SYNTH AUDIO]'
& $binary --plain -c 'exit 7'
if ($LASTEXITCODE -ne 7) { throw 'Exit status was lost' }
if ($SkipVulkan) {
    Write-Host 'Windows native command checks passed. Vulkan rendering explicitly skipped on this host.'
    exit 0
}
$previews = @(Join-Path $repoRoot 'target/desktop-4k.png')
$previews += Join-Path $repoRoot 'target/desktop-audio-source-4k.png'
1..7 | ForEach-Object { $previews += Join-Path $repoRoot "target/desktop-page-$_-4k.png" }
1..5 | ForEach-Object { $previews += Join-Path $repoRoot "target/desktop-upscaler-$_-4k.png" }
foreach ($preview in $previews) {
    if (Test-Path -LiteralPath $preview) { Remove-Item -LiteralPath $preview }
}
$reportPath = Join-Path $repoRoot 'target/vulkan-smoke.json'
if (Test-Path -LiteralPath $reportPath) { Remove-Item -LiteralPath $reportPath }
& $binary --gui-smoke
if ($LASTEXITCODE -ne 0) { throw 'Native desktop smoke failed' }
foreach ($preview in $previews) {
    $bytes = [System.IO.File]::ReadAllBytes($preview)
    $width = ([int]$bytes[16] -shl 24) -bor ([int]$bytes[17] -shl 16) -bor ([int]$bytes[18] -shl 8) -bor [int]$bytes[19]
    $height = ([int]$bytes[20] -shl 24) -bor ([int]$bytes[21] -shl 16) -bor ([int]$bytes[22] -shl 8) -bor [int]$bytes[23]
    if ($width -ne 3840 -or $height -ne 2160) { throw "Desktop page is not 3840 x 2160: $preview" }
}
$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
if ($report.backend -ne 'Vulkan' -or $report.pages_rendered -ne 8 -or $report.native_command -ne 'passed' -or $report.native_window -ne 'passed' -or $report.audio_graphs -ne 8 -or $report.fsr_modes_tested -ne 4 -or -not $report.manual_scale_tested -or $report.windows_pcm_output -ne 'passed (master muted)') {
    throw 'Vulkan smoke report did not verify all pages and the graphical command worker'
}
Write-Host 'Windows native commands, SIM models, all eight Vulkan pages, four FSR modes, manual scaling, audio graphs, and fresh 4K rendering passed.'
