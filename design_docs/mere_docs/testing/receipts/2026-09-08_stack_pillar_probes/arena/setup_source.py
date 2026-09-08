"""Recreate the pinned source for this disposable arena experiment.

Usage: python setup_source.py <path-to-genet-repository>
Run cargo test --locked --manifest-path probe/Cargo.toml afterward.
The source tree is copied from Git, not the checkout's uncommitted files.
"""
from pathlib import Path
import io
import subprocess
import sys
import zipfile

REVISION = 'ee0b314b3e9ac4a2fadb07fb7816990fa3f2b71d'
base = Path(__file__).resolve().parent
source = base / 'source'
if source.exists():
    raise SystemExit('source already exists; use a fresh experiment directory')
archive = subprocess.check_output([
    'git', '-C', sys.argv[1], 'archive', '--format=zip', REVISION,
    'Cargo.toml', 'rust-toolchain.toml', 'components', 'support/patches',
])
with zipfile.ZipFile(io.BytesIO(archive)) as z:
    for member in z.namelist():
        target = (source / member).resolve()
        if not target.is_relative_to(source.resolve()):
            raise SystemExit('unsafe archive member: ' + member)
    z.extractall(source)
print('Restored pinned source', REVISION)
