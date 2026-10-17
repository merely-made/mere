$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$source = Join-Path $root 'scripts/cross-repo-smoke.ps1'
$tokens = $null
$parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($source, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Source parse failed.' }
$definition = $ast.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Assert-Local'}, $true).Extent.Text
if (-not $definition) { throw 'Source function absent.' }
Invoke-Expression $definition
# Only Assert-Local is evaluated. The product script's top-level Step/builds
# and system changes never execute. Cargo is a deterministic tree-output double.
function cargo { $global:LASTEXITCODE = 0; $script:mockTree }
$cases = @()
foreach ($case in @(
    @{name='registry_mislabelled_local'; tree='netrender v0.1.0'; failures=0},
    @{name='ssh_git_mislabelled_local'; tree='netrender v0.1.0 (ssh://git@example.invalid/netrender#12345678)'; failures=0},
    @{name='https_git_refused_control'; tree='netrender v0.1.0 (https://example.invalid/netrender#12345678)'; failures=1},
    @{name='path_accepted_control'; tree='netrender v0.1.0 (C:\fixture\netrender)'; failures=0}
)) {
    $script:failures = @()
    $script:mockTree = $case.tree
    Assert-Local 'audit' $root 'netrender'
    if ($script:failures.Count -ne $case.failures) { throw "Unexpected classification for $($case.name)" }
    $cases += [ordered]@{name=$case.name; tree=$case.tree; failures=@($script:failures)}
}
[ordered]@{
    source_function_sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($definition))).ToLowerInvariant()
    cases=$cases
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'script_local_source_cases.json') -Encoding utf8
