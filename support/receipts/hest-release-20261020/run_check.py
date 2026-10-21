# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

import datetime,json,os,pathlib,subprocess,sys,time
base=pathlib.Path(__file__).parent
name,cwd,*command=sys.argv[1:]
record=base/(name+'.json')
if record.exists():raise SystemExit('refusing to overwrite '+str(record))
start=time.monotonic()
revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=cwd,text=True).strip()
with (base/(name+'.log')).open('wb') as log:
 p=subprocess.run(command,cwd=cwd,stdout=log,stderr=subprocess.STDOUT)
data={'name':name,'cwd':cwd,'command':command,'source_revision':revision,'exit_code':p.returncode,'elapsed_seconds':round(time.monotonic()-start,2),'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'log':name+'.log'}
record.write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(data))
print((base/(name+'.log')).read_text(errors='replace')[-3500:])
raise SystemExit(p.returncode)
