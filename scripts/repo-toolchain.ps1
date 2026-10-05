# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

# The repository's pinned Rust toolchain, for the runners that build from a
# neutral directory (burn migration plan 13.45). rustup picks a toolchain from
# the working directory, so a runner that leaves the checkout would otherwise
# get the machine's default without saying so.
#
# Dot-source this file, then call Use-RepoToolchain with mere's root. It reads
# the channel from rust-toolchain.toml, so a bump there carries through. It
# refuses a channel that is not installed, with no download and no fallback.
# It pins RUSTUP_TOOLCHAIN for the rest of the runner and prints the rustc
# version line the build will use.

function Use-RepoToolchain {
    param([Parameter(Mandatory)][string]$MereRoot)
    $file = Join-Path $MereRoot 'rust-toolchain.toml'
    $match = [regex]::Match((Get-Content -Raw -LiteralPath $file), '(?m)^\s*channel\s*=\s*"([^"]+)"')
    if (-not $match.Success) { throw "No toolchain channel in $file." }
    $channel = $match.Groups[1].Value
    # Never let a rustup proxy install a missing toolchain behind the check.
    $env:RUSTUP_AUTO_INSTALL = '0'
    $installed = @(rustup toolchain list | ForEach-Object { ($_ -split '\s+')[0] })
    if (-not ($installed | Where-Object { $_ -eq $channel -or $_ -like "$channel-*" })) {
        throw "The repository pins Rust $channel ($file), which is not installed. Install it with rustup; this runner does not fall back to another toolchain."
    }
    $env:RUSTUP_TOOLCHAIN = $channel
    $version = (rustc --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $version) { throw "rustc from toolchain $channel did not run." }
    Write-Host "toolchain: $channel ($version)"
}
