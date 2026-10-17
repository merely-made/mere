param([string]$Config,[string]$Mode)
$ErrorActionPreference='Stop'
$taskCfg=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
function nvidia-smi {$global:LASTEXITCODE=0; 'Synthetic adapter,4096,2048'}
function Get-CimInstance {[pscustomobject]@{TotalVisibleMemorySize=1048576;FreePhysicalMemory=524288}}
if($Mode -eq 'stages'){
 & (Join-Path $taskCfg.root 'support/run_resident_inference_stages.ps1') -Executable $taskCfg.empty -ModelDirectory $taskCfg.here -FormatWorkDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'stages')
}elseif($Mode -eq 'streaming'){
 & (Join-Path $taskCfg.root 'support/run_resident_layer_streaming.ps1') -TestBinary $taskCfg.fake -ModelDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'streaming')
}elseif($Mode -eq 'formats'){
 & (Join-Path $taskCfg.root 'support/run_resident_weight_formats.ps1') -TestBinary $taskCfg.fake -ModelDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'formats') -Format f16 -Loader burn-flex
}elseif($Mode -eq 'custody'){
 $output=Join-Path $taskCfg.cases 'custody';New-Item -ItemType Directory -Path $output|Out-Null
 $ProbeBinary=$taskCfg.fake;$taskDevice='00000000-0000-0000-0000-000000000001';$realm='synthetic-only'
 $taskRows=@()
 foreach($scriptName in @('run_sennet_custody.ps1','run_tucket_custody.ps1')){
  $taskTokens=$null;$taskErrors=$null
  $taskAst=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $taskCfg.root "testing/$scriptName"),[ref]$taskTokens,[ref]$taskErrors)
  if($taskErrors){throw 'Unexpected source parse error'}
  $taskFn=$taskAst.Find({param($taskNode) $taskNode -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $taskNode.Name -eq 'Invoke-Probe'},$true)
  Invoke-Expression $taskFn.Extent.Text
  $taskRow=Invoke-Probe ($scriptName.Replace('.ps1','')) 'synthetic-unused-endpoint' 'pending' $false
  if($taskRow.result -ne 'failed' -or $taskRow.exit_code -ne 0){throw 'Wrong probe control result'}
  $taskRows+=@{script=$scriptName;returned_without_error=$true;returned_result=$taskRow.result;child_exit=$taskRow.exit_code}
 }
 $taskRows|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $taskCfg.cases 'custody-functions.json')
}else{throw 'Unknown mode'}
