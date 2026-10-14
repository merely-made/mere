# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
# Give new SSH sessions a POSIX shell for Codex's remote bootstrap probes.
[CmdletBinding()]
param([switch]$Restore)
$ErrorActionPreference = 'Stop'
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'This SSH shell setup requires native administrator elevation.'
}
$config = Join-Path $env:ProgramData 'ssh\sshd_config'
if (-not (Get-Content -LiteralPath $config -Raw).StartsWith('# Djinn LAN SSH enrollment')) {
    throw 'Preserve SSH configuration owned elsewhere.'
}
$shell = 'C:\Program Files\Git\usr\bin\bash.exe'
if (-not (Test-Path -LiteralPath $shell -PathType Leaf)) { throw 'The tested Git Bash executable is missing.' }
$registryPath = 'HKLM:\SOFTWARE\OpenSSH'
$snapshotPath = Join-Path $env:ProgramData 'ssh\codex-default-shell.json'
$names = @('DefaultShell','DefaultShellCommandOption')
$desired = @{ DefaultShell = $shell; DefaultShellCommandOption = '-c' }
$registry = Get-Item -LiteralPath $registryPath
$current = @{}
foreach ($name in $names) {
    $exists = $name -in $registry.GetValueNames()
    $current[$name] = @{
        exists = $exists
        value = if ($exists) { $registry.GetValue($name, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) } else { $null }
        kind = if ($exists) { $registry.GetValueKind($name).ToString() } else { $null }
    }
}
if (Test-Path -LiteralPath $snapshotPath) {
    $snapshot = Get-Content -LiteralPath $snapshotPath -Raw | ConvertFrom-Json -AsHashtable
    if ($snapshot.schema -ne 'codex.ssh-shell/snapshot-v1') { throw 'Snapshot is owned elsewhere.' }
} else {
    if ($Restore) { throw 'No original SSH shell snapshot exists.' }
    foreach ($name in $names) {
        if ($current[$name].exists -and $current[$name].value -ne $desired[$name]) {
            throw "Existing $name is owned elsewhere; preserve it."
        }
    }
    $snapshot = @{ schema = 'codex.ssh-shell/snapshot-v1'; original = $current }
    [IO.File]::WriteAllText($snapshotPath, (($snapshot | ConvertTo-Json -Depth 5) + "`n"), [Text.UTF8Encoding]::new($false))
    & icacls.exe $snapshotPath /inheritance:r /grant:r '*S-1-5-18:F' '*S-1-5-32-544:F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not protect the SSH shell snapshot.' }
}
foreach ($name in $names) {
    $original = $snapshot.original[$name]
    $matchesOriginal = $current[$name].exists -eq $original.exists -and $current[$name].value -eq $original.value -and $current[$name].kind -eq $original.kind
    $matchesDesired = $current[$name].exists -and $current[$name].value -eq $desired[$name]
    if (-not ($matchesOriginal -or $matchesDesired)) { throw "Concurrent $name change detected; preserve it." }
}
foreach ($name in $names) {
    if ($Restore) {
        $original = $snapshot.original[$name]
        if ($original.exists) {
            New-ItemProperty -LiteralPath $registryPath -Name $name -Value $original.value -PropertyType $original.kind -Force | Out-Null
        } else {
            Remove-ItemProperty -LiteralPath $registryPath -Name $name -ErrorAction SilentlyContinue
        }
    } else {
        New-ItemProperty -LiteralPath $registryPath -Name $name -Value $desired[$name] -PropertyType String -Force | Out-Null
    }
}
# Registry selection applies to subsequent sessions; preserve running sshd and clients.
Get-ItemProperty -LiteralPath $registryPath | Select-Object DefaultShell,DefaultShellCommandOption
Get-Service sshd | Select-Object Name,Status,StartType
