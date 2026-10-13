# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import os, subprocess, json, time
receipt=Path('C:/Users/mark_/Code/testing/mere/coupling-trace')
steps=[
 ('default',['cargo','test','-p','seiche','--locked','--offline','-j','2','--','--test-threads=4']),
 ('inline',['cargo','test','-p','seiche','--no-default-features','--locked','--offline','-j','2','--','--test-threads=4']),
 ('serde',['cargo','test','-p','seiche','--features','serde','--locked','--offline','-j','2','--','--test-threads=4']),
 ('wasm',['cargo','check','-p','seiche','--lib','--no-default-features','--target','wasm32-unknown-unknown','--locked','--offline','-j','2']),
 ('example',['cargo','run','-p','seiche','--no-default-features','--example','coupling_trace','--locked','--offline','-j','2']),
 ('host-bridge',['cargo','test','-p','pictograph','--features','canvas','--lib','coupling','--locked','--offline','-j','2','--','--test-threads=4','--nocapture']),
 ('clippy',['cargo','clippy','-p','seiche','--all-targets','--locked','--offline','-j','2'])]
env=dict(os.environ,CARGO_TARGET_DIR='C:/t/cargo-targets/mere')
results=[]
for name,cmd in steps:
 print('Starting '+name,flush=True);start=time.monotonic()
 with (receipt/(name+'.log')).open('wb') as log:
  result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append(dict(name=name,command=cmd,exit_code=result.returncode,elapsed_seconds=round(time.monotonic()-start,3)))
 (receipt/'gates.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
 lines=(receipt/(name+'.log')).read_text(encoding='utf-8',errors='replace').splitlines()
 print(name+': '+str(result.returncode)+'\n'+'\n'.join(lines[-5:]),flush=True)
 if result.returncode: raise SystemExit(result.returncode)
