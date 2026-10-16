# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

param(
    [Parameter(Mandatory=$true)][string]$TestBinary,
    [Parameter(Mandatory=$true)][string]$ModelDirectory,
    [Parameter(Mandatory=$true)][string]$WorkDirectory,
    [Parameter(Mandatory=$true)][ValidateSet('bf16','f16','q8-block32','q4-block32')][string]$Format,
    [Parameter(Mandatory=$true)][ValidateSet('burn-flex','burn-wgpu')][string]$Loader
)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $WorkDirectory | Out-Null
$env:ESP_SMOLLM2_DIR = $ModelDirectory
$env:ESP_FORMAT_WORK_DIR = $WorkDirectory
$env:ESP_WEIGHT_FORMAT = $Format
$env:ESP_WEIGHT_LOADER = $Loader
$taskStem = "$Format-$Loader"

function Read-GpuMemory {
    $taskRows = @(& nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv,noheader,nounits |
        ConvertFrom-Csv -Header Name,TotalMiB,FreeMiB)
    if ($LASTEXITCODE -ne 0 -or $taskRows.Count -ne 1) {
        throw 'This receipt runner requires one NVIDIA GPU and a working nvidia-smi.'
    }
    return @{
        name = $taskRows[0].Name.Trim()
        total_bytes = [uint64]$taskRows[0].TotalMiB.Trim() * 1MB
        free_bytes = [uint64]$taskRows[0].FreeMiB.Trim() * 1MB
    }
}

$taskGpuBefore = Read-GpuMemory
$taskOs = Get-CimInstance Win32_OperatingSystem
$taskStart = [DateTime]::UtcNow
$taskPeakWorkingSet = [int64]0
$taskGpuPeakUsed = $taskGpuBefore.total_bytes - $taskGpuBefore.free_bytes
$taskSamples = 0
$taskProcess = Start-Process -FilePath (Resolve-Path -LiteralPath $TestBinary).Path -ArgumentList @(
    '--ignored','--exact','measure_real_weight_format','--nocapture','--test-threads=1'
) -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $WorkDirectory "$taskStem.stdout.log") `
    -RedirectStandardError (Join-Path $WorkDirectory "$taskStem.stderr.log")
$null = $taskProcess.Handle
do {
    $taskProcess.Refresh()
    $taskPeakWorkingSet = [Math]::Max($taskPeakWorkingSet, $taskProcess.PeakWorkingSet64)
    if ($taskSamples % 4 -eq 0) {
        $taskGpuSample = Read-GpuMemory
        $taskGpuPeakUsed = [Math]::Max($taskGpuPeakUsed, $taskGpuSample.total_bytes - $taskGpuSample.free_bytes)
    }
    $taskSamples += 1
} while (-not $taskProcess.WaitForExit(250))
$taskProcess.WaitForExit()
$taskExitCode = $taskProcess.ExitCode
$taskSoftwarePath = Join-Path $WorkDirectory "$taskStem.json"
$taskFresh = (Test-Path -LiteralPath $taskSoftwarePath) -and
    (Get-Item -LiteralPath $taskSoftwarePath).LastWriteTimeUtc -ge $taskStart
$taskReceipt = [ordered]@{
    recorded_at_utc = $taskStart.ToString('o')
    command = @($TestBinary,'--ignored','--exact','measure_real_weight_format','--nocapture','--test-threads=1')
    binary_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $TestBinary).Hash.ToLowerInvariant()
    exit_code = $taskExitCode
    fresh_software_receipt = $taskFresh
    elapsed_seconds = ([DateTime]::UtcNow - $taskStart).TotalSeconds
    process_peak_working_set_bytes = $taskPeakWorkingSet
    host_memory_bytes = [uint64]$taskOs.TotalVisibleMemorySize * 1KB
    host_available_bytes_before = [uint64]$taskOs.FreePhysicalMemory * 1KB
    gpu_before = $taskGpuBefore
    gpu_global_peak_used_bytes = $taskGpuPeakUsed
    samples = $taskSamples
    gpu_measurement_scope = 'Entire GPU including other processes; not an isolated per-process VRAM measurement.'
}
$taskReceipt | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 (Join-Path $WorkDirectory "$taskStem.host.json")
$taskReceipt | ConvertTo-Json -Depth 8
if ($taskExitCode -ne 0 -or -not $taskFresh) {
    Get-Content -LiteralPath (Join-Path $WorkDirectory "$taskStem.stdout.log") -Tail 45
    Get-Content -LiteralPath (Join-Path $WorkDirectory "$taskStem.stderr.log") -Tail 45
    exit 1
}
