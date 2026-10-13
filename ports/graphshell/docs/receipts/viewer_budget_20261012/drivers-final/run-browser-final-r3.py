#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
import datetime,json,os,signal,subprocess,sys,time
from pathlib import Path
root=Path(__file__).resolve().parent
cutoff=datetime.datetime.fromisoformat(sys.argv[1].replace('Z','+00:00')).timestamp()
node='/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node'
assert cutoff>time.time()+120,'Fresh finite runtime window required'
candidate=json.loads((root/'browser-candidate-final-r3.json').read_text())
for name,digest in candidate['bundle_sha256'].items():
 assert __import__('hashlib').sha256((root/'web-final/pkg'/name).read_bytes()).hexdigest()==digest,'Frozen package changed: '+name
for name,digest in candidate['served_input_sha256'].items():
 assert __import__('hashlib').sha256((root/'web-final'/name).read_bytes()).hexdigest()==digest,'Frozen served input changed: '+name
subprocess.run([node,'-e',"require('./assert-console-unlocked.cjs')()"],cwd=root,check=True)
state={'source':json.loads((root/'browser-candidate-final-r3.json').read_text())['source'],'cutoff':sys.argv[1],'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'drivers':[],'closed':False}
processes=[]
def save(): (root/'browser-lease-final-r3.json').write_text(json.dumps(state,indent=2)+'\n')
try:
 with (root/'http-final-r3.log').open('w') as http_log:
  server=subprocess.Popen([sys.executable,'-m','http.server','8847','--bind','127.0.0.1','--directory',str(root)],stdout=http_log,stderr=subprocess.STDOUT,start_new_session=True);processes.append(server)
  inhibitor=subprocess.Popen(['/usr/bin/caffeinate','-di','-t',str(max(1,int(cutoff-time.time())))],start_new_session=True);processes.append(inhibitor)
  state['owned_pids']=[p.pid for p in processes];save();time.sleep(1);assert server.poll() is None,'HTTP server failed to bind'
  for driver in ['verify-presenter-final-sync.cjs','verify-budget-final.cjs']:
   assert time.time()<cutoff-10,'Lease cutoff reached'
   print('START',driver,flush=True)
   with (root/(driver+'.r3.log')).open('w') as log:
    run=subprocess.Popen([node,driver],cwd=root,env=dict(os.environ,RECEIPT_ROUND='r3'),stdout=log,stderr=subprocess.STDOUT,start_new_session=True);processes.append(run);state['owned_pids'].append(run.pid);save()
    code=run.wait(timeout=max(1,cutoff-time.time()-5));state['drivers'].append({'driver':driver,'exit':code});save();print('EXIT',driver,code,flush=True)
    assert code==0,driver+' failed; preserve run and release'
except BaseException as e:
 state['failure']=str(e);save();raise
finally:
 for p in reversed(processes):
  if p.poll() is None:
   try: os.killpg(p.pid,signal.SIGTERM);p.wait(timeout=5)
   except (ProcessLookupError,subprocess.TimeoutExpired):
    try: os.killpg(p.pid,signal.SIGKILL)
    except ProcessLookupError: pass
 state['closed']=True;state['released']=datetime.datetime.now(datetime.timezone.utc).isoformat();save();print('CLOSED owned runtime',flush=True)
