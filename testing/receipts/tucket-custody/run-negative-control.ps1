# Historical negative-control recipe. No serial or RF calls are made here.
# Restores the exact original source in finally, then rebuilds the clean probe.
$ErrorActionPreference='Stop'
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$sourcePath=Join-Path $repo 'ports/signalman/src/authority/tucket.rs'
$backup=Join-Path $PSScriptRoot 'tucket-authority-clean.rs'
if((Get-FileHash $sourcePath).Hash -ne (Get-FileHash $backup).Hash){throw 'Source changed since the clean checkpoint.'}
$before=(Get-FileHash $sourcePath).Hash
$original=[IO.File]::ReadAllText($sourcePath)
$needle='        let wallet = self.tucket_wallet()?;'
$fallback=@'
        // FORBIDDEN FALLBACK, restored only for this negative control.
        if self.tucket_wallet().is_err() {
            let local = personae::InMemoryProvider::from_seed([0x47; 32]);
            let material = TucketStationMaterial::derive(&local, device.as_uuid().as_bytes(), realm)
                .map_err(CustodyClientError::Release)?;
            return Ok(ReleasedTucketStation {
                material,
                lease: TucketCustodyLease {
                    authority: self.clone(), wallet: local.master_public_key().to_bytes(),
                    closed: Arc::new(AtomicBool::new(false)),
                    stop: Arc::new(AtomicBool::new(false)), join: None,
                },
            });
        }
        let wallet = self.tucket_wallet()?;
'@
if(($original.Split($needle).Count-1) -ne 1){throw 'Require exactly one release entry point.'}
$env:CARGO_HOME='C:\t\cargo-homes\mere-upstreams'
$env:CARGO_TARGET_DIR='C:\t\cargo-targets\mere'
$failure=$null
try {
    [IO.File]::WriteAllText($sourcePath,$original.Replace($needle,$fallback))
    $mutated=(Get-FileHash $sourcePath).Hash
    cargo build --locked --offline -p mere-signalman --example tucket_custody_receipt -j 2 *> (Join-Path $PSScriptRoot 'negative-build.log')
    if($LASTEXITCODE -ne 0){throw 'Negative-control build failed.'}
    & (Join-Path $repo 'testing/run_tucket_custody.ps1') -HostBinary 'C:\t\cargo-targets\mere\debug\tucket-custody-receipt-host.exe' -ProbeBinary 'C:\t\cargo-targets\mere\debug\examples\tucket_custody_receipt.exe' -OutputDirectory (Join-Path $PSScriptRoot 'negative-control') -ExpectFallbackFailure
} catch {$failure=$_}
finally {
    Copy-Item -LiteralPath $backup -Destination $sourcePath
    # Copy-Item preserves the old mtime. Cargo must see the restoration as new.
    (Get-Item -LiteralPath $sourcePath).LastWriteTimeUtc=[DateTime]::UtcNow
    $restored=(Get-FileHash $sourcePath).Hash
    if($before -ne $restored){throw 'Exact source restoration failed.'}
    cargo build --locked --offline -p mere-signalman --example tucket_custody_receipt -j 2 *> (Join-Path $PSScriptRoot 'restored-build.log')
    $cleanExit=$LASTEXITCODE
    [ordered]@{before_sha256=$before;negative_sha256=$mutated;restored_sha256=$restored;clean_build_exit=$cleanExit;source_restored=($before -eq $restored)}|ConvertTo-Json|Set-Content (Join-Path $PSScriptRoot 'source-restoration.json')
    if($cleanExit -ne 0){throw 'Restored-source build failed.'}
}
if($failure){throw $failure}
