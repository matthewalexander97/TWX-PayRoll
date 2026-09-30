$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
    $taskFailures = @()
    foreach ($taskArguments in @(
        @('fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--', '--check'),
        @('check', '--manifest-path', 'src-tauri/Cargo.toml', '--locked'),
        @('test', '--manifest-path', 'src-tauri/Cargo.toml', '--locked'),
        @('clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--all-targets', '--', '-D', 'warnings')
    )) {
        & cargo @taskArguments
        if ($LASTEXITCODE -ne 0) { $taskFailures += "cargo $($taskArguments -join ' ')" }
    }
    & npm.cmd run tauri -- build --no-bundle -- --locked
    if ($LASTEXITCODE -ne 0) { $taskFailures += 'npm run tauri -- build --no-bundle -- --locked' }
    if ($taskFailures.Count -gt 0) { throw "Native validation failed: $($taskFailures -join ', ')" }
} finally { Pop-Location }
