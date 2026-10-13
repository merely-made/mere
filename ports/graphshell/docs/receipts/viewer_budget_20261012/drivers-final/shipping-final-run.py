# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
from pathlib import Path
import subprocess,os,json
repo=Path('/Users/markik/Code/worktrees/mere-nested-graph-view');out=Path(__file__).parent/'shipping-final';out.mkdir(exist_ok=True)
compiler=Path('/Users/markik/.rustup/toolchains/1.98.1-x86_64-apple-darwin/bin')
env={**os.environ,'RUSTC':str(compiler/'rustc'),'RUSTDOC':str(compiler/'rustdoc'),'DYLD_LIBRARY_PATH':str(compiler.parent/'lib')}
cmd=['/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3',str(repo/'ports/graphshell/web/scripts/build-viewer.py'),'--out-dir',str(out/'pkg'),'--cargo',str(compiler/'cargo'),'--wasm-bindgen','/Users/markik/.cargo/bin/wasm-bindgen','--offline']
with (out/'build-and-gate.log').open('w') as f:r=subprocess.run(cmd,cwd=repo,env=env,stdout=f,stderr=subprocess.STDOUT)
print('SHIPPING GATE',r.returncode,flush=True)
if r.returncode:raise SystemExit(r.returncode)
print((out/'pkg/viewer-delivery.json').read_text(),flush=True)
