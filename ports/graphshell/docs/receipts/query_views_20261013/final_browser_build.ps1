# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path "$PSScriptRoot/../../../../..").Path
function Get-SourceQualification {
    Push-Location $repository
    try {
        foreach ($relative in (& git ls-files '*.rs' '*.toml' '*.lock' '*.html' '*.js')) {
            $path = Join-Path $repository $relative
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                @{ path=$relative; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }
            }
        }
    } finally { Pop-Location }
}
$sources = @(Get-SourceQualification)
$env:CARGO_TARGET_DIR = 'C:/t/cargo-targets/mere'
$env:CARGO_BUILD_BUILD_DIR = 'C:/t/cargo-targets/mere/build'
$env:CARGO_NET_OFFLINE = 'true'
Set-Location "$repository/ports/graphshell/web"
& cargo +1.99.0 build --target wasm32-unknown-unknown --locked --offline -j1 *> "$PSScriptRoot/final-browser-build.log"
$code = $LASTEXITCODE
Write-Output "final-browser-build: exit $code"
if ($code -ne 0) { exit $code }
$package = 'C:/t/cargo-targets/mere/retained-content-browser'
$markerPath = "$package/.qualification-owner.json"
if (-not (Test-Path -LiteralPath $package)) {
    New-Item -ItemType Directory -Path $package | Out-Null
    @{
        owner='query views and retained clip browser qualification'
        generated_by='wasm-bindgen0.2.129'
        receipt="$PSScriptRoot/final-browser-artifact.json"
        generated_files=@('graphshell_web_bg.wasm','graphshell_web.js','graphshell_web.d.ts','graphshell_web_bg.wasm.d.ts')
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $markerPath
}
if (-not (Test-Path -LiteralPath $markerPath)) { throw 'Browser package owner marker is missing' }
$owner = Get-Content -LiteralPath $markerPath -Raw | ConvertFrom-Json
if ($owner.owner -ne 'query views and retained clip browser qualification') { throw 'Browser package owner does not match' }
& wasm-bindgen 'C:/t/cargo-targets/mere/wasm32-unknown-unknown/debug/graphshell_web.wasm' --target web --out-dir $package *> "$PSScriptRoot/final-browser-bindgen.log"
$code = $LASTEXITCODE
Write-Output "final-browser-bindgen: exit $code"
if ($code -ne 0) { exit $code }
Set-Location $repository
$after = @(Get-SourceQualification)
if (($sources | ConvertTo-Json -Depth 4 -Compress) -ne ($after | ConvertTo-Json -Depth 4 -Compress)) {
    throw 'Tracked consumer source changed during the browser build; preserve logs and rebuild the final artifact'
}
$artifact = @{
    source_head=(& git rev-parse HEAD)
    rust='1.99.0'
    command='cargo +1.99.0 build --target wasm32-unknown-unknown --locked --offline -j1'
    cwd='ports/graphshell/web'
    target=$env:CARGO_TARGET_DIR
    build_dir=$env:CARGO_BUILD_BUILD_DIR
    wasm_bindgen=(& wasm-bindgen --version)
    genet='b6bfbeb556a6a17eddf8d5ad8254d3fcac95b084'
    netrender='6a24ae08f55129b474282b8e4911564f875ce5fd'
    files=@()
    consumer_sources=$sources
}
foreach ($name in $owner.generated_files) {
    $file = Get-Item -LiteralPath "$package/$name"
    $artifact.files += @{ path=$file.FullName; bytes=$file.Length; sha256=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
}
$artifact | ConvertTo-Json -Depth 5 | Set-Content "$PSScriptRoot/final-browser-artifact.json"
$owner.receipt = "$PSScriptRoot/final-browser-artifact.json"
$owner | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $markerPath
