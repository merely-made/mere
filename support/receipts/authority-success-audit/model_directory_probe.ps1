# Evaluate only the exact production directory guard AST. No fetch or file writes
# from the production script are executed; only this audit receipt is written.
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$source = Join-Path $repo 'ports/distillery/probe/fetch-model-matrix.ps1'
$receiptPath = Join-Path $PSScriptRoot 'model_directory_probe_receipt.json'
if (Test-Path -LiteralPath $receiptPath) { throw 'Preserve the existing receipt' }
$before = (Get-FileHash -Algorithm SHA256 -LiteralPath $source).Hash.ToLowerInvariant()
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($source, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Production script did not parse' }
$loop = @($ast.EndBlock.Statements | Where-Object {
    $_ -is [System.Management.Automation.Language.ForEachStatementAst] -and $_.Variable.Extent.Text -eq '$model'
})
if ($loop.Count -ne 1) { throw 'Expected one model loop' }
$statements = @($loop[0].Body.Statements)[0..2]
if ($statements[0].Extent.Text -notlike '$relativeDirectory =*' -or
    $statements[1] -isnot [System.Management.Automation.Language.IfStatementAst] -or
    $statements[2].Extent.Text -notlike '$directory = Join-Path*') {
    throw 'Guard shape changed; inspect before running'
}
$exact = ($statements | ForEach-Object { $_.Extent.Text }) -join "`n"
$code = [scriptblock]::Create($exact + "`n" + '[System.IO.Path]::GetFullPath($directory)')
$modelsRoot = Join-Path $repo 'models'
$rows = foreach ($url in @('/models/..', '/models/.', '/models/all-MiniLM-L6-v2')) {
    $model = [pscustomobject]@{ model_base_url = $url }
    $directory = & $code
    [pscustomobject]@{
        input = $url
        accepted_by_production_guard = $true
        resolved = $directory
        is_model_root_itself = $directory.TrimEnd('\') -eq $modelsRoot.TrimEnd('\')
        is_repository_root = $directory.TrimEnd('\') -eq $repo.TrimEnd('\')
        strictly_below_model_root = $directory.StartsWith($modelsRoot.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)
    }
}
if (-not $rows[0].is_repository_root -or $rows[0].strictly_below_model_root) { throw 'Parent escape not observed' }
if (-not $rows[1].is_model_root_itself -or $rows[1].strictly_below_model_root) { throw 'Root alias not observed' }
if (-not $rows[2].strictly_below_model_root) { throw 'Ordinary child control failed' }
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $source).Hash.ToLowerInvariant() -ne $before) { throw 'Source changed during observation' }
$receipt = [ordered]@{
    date_utc = [DateTime]::UtcNow.ToString('O')
    source = 'ports/distillery/probe/fetch-model-matrix.ps1'
    source_sha256 = $before
    exact_production_guard = $exact
    checks = 3
    exit = 0
    rows = $rows
    scope = 'The production guard AST is extracted and evaluated directly with synthetic model_base_url values. No curl, network, model download, directory creation, move or production/original-test edit is executed. This demonstrates malformed caller-supplied matrix paths being accepted; no externally reachable attack, actual overwrite, or current committed matrix damage is claimed.'
}
$json = $receipt | ConvertTo-Json -Depth 8
[System.IO.File]::WriteAllText($receiptPath, $json + "`n", [System.Text.UTF8Encoding]::new($false))
$json
