param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('build', 'test', 'fmt', 'clippy', 'run', 'check')]
    [string]$Task,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArguments
)
$ErrorActionPreference = 'Stop'
$toolchain = '1.93.1-x86_64-pc-windows-gnu'
$cargoBin = Join-Path $env:USERPROFILE '.cargo/bin'
$rustup = Join-Path $cargoBin 'rustup.exe'
$cargo = Join-Path $cargoBin 'cargo.exe'
$compiler = (& $rustup which --toolchain $toolchain rustc | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Install the pinned compiler with mise run windows:setup' }
$toolchainRoot = Split-Path (Split-Path $compiler -Parent) -Parent
$nativeTools = Join-Path $toolchainRoot 'lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained'
if (-not (Test-Path -LiteralPath (Join-Path $nativeTools 'dlltool.exe'))) {
    throw 'The pinned GNU toolchain is missing its bundled dlltool.exe'
}
# wgpu's bindings use raw-dylib imports. Rust's GNU dlltool needs an unbundled
# assembler, so adapt the protocol to the bundled LLVM library writer instead.
$env:PATH = "$nativeTools;$cargoBin;$env:PATH"
$env:OPSEC_RUST_LLD = Join-Path $toolchainRoot 'lib/rustlib/x86_64-pc-windows-gnu/bin/rust-lld.exe'
$repoRoot = Split-Path $PSScriptRoot -Parent
$helperDirectory = Join-Path $repoRoot 'target/windows-tools'
$helper = Join-Path $helperDirectory 'windows-imports.exe'
$helperSource = Join-Path $PSScriptRoot 'windows-imports.rs'
if ($Task -ne 'fmt') {
    New-Item -ItemType Directory -Force -Path $helperDirectory | Out-Null
    if (-not (Test-Path -LiteralPath $helper) -or (Get-Item $helperSource).LastWriteTimeUtc -gt (Get-Item $helper).LastWriteTimeUtc) {
        & $compiler $helperSource --edition 2024 -O -o $helper
        if ($LASTEXITCODE -ne 0) { throw 'Failed to compile the Windows import-library adapter' }
    }
    $flags = ConvertTo-Json -Compress -InputObject @('-C', "dlltool=$helper")
    $config = Join-Path $helperDirectory 'imports.toml'
    [System.IO.File]::WriteAllText($config, "[target.x86_64-pc-windows-gnu]`nrustflags = $flags`n", (New-Object System.Text.UTF8Encoding($false)))
    & $cargo "+$toolchain" --config $config $Task @CargoArguments
} else {
    & $cargo "+$toolchain" $Task @CargoArguments
}
exit $LASTEXITCODE
