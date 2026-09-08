"""Apply only the disposable retention experiment, never the shared checkout."""
from pathlib import Path
import difflib
import hashlib
import json
import shutil

base = Path(__file__).resolve().parent
shutil.copy2(base / 'baseline-Cargo.toml', base / 'probe/Cargo.toml')
source = base / 'source/components/genet-scripted-dom/lib.rs'
baseline_bytes = source.read_bytes()
old = baseline_bytes.decode('utf-8').replace('\r\n', '\n')
needle = '            self.release_subtree(child);'
assert old.count(needle) == 2, 'expected exactly text and fragment replacement sites'
new = old.replace(needle, '            // Research candidate: orphan; pin-aware collection owns reclamation.')
patch = ''.join(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
    fromfile='a/components/genet-scripted-dom/lib.rs', tofile='b/components/genet-scripted-dom/lib.rs'))
(base / 'retention-candidate.patch').write_text(patch, encoding='utf-8')
source.write_text(new, encoding='utf-8')
(base / 'candidate-source.json').write_text(json.dumps({
    'scope': 'disposable snapshot only; no production fix',
    'baseline_sha256': hashlib.sha256(baseline_bytes).hexdigest(),
    'candidate_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'patch_sha256': hashlib.sha256((base / 'retention-candidate.patch').read_bytes()).hexdigest(),
}, indent=2))
print('Prepared two-site orphan-until-collection candidate in scratch snapshot')
