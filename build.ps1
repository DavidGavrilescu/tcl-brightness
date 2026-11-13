param([switch]$Check)
$ErrorActionPreference='Stop'
Set-Location $PSScriptRoot
if(Test-Path '.tools\cargo\bin\cargo.exe') {
    $env:RUSTUP_HOME=Join-Path $PSScriptRoot '.tools\rustup'
    $env:CARGO_HOME=Join-Path $PSScriptRoot '.tools\cargo'
    $env:PATH="$env:CARGO_HOME\bin;$env:PATH"
}
if($Check){cargo fmt --check;if($LASTEXITCODE){throw 'Formatting check failed'}}
cargo build --release
if($LASTEXITCODE){throw 'Build failed'}
