# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import os,subprocess,json,shutil,re
repo=Path('/Users/markik/Code/worktrees/mere-nested-graph-view');out=Path(__file__).parent
compiler=Path('/Users/markik/.rustup/toolchains/1.98.1-x86_64-apple-darwin/bin')
env={**os.environ,'RUSTC':str(compiler/'rustc'),'RUSTDOC':str(compiler/'rustdoc'),'DYLD_LIBRARY_PATH':str(compiler.parent/'lib')}
commands=[(repo,n,['test','--locked','--offline','-p','graphshell','--no-default-features','--features','web',n,'--lib','-j1']) for n in ['viewer_commands','canvas_gestures','canvas_controls','host_','presentation_binding','presentation_editor','node_presentation','coupling_edit']]
commands += [(repo,'canvas_field_runtime',['test','--locked','--offline','-p','pictograph','--features','canvas','--lib','field_runtime','-j1'])]
web=repo/'ports/graphshell/web'
commands += [(web,'viewer-build',['build','--locked','--offline','--target','wasm32-unknown-unknown','--no-default-features','-j1']),
 (web,'product-build',['build','--locked','--offline','--target','wasm32-unknown-unknown','--no-default-features','--features','product,applets','-j1']),
 (web,'main-check',['check','--locked','--offline','--target','wasm32-unknown-unknown','--no-default-features','--features','main-page,applets','-j1'])]
records=[]
for cwd,name,args in commands:
 print('START',name,flush=True)
 log=out/(name+'.log')
 with log.open('w') as stream:r=subprocess.run([str(compiler/'cargo'),*args],cwd=cwd,env=env,stdout=stream,stderr=subprocess.STDOUT)
 records.append({'name':name,'command':[str(compiler/'cargo'),*args],'cwd':str(cwd),'exit_code':r.returncode,'log':str(log),'tests':re.findall(r'test result: ([^\n]+)',log.read_text())})
 (out/'gates.json').write_text(json.dumps(records,indent=2)+'\n')
 print('DONE',name,r.returncode,flush=True)
 if r.returncode:raise SystemExit(r.returncode)
 if name in ['viewer-build','product-build']:
  shutil.copy2(web/'target/wasm32-unknown-unknown/debug/graphshell_web.wasm',out/(name+'.wasm'))
