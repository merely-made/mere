# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

param(
    [int]$Port = 8733,
    [string]$TargetDir = 'C:\t\cubek-browser-extrema-repro',
    [string]$WasmBindgen = 'wasm-bindgen',
    [switch]$NoServe
)

$ErrorActionPreference = 'Stop'
$reproRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$mereRoot = (Resolve-Path (Join-Path $reproRoot '..\..\..\..\..')).Path
# The repository's pinned toolchain (rust-toolchain.toml; burn plan 13.45),
# not whatever rustup picks in the neutral directory below.
. (Join-Path $mereRoot 'scripts\repo-toolchain.ps1')
Use-RepoToolchain -MereRoot $mereRoot
$env:CARGO_TARGET_DIR = $TargetDir

New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
$bindgenVersion = (& $WasmBindgen --version).Trim()
if ($bindgenVersion -ne 'wasm-bindgen 0.2.129') {
    throw "The repro requires wasm-bindgen CLI 0.2.129; got '$bindgenVersion'."
}
Push-Location $TargetDir
try {
    # The committed wasm cfg (.cargo/config.toml, ruling 558); this neutral
    # directory would not find it.
    cargo build --locked --manifest-path (Join-Path $reproRoot 'Cargo.toml') --config (Join-Path $reproRoot '.cargo\config.toml') --release --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

$wasm = Join-Path $TargetDir 'wasm32-unknown-unknown\release\cubek_browser_extrema_repro.wasm'
$package = Join-Path $reproRoot 'web\pkg'
& $WasmBindgen --target web --out-dir $package $wasm
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$url = "http://localhost:$Port/"
Write-Host "Cubek browser extrema repro: $url"
if ($NoServe) { exit 0 }
Write-Host 'Press Ctrl+C to stop the server.'
python -m http.server $Port --directory $reproRoot\web
