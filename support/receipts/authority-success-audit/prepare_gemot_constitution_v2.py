"""Preserve a pre-build supplier miss and use an available retained family."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json
HERE = Path(__file__).resolve().parent
FP = Path('C:/t/cargo-targets/mere/build/debug/.fingerprint')
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
old = HERE / 'run_gemot_constitution_probe.py'
failed_family = FP / 'gemot-d1280ffacb26c7cf/lib-gemot.json'
deps = {r[1]: r[3] for r in json.loads(failed_family.read_text())['deps']}
observed = []
for folder in FP.iterdir():
    if not folder.is_dir() or not folder.name.startswith(('identity-', 'personae-')): continue
    for marker in [folder / 'lib-identity', folder / 'lib-personae']:
        if not marker.is_file(): continue
        crate = marker.name[4:]
        artifact = FP.parent / 'deps' / ('lib'+crate+'-'+folder.name.rsplit('-',1)[1]+'.rlib')
        encoded = marker.read_text().strip()
        if len(encoded) != 16: continue
        observed.append({'path': str(marker), 'sha256': sha(marker),
                         'value': int.from_bytes(bytes.fromhex(encoded), 'little'), 'artifact_exists': artifact.is_file()})
assert not any(r['value'] == deps['identity'] and r['artifact_exists'] for r in observed)
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-gemot-constitution.*'))
failure = {'date_utc': datetime.now(timezone.utc).isoformat(), 'runner_sha256': sha(old),
           'selected_family': {'path': str(failed_family), 'sha256': sha(failed_family)},
           'expected_identity_fingerprint': deps['identity'], 'observed': observed,
           'executed_compilation': False, 'executed_checks': 0,
           'qualification': 'Initial bounded supplier selection stopped before rustc: the named d128 family had no matching current identity marker/artifact pair. A targeted inventory found two other complete ten-supplier families, so this is neither a Cargo build failure nor an unavailable diagnostic.'}
with (HERE / 'gemot_constitution_v1_supplier_miss.json').open('x', encoding='utf-8') as out:
    json.dump(failure, out, indent=2); out.write('\n')
text = old.read_text(encoding='utf-8')
assert text.count('gemot-d1280ffacb26c7cf') == 1
text = text.replace('gemot-d1280ffacb26c7cf', 'gemot-2a2dd68ccc68833a')
text = text.replace("NAME = 'gemot_constitution_probe'", "NAME = 'gemot_constitution_probe_v2'")
text = text.replace("(NAME + '.rs')", "'gemot_constitution_probe.rs'")
with (HERE / 'run_gemot_constitution_probe_v2.py').open('x', encoding='utf-8') as out: out.write(text)
print('Preserved initial pre-build supplier miss; prepared v2 with complete retained family 2a2dd68ccc68833a')
