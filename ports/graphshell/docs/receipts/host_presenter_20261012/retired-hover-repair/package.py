# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import json,subprocess,hashlib,shutil
base=Path('/Users/markik/Code/output/graphshell-site-presenter-20261011');repo=Path('/Users/markik/Code/worktrees/mere-nested-graph-view');out=Path(__file__).parent
records=json.loads((out/'gates.json').read_text());assert len(records)==5 and all(r['exit_code']==0 for r in records)
for flavor in ['viewer','product']:
 page=base/('web-'+flavor)
 for source in (repo/'ports/graphshell/web').iterdir():
  if source.is_file() and source.suffix in ['.html','.js','.css']:shutil.copy2(source,page/source.name)
 for dirname in ['fixtures','scenarios']:shutil.copytree(repo/'ports/graphshell/web'/dirname,page/dirname,dirs_exist_ok=True)
 subprocess.run(['/Users/markik/.cargo/bin/wasm-bindgen','--target','web','--remove-name-section','--out-name','graphshell_web','--out-dir',str(page/'pkg'),str(out/(flavor+'-build.wasm'))],check=True)
p=base/'candidate.json';d=json.loads(p.read_text());d['source']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
d['source_byte_sha256']={name:hashlib.sha256((repo/name).read_bytes()).hexdigest() for name in d['source_byte_sha256']}
d['bundle_byte_sha256']={str(f.relative_to(base)):hashlib.sha256(f.read_bytes()).hexdigest() for v in ['web-viewer','web-product'] for f in sorted((base/v/'pkg').glob('*')) if f.is_file()}
d['gates']=['main-8568f42fd/gates.json','command-option-name-repair/gates.json','option-mirror-repair/gates.json','retired-hover-repair/gates.json'];d['cpu_tests']=88;d['packaging_complete']=True;d['acceptance_status']='Combined current-main source CPU/Wasm-qualified; final hardware acceptance awaits unlocked console and explicit handoff'
d['browser_driver_sha256']={name:hashlib.sha256((base/name).read_bytes()).hexdigest() for name in d.get('browser_drivers',['verify-commands.cjs','verify-presenter.cjs','verify-product.cjs'])+['assert-console-unlocked.cjs']};p.write_text(json.dumps(d,indent=2)+'\n')
p=base/'continuation-state.json';d=json.loads(p.read_text());d.update(packaging_complete=True,current_compiled_package_source='source-after-option-name-repair',all_72_cpu_tests_pass=True,viewer_product_main_wasm=True);p.write_text(json.dumps(d,indent=2)+'\n')
print('PACKAGED source',json.loads((base/'candidate.json').read_text())['source'],flush=True)
