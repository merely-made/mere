# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import subprocess,hashlib,os,json,time
raw=Path('C:/Users/mark_/Code/testing/mere/field-runtime');env=dict(os.environ,CARGO_TARGET_DIR='C:/t/cargo-targets/mere')
controls=[
 ('negative-zero-clock','crates/conatus/seiche/src/coupling_force.rs','        use CouplingEffect::{AddForce, Displace, Inert, ScaleVelocity, SetVelocity};','        let sample_time = 0.0;\n        use CouplingEffect::{AddForce, Displace, Inert, ScaleVelocity, SetVelocity};','field_runtime_clock_controls_real_motion_and_reset_does_not_rewind_solver_age'),
 ('negative-exclusive-boundary','crates/conatus/numen/src/field.rs','x >= *min_x && x <= *max_x','x > *min_x && x <= *max_x','field_runtime_inclusive_reference_points_filter_actual_writes'),
 ('negative-initial-entry','crates/conatus/seiche/src/coupling_membership.rs','} else if change == Entered {','} else if change == Entered || (change == Initial && eligibility == CouplingEligibility::Inside) {','field_runtime_repeated_appearances_are_independent_and_initial_matches_do_not_act')]
results=[]
for name,fn,before,after,test in controls:
 p=Path(fn);original=p.read_bytes();digest=hashlib.sha256(original).hexdigest();s=original.decode();assert s.count(before)==1
 cmd=['cargo','test','-p','seiche','--no-default-features','--lib',test,'--locked','--offline','-j','2','--','--nocapture']
 print('Starting '+name,flush=True)
 try:
  p.write_bytes(s.replace(before,after).encode())
  with (raw/(name+'.log')).open('wb') as log:result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
  output=(raw/(name+'.log')).read_text(encoding='utf-8');assert result.returncode != 0 and 'FAILED' in output and test in output,(name,result.returncode)
  print(name+': expected failure',flush=True)
 finally:
  p.write_bytes(original);assert hashlib.sha256(p.read_bytes()).hexdigest()==digest
 results.append({'name':name,'source':fn,'command':cmd,'expected_test':test,'exit_code':result.returncode,'restored':True,'sha256_before':digest,'sha256_after':hashlib.sha256(p.read_bytes()).hexdigest()})
 (raw/'negative-controls.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
steps=[
 ('focused',['cargo','test','-p','seiche','--lib','field_runtime','--locked','--offline','-j','2','--','--test-threads=4','--nocapture']),
 ('trajectory',['cargo','test','-p','seiche','--lib','execution_trajectory_receipt','--locked','--offline','-j','2','--','--nocapture']),
 ('host-field-runtime',['cargo','test','-p','pictograph','--features','canvas','--lib','field_runtime','--locked','--offline','-j','2','--','--test-threads=4','--nocapture'])]
for name,cmd in steps:
 print('Starting restored '+name,flush=True);start=time.monotonic()
 with (raw/(name+'.log')).open('wb') as log:result=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append({'name':name,'command':cmd,'exit_code':result.returncode,'elapsed_seconds':round(time.monotonic()-start,3)})
 (raw/'final-gates.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
 print(name+': '+str(result.returncode)+'\n'+'\n'.join((raw/(name+'.log')).read_text(encoding='utf-8').splitlines()[-4:]),flush=True)
 if result.returncode:raise SystemExit(result.returncode)
