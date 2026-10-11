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
& $binary --gui-smoke
if ($LASTEXITCODE -ne 0) { throw 'Native desktop smoke failed' }
$preview = Join-Path $repoRoot 'target/desktop-4k.bmp'
$bytes = [System.IO.File]::ReadAllBytes($preview)
if ([BitConverter]::ToInt32($bytes, 18) -ne 3840 -or [Math]::Abs([BitConverter]::ToInt32($bytes, 22)) -ne 2160) {
    throw 'Desktop preview is not 3840 x 2160'
}
Write-Host 'Windows native commands, SIM consoles, exit status, desktop, and 4K rendering passed.'
