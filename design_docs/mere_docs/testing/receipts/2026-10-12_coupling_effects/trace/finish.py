# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import subprocess,hashlib,json,os,time
root=Path('.');receipt=Path('C:/Users/mark_/Code/testing/mere/coupling-trace')
env=dict(os.environ,CARGO_TARGET_DIR='C:/t/cargo-targets/mere')
controls=[
 ('negative-cap-skips-physics','crates/conatus/seiche/src/coupling_trace.rs','let capture = trace.has_room();','let capture = trace.has_room();\n            if !capture { continue; }',['cargo','test','-p','seiche','--no-default-features','--lib','coupling_trace::tests::capped_latest_tick','--locked','--offline','-j','2','--','--nocapture'],'capped_latest_tick_does_not_skip_physics_or_accumulate_history'),
 ('negative-local-index-identity','crates/canvas/pictograph/src/canvas/seiche_bridge.rs','member: node.id.as_u128(),','member: key.index() as u128,',['cargo','test','-p','pictograph','--features','canvas','--lib','canvas::coupling_trace::tests::','--locked','--offline','-j','2','--','--nocapture'],'canvas_live_rebuild_keeps_exact_owners_and_clears_retired_binding_trace')]
results=[]
for name,fn,before,after,cmd,test in controls:
 p=root/fn;original=p.read_bytes();digest=hashlib.sha256(original).hexdigest();text=original.decode();assert text.count(before)==1
 print('Starting '+name,flush=True)
 try:
  p.write_bytes(text.replace(before,after).encode())
  with (receipt/(name+'.log')).open('wb') as log:
   result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
  output=(receipt/(name+'.log')).read_text(encoding='utf-8')
  assert result.returncode != 0 and ('FAILED' in output) and (test in output),(name,result.returncode)
  print(name+': expected test failure',flush=True)
 finally:
  p.write_bytes(original)
  assert hashlib.sha256(p.read_bytes()).hexdigest()==digest
 results.append(dict(name=name,command=cmd,expected_test=test,exit_code=result.returncode,restored=True,sha256_before=digest,sha256_after=hashlib.sha256(p.read_bytes()).hexdigest()))
 (receipt/'negative-controls.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
# Final positive checks run only after every mutation is restored.
steps=[
 ('focused',['cargo','test','-p','seiche','--lib','coupling','--locked','--offline','-j','2','--','--test-threads=4','--nocapture']),
 ('host-bridge',['cargo','test','-p','pictograph','--features','canvas','--lib','coupling','--locked','--offline','-j','2','--','--test-threads=4','--nocapture']),
 ('clippy',['cargo','clippy','-p','seiche','--all-targets','--locked','--offline','-j','2'])]
for name,cmd in steps:
 checkpoint=subprocess.check_output(['git','rev-parse','HEAD']).decode().strip();print('Starting restored '+name,flush=True);start=time.monotonic()
 with (receipt/(name+'.log')).open('wb') as log:
  result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append(dict(name=name,command=cmd,checkpoint=checkpoint,exit_code=result.returncode,elapsed_seconds=round(time.monotonic()-start,3)))
 (receipt/'final-gates.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
 print(name+': '+str(result.returncode)+'\n'+'\n'.join((receipt/(name+'.log')).read_text(encoding='utf-8').splitlines()[-4:]),flush=True)
 assert result.returncode==0,name
