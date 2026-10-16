# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
# Enroll one additional LAN peer without replacing the existing SSH setup.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PeerAddress,
    [Parameter(Mandatory)][string]$PeerName,
    [Parameter(Mandatory)][string]$AuthorizedKeyFile
)
$ErrorActionPreference = 'Stop'
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'This peer enrollment requires native administrator elevation.'
}
if ($PeerName -notmatch '^[a-z0-9-]+$') { throw 'Invalid peer name.' }
$address = [Net.IPAddress]::Parse($PeerAddress)
if ($address.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork -or
    $PeerAddress -in @('0.0.0.0','255.255.255.255')) { throw 'One IPv4 peer is required.' }
$config = Join-Path $env:ProgramData 'ssh\sshd_config'
if (-not (Get-Content -LiteralPath $config -Raw).StartsWith('# Djinn LAN SSH enrollment')) {
    throw 'Preserve SSH configuration owned elsewhere.'
}
$key = (Get-Content -LiteralPath $AuthorizedKeyFile -Raw).Trim()
if ($key -notmatch '^ssh-ed25519 [A-Za-z0-9+/]+={0,2}( [^\r\n]*)?$') { throw 'One public Ed25519 key is required.' }
$ruleName = "Djinn-SSH-LAN-$PeerName"
$rule = Get-NetFirewallRule -Name $ruleName -ErrorAction SilentlyContinue
if ($rule) {
    $values = @(($rule | Get-NetFirewallAddressFilter).RemoteAddress)
    if ($values.Count -ne 1 -or $values[0] -ne $PeerAddress) { throw 'Existing peer rule differs; preserve it.' }
}
$keyPath = Join-Path $env:ProgramData 'ssh\administrators_authorized_keys'
$lines = @(Get-Content -LiteralPath $keyPath)
$identity = ($key -split ' ')[0..1] -join ' '
if (-not ($lines | Where-Object { (($_ -split ' ')[0..1] -join ' ') -eq $identity })) {
    [IO.File]::AppendAllText($keyPath, "`n$key`n", [Text.UTF8Encoding]::new($false))
}
& icacls.exe $keyPath /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Could not secure the administrator public key file.' }
if (-not $rule) {
    New-NetFirewallRule -Name $ruleName -DisplayName "Djinn SSH LAN peer: $PeerName" `
        -Direction Inbound -Action Allow -Protocol TCP -LocalPort 22 -RemoteAddress $PeerAddress -Profile Any | Out-Null
}
Get-Service sshd | Select-Object Name,Status,StartType
Get-NetFirewallRule -Name $ruleName | Get-NetFirewallAddressFilter | Select-Object RemoteAddress
