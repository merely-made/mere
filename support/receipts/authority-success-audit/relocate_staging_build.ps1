$ErrorActionPreference = 'Stop'
$sourceExpected = 'C:\Users\mark_\Code\repos\mere\support\receipts\authority-success-audit\cpu-staging-runtime\target\build'
$destinationExpected = 'C:\t\cargo-targets\mere\build'
$sourcePath = (Resolve-Path -LiteralPath $sourceExpected).Path
$destinationPath = [IO.Path]::GetFullPath($destinationExpected)
if ($sourcePath -ne $sourceExpected -or $destinationPath -ne $destinationExpected) { throw 'Resolved build path differs from reviewed source/destination' }
if (Test-Path -LiteralPath $destinationPath) { throw 'Shared build destination already has an owner; do not overwrite it' }
$marker = Join-Path $sourcePath 'CACHEDIR.TAG'
if (-not (Get-Content -LiteralPath $marker -Raw).StartsWith('Signature: 8a477f597d28d172789f06886806bc55')) { throw 'Source is not Cargo marker-identified generated output' }
$owners = @(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(cargo|rustc|rust-lld|link|clang).*exe$' -and $_.CommandLine -like '*cpu-staging-runtime*' })
if ($owners.Count -ne 0) { throw 'Build still has a live compiler owner' }
$removed = @()
$deps = Join-Path $sourcePath 'debug\deps'
foreach ($file in Get-ChildItem -LiteralPath $deps -Filter 'authority_success_refresh_staging-*' -File) {
    if ($file.Extension -notin @('.exe','.pdb')) { continue }
    $bytes = [IO.File]::ReadAllBytes($file.FullName)
    $valid = ($file.Extension -eq '.exe' -and $bytes[0] -eq 0x4d -and $bytes[1] -eq 0x5a) -or ($file.Extension -eq '.pdb' -and [Text.Encoding]::ASCII.GetString($bytes,0,19).StartsWith('Microsoft C/C++ MSF'))
    if (-not $valid) { throw 'Unexpected generated file marker' }
    $removed += @{ path=$file.FullName; bytes=$file.Length; sha256=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
    Remove-Item -LiteralPath $file.FullName
}
$cacheBytes = (Get-ChildItem -LiteralPath $sourcePath -File -Recurse | Measure-Object -Property Length -Sum).Sum
Move-Item -LiteralPath $sourcePath -Destination $destinationPath
$oldParent = Split-Path -Parent $sourcePath
if (@(Get-ChildItem -LiteralPath $oldParent -Force).Count -ne 0) { throw 'Unexpected files remain in generated local target; preserve them' }
Remove-Item -LiteralPath $oldParent
$receipt = @{ source=$sourcePath; destination=$destinationPath; cache_bytes=$cacheBytes; marker='Cargo CACHEDIR.TAG signature'; live_owners_before_move=0; removed_outputs=$removed; source_directory_removed=(-not (Test-Path -LiteralPath $oldParent)); isolated_target_retained=$false; qualification='A local Cargo build-dir setting redirected the CPU gate despite CARGO_TARGET_DIR. Only marker-identified generated output with no live owner was moved to the approved stable shared build directory; only this audit package executable/PDB were deleted. The fresh reusable supplier cache remains owned by the active full-source audit. Future gates must also override build.build-dir explicitly. No source or prior evidence deleted.' }
$receiptPath = 'C:\Users\mark_\Code\repos\mere\support\receipts\authority-success-audit\staging_build_relocation_receipt.json'
if (Test-Path -LiteralPath $receiptPath) { throw 'Receipt already exists' }
[IO.File]::WriteAllText($receiptPath, ($receipt | ConvertTo-Json -Depth 8) + "`n")
$receipt | ConvertTo-Json -Depth 8
