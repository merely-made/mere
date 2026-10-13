# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
from pathlib import Path
import os,subprocess,json
repo=Path('/Users/markik/Code/worktrees/mere-nested-graph-view');web=repo/'ports/graphshell/web';out=Path(__file__).parent/'wasm-gates-final';out.mkdir(exist_ok=True)
compiler=Path('/Users/markik/.rustup/toolchains/1.98.1-x86_64-apple-darwin/bin')
env={**os.environ,'RUSTC':str(compiler/'rustc'),'RUSTDOC':str(compiler/'rustdoc'),'DYLD_LIBRARY_PATH':str(compiler.parent/'lib')}
rows=[]
for name,features in [('viewer',['--no-default-features']),('product',['--no-default-features','--features','product,applets']),('main-page',['--no-default-features','--features','main-page,applets']),('default',['--features','applets'])]:
 cmd=[str(compiler/'cargo'),'check','--locked','--offline','--target','wasm32-unknown-unknown',*features,'-j1'];print('START',name,flush=True)
 with (out/(name+'.log')).open('w') as f:r=subprocess.run(cmd,cwd=web,env=env,stdout=f,stderr=subprocess.STDOUT)
 rows.append({'name':name,'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'command':cmd,'exit_code':r.returncode});(out/'gates.json').write_text(json.dumps(rows,indent=2)+'\n');print('DONE',name,r.returncode,flush=True)
 if r.returncode:raise SystemExit(r.returncode)
