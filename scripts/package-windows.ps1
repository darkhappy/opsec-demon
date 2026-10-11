$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$source = Join-Path $repoRoot 'target/release/opsec-demon.exe'
$destination = Join-Path $repoRoot 'target/dist'
if (-not (Test-Path -LiteralPath $source)) { throw 'Run mise run build first.' }
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$executable = Join-Path $destination 'OPSEC-DEMON.exe'
Copy-Item -LiteralPath $source -Destination $executable -Force
Copy-Item -LiteralPath (Join-Path $repoRoot 'assets/fsr1.wgsl') -Destination (Join-Path $destination 'FidelityFX-FSR1-source-and-license.wgsl') -Force
$algorithm = [System.Security.Cryptography.SHA256]::Create()
$stream = [System.IO.File]::OpenRead($executable)
try {
    $hash = [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
} finally {
    $stream.Dispose()
    $algorithm.Dispose()
}
[System.IO.File]::WriteAllText((Join-Path $destination 'SHA256SUMS.txt'), "$hash  OPSEC-DEMON.exe`n")
Write-Host "Standalone Windows executable: $executable"
