# Deliberately use raw script arguments: PowerShell 5's advanced parameter
# binder interprets cargo's `-- -D warnings` separator as an ambiguous parameter.
$Task = [string]$args[0]
$CargoArguments = @($args | Select-Object -Skip 1)
if ($Task -notin @('build', 'test', 'fmt', 'clippy', 'run', 'check', 'doc')) {
    throw 'Usage: rust-windows.ps1 build|test|fmt|clippy|run|check|doc [cargo arguments]'
}
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
    $shellDefinition = Join-Path $PSScriptRoot 'windows-shlwapi.def'
    $linkerSource = Join-Path $PSScriptRoot 'windows-link.rs'
    $linker = Join-Path $helperDirectory 'windows-link.exe'
    if (-not (Test-Path -LiteralPath $linker) -or (Get-Item $linkerSource).LastWriteTimeUtc -gt (Get-Item $linker).LastWriteTimeUtc) {
        & $compiler $linkerSource --edition 2024 -O -o $linker
        if ($LASTEXITCODE -ne 0) { throw 'Failed to compile the LLVM Windows linker adapter' }
    }
    # Include adapter source fingerprints in the library path, so cargo rebuilds
    # raw-dylib consumers when the generator changes instead of reusing old imports.
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $importHash = [BitConverter]::ToString($algorithm.ComputeHash([System.IO.File]::ReadAllBytes($helperSource))).Replace('-', '').Substring(0, 12)
        $linkerHash = [BitConverter]::ToString($algorithm.ComputeHash([System.IO.File]::ReadAllBytes($linkerSource))).Replace('-', '').Substring(0, 12)
    } finally { $algorithm.Dispose() }
    $libraryDirectory = Join-Path $helperDirectory "$importHash-$linkerHash"
    New-Item -ItemType Directory -Force -Path $libraryDirectory | Out-Null
    $shellLibrary = Join-Path $libraryDirectory 'libshlwapi.a'
    & $env:OPSEC_RUST_LLD -flavor link /lib /machine:x64 "/def:$shellDefinition" "/out:$shellLibrary"
    if ($LASTEXITCODE -ne 0) { throw 'Failed to generate the Windows shell API import library' }
    # LLVM's import libraries also need LLVM's PE linker. GNU ld can emit a
    # malformed import table when raw-dylib libraries and MinGW archives mix.
    $flags = ConvertTo-Json -Compress -InputObject @('-C', "dlltool=$helper", '-C', "linker=$linker", '-C', 'linker-flavor=ld.lld', '-C', 'link-self-contained=yes', '-L', "native=$libraryDirectory")
    $config = Join-Path $helperDirectory 'imports.toml'
    [System.IO.File]::WriteAllText($config, "[target.x86_64-pc-windows-gnu]`nrustflags = $flags`n", (New-Object System.Text.UTF8Encoding($false)))
    & $cargo "+$toolchain" --config $config $Task @CargoArguments
} else {
    & $cargo "+$toolchain" $Task @CargoArguments
}
exit $LASTEXITCODE
