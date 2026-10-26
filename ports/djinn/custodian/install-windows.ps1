# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
#
# Install Djinn's custodian as an SCM service under its virtual account
# NT SERVICE\djinn-custodian (custody isolation plan §5.2). Run from an
# elevated PowerShell, for the desktop user whose vault it will hold:
#
#   .\install-windows.ps1 -User <name> -Binary <path\djinn-custodian.exe> [-AdoptFrom <djinn vault dir>]
#
# The service's executable, configuration and state are writable only by
# Administrators, SYSTEM and (state only) the service SID. The pipe admits
# only the owner's SID; Djinn checks the pipe server is the service SID.
# The CNG wrapping key of §5.2 is not created here; the vault stays the
# portable passphrase vault until that envelope is qualified.
param(
    [Parameter(Mandatory = $true)][string]$User,
    [Parameter(Mandatory = $true)][string]$Binary,
    [string]$AdoptFrom
)
$ErrorActionPreference = 'Stop'

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Run from an elevated PowerShell.' }
$ownerSid = (New-Object Security.Principal.NTAccount($User)).Translate([Security.Principal.SecurityIdentifier]).Value

$programRoot = Join-Path $env:ProgramFiles 'Merely\djinn-custodian'
$dataRoot = 'C:\ProgramData\djinn-custodian'
$state = Join-Path $dataRoot 'state'
$vault = Join-Path $state 'vault'
New-Item -ItemType Directory -Force -Path $programRoot, $dataRoot, $state, $vault | Out-Null
Copy-Item -Force $Binary (Join-Path $programRoot 'djinn-custodian.exe')

$config = Join-Path $dataRoot 'custodian.json'
@"
{
  "vault": {"directory": "$($vault.Replace('\','\\'))"},
  "owner": {"windows_sid": "$ownerSid"},
  "consent": "deny",
  "principals": [
    {"name": "djinn", "match": {"windows-sid": "$ownerSid"}, "grants": [
      {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"}
    ]}
  ]
}
"@ | Set-Content -Encoding ascii $config

$exe = Join-Path $programRoot 'djinn-custodian.exe'
if (-not (Get-Service djinn-custodian -ErrorAction SilentlyContinue)) {
    New-Service -Name djinn-custodian -DisplayName 'Djinn custodian' -StartupType Automatic `
        -BinaryPathName "`"$exe`" --config `"$config`"" | Out-Null
}
sc.exe config djinn-custodian obj= 'NT SERVICE\djinn-custodian' | Out-Null
sc.exe sidtype djinn-custodian unrestricted | Out-Null
$serviceSid = (New-Object Security.Principal.NTAccount('NT SERVICE\djinn-custodian')).Translate([Security.Principal.SecurityIdentifier]).Value

# Program files: Administrators and SYSTEM write, the service reads.
icacls $programRoot /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)RX" | Out-Null
# Configuration: readable by the service, writable by Administrators only.
icacls $dataRoot /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)RX" | Out-Null
# State: the service alone (plus SYSTEM and Administrators for recovery).
icacls $state /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)M" | Out-Null

$client = Join-Path $dataRoot 'client.json'
@"
{"endpoint": "\\\\.\\pipe\\djinn-custodian-$ownerSid", "server": {"windows_sids": ["$serviceSid"]}}
"@ | Set-Content -Encoding ascii $client
icacls $client /inheritance:r /grant:r 'Administrators:F' 'SYSTEM:F' 'Users:R' | Out-Null

if ($AdoptFrom) {
    Write-Host 'Adopt the existing vault by running, as the service is stopped:'
    Write-Host "  `"$exe`" adopt --config `"$config`" --from `"$AdoptFrom`""
    Write-Host 'from an elevated prompt, then re-run icacls on the state directory as above.'
}
Start-Service djinn-custodian
Write-Host "Installed. Service SID $serviceSid; owner $ownerSid."
