# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
# Run from an elevated PowerShell. Installs the inbox server for one LAN peer.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PeerAddress,
    [Parameter(Mandatory)][string]$LoginUser,
    [string]$AuthorizedKeyFile,
    [string]$ReportPath = (Join-Path $env:LOCALAPPDATA 'Djinn\ssh-server-setup.log')
)
$ErrorActionPreference = 'Stop'
$admin = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $admin.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this script in an elevated PowerShell window.'
}
New-Item -ItemType Directory -Path (Split-Path -Parent $ReportPath) -Force | Out-Null
Start-Transcript -Path $ReportPath -Append | Out-Null
$address = [Net.IPAddress]::Parse($PeerAddress)
if ($address.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork -or
    $PeerAddress -eq '0.0.0.0' -or $PeerAddress -eq '255.255.255.255') {
    throw 'PeerAddress must name one IPv4 peer.'
}
if ($LoginUser -notmatch '^[a-zA-Z0-9_.-]+$') { throw 'Invalid local account name.' }
Get-LocalUser -Name $LoginUser -ErrorAction Stop | Out-Null
$config = Join-Path $env:ProgramData 'ssh\sshd_config'
$marker = '# Djinn LAN SSH enrollment'
if ((Test-Path -LiteralPath $config) -and
    -not (Get-Content -LiteralPath $config -Raw).StartsWith($marker)) {
    throw "Existing SSH configuration is owned elsewhere: $config. Review it before installation."
}
$key = $null
if ($AuthorizedKeyFile) {
    $key = (Get-Content -LiteralPath $AuthorizedKeyFile -Raw).Trim()
    if ($key -notmatch '^ssh-ed25519 [A-Za-z0-9+/]+={0,2}( [^\r\n]*)?$') {
        throw 'AuthorizedKeyFile must contain one public Ed25519 key.'
    }
}
$existingDefaultRule = Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue
if ($existingDefaultRule -and -not (Test-Path -LiteralPath $config)) {
    throw 'An existing OpenSSH firewall rule needs review before a fresh installation.'
}
$capability = Get-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0'
if ($capability.State -ne 'Installed') {
    $result = Add-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0'
    if ($result.RestartNeeded) { throw 'OpenSSH installation requires a Windows restart; rerun afterward.' }
}
# Windows creates this rule during capability installation. Limit that rule
# before starting the new service, including when Wi-Fi is classified Public.
$defaultRule = Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue
if ($defaultRule) {
    $defaultRule | Set-NetFirewallRule -Enabled False
}
$ruleName = 'Djinn-SSH-LAN-Peer'
$rule = Get-NetFirewallRule -Name $ruleName -ErrorAction SilentlyContinue
if ($rule) {
    $filter = $rule | Get-NetFirewallAddressFilter
    $peerValues = @($filter.RemoteAddress)
    if ($peerValues.Count -ne 1 -or $peerValues[0] -ne $PeerAddress) {
        throw 'The existing Djinn SSH rule has a different peer; preserve it and review the change.'
    }
} else {
    New-NetFirewallRule -Name $ruleName -DisplayName 'Djinn SSH LAN peer' `
        -Direction Inbound -Action Allow -Protocol TCP -LocalPort 22 `
        -RemoteAddress $PeerAddress -Profile Any | Out-Null
}
if (-not (Test-Path -LiteralPath "$config.djinn-before") -and (Test-Path -LiteralPath $config)) {
    Copy-Item -LiteralPath $config -Destination "$config.djinn-before"
}
$text = @"
$marker
Port 22
PubkeyAuthentication yes
PasswordAuthentication no
KbdInteractiveAuthentication no
AuthenticationMethods publickey
AllowUsers $LoginUser
Subsystem sftp sftp-server.exe
Match Group administrators
    AuthorizedKeysFile __PROGRAMDATA__/ssh/administrators_authorized_keys
"@
[IO.File]::WriteAllText($config, $text.Replace("`r`n", "`n") + "`n", [Text.UTF8Encoding]::new($false))
$sshd = Join-Path $env:WINDIR 'System32\OpenSSH\sshd.exe'
# A fresh inbox installation has no host keys until the service's first run.
# Generate missing keys before validating; -A leaves existing host keys intact.
& (Join-Path $env:WINDIR 'System32\OpenSSH\ssh-keygen.exe') -A
if ($LASTEXITCODE -ne 0) { throw 'Could not initialize SSH host keys.' }
& $sshd -t
if ($LASTEXITCODE -ne 0) { throw 'sshd configuration validation failed.' }
if ($key) {
    $keyPath = Join-Path $env:ProgramData 'ssh\administrators_authorized_keys'
    $lines = if (Test-Path -LiteralPath $keyPath) { @(Get-Content -LiteralPath $keyPath) } else { @() }
    if ($lines -notcontains $key) {
        [IO.File]::WriteAllText($keyPath, (($lines + $key) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
    }
    & icacls.exe $keyPath /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not secure administrators_authorized_keys.' }
}
Set-Service -Name sshd -StartupType Automatic
Restart-Service -Name sshd
Get-Service -Name sshd | Select-Object Name, Status, StartType
Get-ChildItem (Join-Path $env:ProgramData 'ssh') -Filter 'ssh_host_*_key.pub' | ForEach-Object {
    & ssh-keygen.exe -lf $_.FullName
}
Stop-Transcript | Out-Null
