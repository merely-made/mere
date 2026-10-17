# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$ModelDirectory,
    [Parameter(Mandatory=$true)][string]$FormatWorkDirectory,
    [Parameter(Mandatory=$true)][string]$WorkDirectory
)
$ErrorActionPreference='Stop'
$taskExecutable=(Resolve-Path -LiteralPath $Executable).Path
$taskModel=(Resolve-Path -LiteralPath $ModelDirectory).Path
$taskFormats=(Resolve-Path -LiteralPath $FormatWorkDirectory).Path
[void](New-Item -ItemType Directory -Path $WorkDirectory -Force)
$taskWork=(Resolve-Path -LiteralPath $WorkDirectory).Path
$env:ESP_STAGE_WORK_DIR=$taskWork
$env:ESP_SMOLLM2_DIR=$taskModel
$env:ESP_FORMAT_WORK_DIR=$taskFormats
$taskOut=Join-Path $taskWork 'native.stdout.log'
$taskErr=Join-Path $taskWork 'native.stderr.log'
$taskArguments=@('--ignored','--exact','real_smollm2_f16_stages_match_and_recover_under_supervised_leases','--test-threads=1','--nocapture')
$taskGpuBefore=(& nvidia-smi --query-gpu=name,driver_version,memory.total,memory.used --format=csv,noheader,nounits 2>$null | Out-String).Trim()
$taskStarted=[DateTime]::UtcNow
$taskProcess=Start-Process -FilePath $taskExecutable -ArgumentList $taskArguments -PassThru -WindowStyle Hidden -RedirectStandardOutput $taskOut -RedirectStandardError $taskErr
$taskPeakHost=[long]0
$taskPeakGpuMiB=[long]0
$taskSamples=0
while (-not $taskProcess.HasExited) {
    $taskProcess.Refresh()
    try { $taskPeakHost=[Math]::Max($taskPeakHost,$taskProcess.PeakWorkingSet64) } catch {}
    $taskUsed=(& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>$null | Select-Object -First 1)
    if ($LASTEXITCODE -eq 0 -and $taskUsed -match '^\s*\d+\s*$') {
        $taskPeakGpuMiB=[Math]::Max($taskPeakGpuMiB,[long]$taskUsed)
        $taskSamples++
    }
    Start-Sleep -Milliseconds 250
    $taskProcess.Refresh()
}
$taskProcess.WaitForExit()
$taskEnded=[DateTime]::UtcNow
$taskProfile=[ordered]@{
    schema='esp.inference-stage-host-profile/v1'
    machine=$env:COMPUTERNAME
    os=[System.Runtime.InteropServices.RuntimeInformation]::OSDescription
    executable=$taskExecutable
    executable_sha256=(Get-FileHash -LiteralPath $taskExecutable -Algorithm SHA256).Hash.ToLowerInvariant()
    arguments=$taskArguments
    source_model=$taskModel
    source_qualification=$taskFormats
    work=$taskWork
    started_utc=$taskStarted.ToString('o')
    ended_utc=$taskEnded.ToString('o')
    wall_seconds=($taskEnded-$taskStarted).TotalSeconds
    exit_code=$taskProcess.ExitCode
    peak_process_working_set_bytes=$taskPeakHost
    sampled_whole_gpu_peak_bytes=$taskPeakGpuMiB*1024*1024
    gpu_samples=$taskSamples
    gpu_before=$taskGpuBefore
    scope='Three separately rooted software devices and libraries on one physical computer/GPU. Process peak includes imports, shared tensors, stages, KV and fallback. GPU samples include unrelated processes. Timings are execution costs, not isolated throughput.'
    stdout_sha256=(Get-FileHash -LiteralPath $taskOut -Algorithm SHA256).Hash.ToLowerInvariant()
    stderr_sha256=(Get-FileHash -LiteralPath $taskErr -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskProfile | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskWork 'native.host.json')
Get-Content -LiteralPath $taskOut -Tail 10
Get-Content -LiteralPath $taskErr -Tail 5
exit $taskProcess.ExitCode
