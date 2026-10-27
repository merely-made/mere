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
# Administrators, SYSTEM and (state only) the service SID. The custody pipe
# and the SSH agent's pipe admit only the owner's SID; Djinn checks the pipe
# server is the service SID. Consent comes from a helper the installer
# registers to run, as the owner, at logon: the same installed executable
# (`djinn-custodian consent`), which the policy matches by that path and
# lets decide petitions and nothing else. It puts each petition to the owner
# through Windows Hello. The CNG wrapping key of §5.2 is not created here;
# the vault stays the portable passphrase vault until that envelope is
# qualified. Djinn's existing OS-rooted vault is adopted into it, below.
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

$custody = Join-Path $state 'data'
New-Item -ItemType Directory -Force -Path $custody | Out-Null
$config = Join-Path $dataRoot 'custodian.json'
$exe = Join-Path $programRoot 'djinn-custodian.exe'
$agentPipe = "\\.\pipe\djinn-custodian-$ownerSid-agent"
# Djinn's grants (custody isolation plan, H2 inventory). Djinn is matched by
# the owner's SID, "some process of this user", so each grant needs consent,
# which only the pinned helper below can give. The helper is listed first:
# the first rule that matches a connection wins, and the helper's token also
# matches Djinn's rule.
$grants = (Get-Content -Raw (Join-Path $PSScriptRoot 'packaging\djinn-grants.json.in')).Replace('@CONSENT@', 'per-generation').Replace('@SIGN_CONSENT@', 'per-operation')
@"
{
  "vault": {"directory": "$($vault.Replace('\','\\'))", "data_root": "$($custody.Replace('\','\\'))", "agent_endpoint": "$($agentPipe.Replace('\','\\'))"},
  "owner": {"windows_sid": "$ownerSid"},
  "consent": "channel",
  "principals": [
    {"name": "consent", "match": {"windows-image": {"sid": "$ownerSid", "image": "$($exe.Replace('\','\\'))"}},
     "consent_channel": true, "grants": [{"operation": "status"}]},
    {"name": "djinn", "match": {"windows-sid": "$ownerSid"}, "grants": $grants}
  ]
}
"@ | Set-Content -Encoding ascii $config

if (-not (Get-Service djinn-custodian -ErrorAction SilentlyContinue)) {
    New-Service -Name djinn-custodian -DisplayName 'Djinn custodian' -StartupType Automatic `
        -BinaryPathName "`"$exe`" --config `"$config`"" | Out-Null
}
sc.exe config djinn-custodian obj= 'NT SERVICE\djinn-custodian' | Out-Null
sc.exe sidtype djinn-custodian unrestricted | Out-Null
$serviceSid = (New-Object Security.Principal.NTAccount('NT SERVICE\djinn-custodian')).Translate([Security.Principal.SecurityIdentifier]).Value

# Program files: Administrators and SYSTEM write; the service and the owner
# (who runs the consent helper from here) read and execute.
icacls $programRoot /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)RX" "*${ownerSid}:(OI)(CI)RX" | Out-Null
# Configuration: readable by the service, writable by Administrators only.
icacls $dataRoot /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)RX" | Out-Null
# State: the service alone (plus SYSTEM and Administrators for recovery).
icacls $state /inheritance:r /grant:r 'Administrators:(OI)(CI)F' 'SYSTEM:(OI)(CI)F' "*${serviceSid}:(OI)(CI)M" | Out-Null

$client = Join-Path $dataRoot 'client.json'
@"
{"endpoint": "\\\\.\\pipe\\djinn-custodian-$ownerSid", "server": {"windows_sids": ["$serviceSid"]},
 "agent": "$($agentPipe.Replace('\','\\'))"}
"@ | Set-Content -Encoding ascii $client
icacls $client /inheritance:r /grant:r 'Administrators:F' 'SYSTEM:F' 'Users:R' | Out-Null

# The consent helper: at the owner's logon, in the owner's interactive
# session, with no time limit, started again if it fails.
$ownerAccount = (New-Object Security.Principal.SecurityIdentifier($ownerSid)).Translate([Security.Principal.NTAccount]).Value
$task = 'djinn-custodian-consent'
Register-ScheduledTask -TaskName $task -Force `
    -Action (New-ScheduledTaskAction -Execute $exe -Argument "consent --client `"$client`"") `
    -Trigger (New-ScheduledTaskTrigger -AtLogOn -User $ownerAccount) `
    -Principal (New-ScheduledTaskPrincipal -UserId $ownerAccount -LogonType Interactive -RunLevel Limited) `
    -Settings (New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) `
        -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew) | Out-Null

if (Test-Path (Join-Path $vault 'vault.json')) {
    Start-Service djinn-custodian
    Start-ScheduledTask -TaskName $task
    Write-Host "Installed and started. Service SID $serviceSid; owner $ownerSid."
} else {
    # The custodian never creates an identity: it starts only on a vault.
    Write-Host "Installed; not started, because there is no vault in $vault yet."
    if ($AdoptFrom -and (Test-Path (Join-Path $AdoptFrom 'vault.json'))) {
        Write-Host 'Adopt Djinn''s passphrase vault from an elevated prompt:'
        Write-Host "  `"$exe`" adopt --config `"$config`" --from `"$AdoptFrom`""
        Write-Host '  (its passphrase on stdin)'
    } elseif ($AdoptFrom -and (Test-Path (Join-Path $AdoptFrom 'auto-unlock-root.json'))) {
        Write-Host 'Djinn''s vault here is OS-rooted: only your own logon opens it, so adopt it as'
        Write-Host "$ownerAccount, from an elevated prompt of that same account (not another administrator),"
        Write-Host 'with Djinn closed and unlocked once (a persisted lock refuses the read):'
        Write-Host "  `"$exe`" adopt --config `"$config`" --from `"$AdoptFrom`""
        Write-Host '  It asks for the new passphrase twice; the copy is proven to open to the same identity.'
    } else {
        Write-Host "Create a vault from an elevated prompt: `"$exe`" init --config `"$config`""
    }
    Write-Host 'Then re-run the icacls line for the state directory from this script, and:'
    Write-Host "  Start-Service djinn-custodian; Start-ScheduledTask -TaskName $task"
}
Write-Host "Point SSH_AUTH_SOCK (or ssh's IdentityAgent) at $agentPipe to use the custodian's agent."
