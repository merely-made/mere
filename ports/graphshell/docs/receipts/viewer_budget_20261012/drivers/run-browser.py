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
subprocess.run([node,'-e',"require('./assert-console-unlocked.cjs')()"],cwd=root,check=True)
state={'source':'cef122f25737a82c4621de49771c90fc357be348','cutoff':sys.argv[1],'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'drivers':[],'closed':False}
processes=[]
def save(): (root/'browser-lease-r1.json').write_text(json.dumps(state,indent=2)+'\n')
try:
 with (root/'http-r1.log').open('w') as http_log:
  server=subprocess.Popen([sys.executable,'-m','http.server','8847','--bind','127.0.0.1','--directory',str(root)],stdout=http_log,stderr=subprocess.STDOUT,start_new_session=True);processes.append(server)
  inhibitor=subprocess.Popen(['/usr/bin/caffeinate','-di','-t',str(max(1,int(cutoff-time.time())))],start_new_session=True);processes.append(inhibitor)
  state['owned_pids']=[p.pid for p in processes];save();time.sleep(1);assert server.poll() is None,'HTTP server failed to bind'
  for driver in ['verify-commands.cjs','verify-presenter.cjs','verify-budget.cjs']:
   assert time.time()<cutoff-10,'Lease cutoff reached'
   print('START',driver,flush=True)
   with (root/(driver+'.r1.log')).open('w') as log:
    run=subprocess.Popen([node,driver],cwd=root,stdout=log,stderr=subprocess.STDOUT,start_new_session=True);processes.append(run);state['owned_pids'].append(run.pid);save()
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
