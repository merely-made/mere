# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path "$PSScriptRoot/../../../../..").Path
Set-Location $repository
$env:CARGO_TARGET_DIR = 'C:/t/cargo-targets/mere'
$env:CARGO_BUILD_BUILD_DIR = 'C:/t/cargo-targets/mere/build'
$env:CARGO_NET_OFFLINE = 'true'
$gates = @(
    @{ Name='final-retained-mount'; Args=@('test','-p','cambium','--lib','retained_mount::tests') },
    @{ Name='final-frisket-workspace'; Args=@('test','-p','cambium','--lib','workspace::tests') },
    @{ Name='final-native-mailbox'; Args=@('test','-p','pelt-core','--test','host_surface','mailbox_polling_follows_admitted_native_window_custody') },
    @{ Name='final-document-polling'; Args=@('test','-p','pelt-core','--test','pool','pending_observation_is_read_only_and_redraw_is_scoped_to_visible_windows') },
    @{ Name='final-shared-query'; Args=@('test','-p','mere','--no-default-features','--features','query','--lib','query_view') },
    @{ Name='final-owner-query'; Args=@('test','-p','graphshell','--no-default-features','--features','query-views,retained-content,image-decode','--lib','query_views::tests') },
    @{ Name='final-clip-wheel'; Args=@('test','-p','graphshell','--no-default-features','--features','query-views,retained-content,image-decode','--lib','retained_content::') },
    @{ Name='final-embedding'; Args=@('test','-p','cambium-rootstock','--no-default-features','--features','embedding-egui,embedding-iced','--lib','embedding::tests') }
)
$results = @()
foreach ($gate in $gates) {
    $arguments = @('+1.99.0') + $gate.Args + @('--locked','--offline','-j1')
    & cargo @arguments *> "$PSScriptRoot/$($gate.Name).log"
    $code = $LASTEXITCODE
    $results += @{ name=$gate.Name; command=('cargo ' + ($arguments -join ' ')); exit_code=$code }
    $results | ConvertTo-Json -Depth 5 | Set-Content "$PSScriptRoot/final-native-gates.json"
    Write-Output "$($gate.Name): exit $code"
    if ($code -ne 0) { exit $code }
}
$toolkitOutput = "$repository/ports/graphshell/docs/receipts/caller_owned_toolkits_20261013/final/native"
& cargo +1.99.0 run -p graphshell --no-default-features --features caller-owned-toolkit-bench,image-decode --example caller_owned_toolkits --locked --offline -j1 -- $toolkitOutput *> "$PSScriptRoot/final-toolkit-bench.log"
$code = $LASTEXITCODE
$results += @{ name='final-toolkit-bench'; command='cargo +1.99.0 run -p graphshell --no-default-features --features caller-owned-toolkit-bench,image-decode --example caller_owned_toolkits --locked --offline -j1 -- ports/graphshell/docs/receipts/caller_owned_toolkits_20261013/final/native'; exit_code=$code }
$results | ConvertTo-Json -Depth 5 | Set-Content "$PSScriptRoot/final-native-gates.json"
Write-Output "final-toolkit-bench: exit $code"
if ($code -ne 0) { exit $code }
& python scripts/check_port_boundaries.py *> "$PSScriptRoot/final-port-boundaries.log"
$code = $LASTEXITCODE
$results += @{ name='final-port-boundaries'; command='python scripts/check_port_boundaries.py'; exit_code=$code }
$results | ConvertTo-Json -Depth 5 | Set-Content "$PSScriptRoot/final-native-gates.json"
Write-Output "final-port-boundaries: exit $code"
exit $code
