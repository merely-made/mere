from pathlib import Path
import gzip,hashlib,json,os,re,struct,subprocess,zipfile
r=Path('/Users/markik/Code/worktrees/mere-authentication-m4');p=Path('/Users/markik/Code/output/dauth-m4-20261022');assert(p/'.dramatis-generated').read_text()=='synthetic authentication qualification'
def read(q):return json.loads(q.read_text())
def sha(q):return hashlib.sha256(q.read_bytes()).hexdigest()
b=read(p/'frozen/build-inputs.json');assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()==b['source_commit'];assert not subprocess.check_output(['git','status','--porcelain'],cwd=r,text=True).strip()
for n,h in b['files'].items():assert sha(r/n)==h,n
for n,h in b['generated_build_inputs']['files'].items():label,rel=n.split('/',1);assert sha(Path(b['generated_build_inputs']['roots'][label])/rel)==h,n
for n,v in b['binaries'].items():assert sha(p/'frozen'/n)==v['sha256'],n
original=read(p/'native-input-evidence.json');transfer={}
for n,info in original['native_changed_encrypted_files'].items():assert sha(p/'qualified-n'/n)==info['sha256']==sha(p/'cold'/n),n;transfer[n]=info
live=[line for line in subprocess.check_output(['ps','-axo','pid=,comm='],text=True).splitlines() if str(p/'frozen') in line];assert not live,live
selected={};captures=[];processes=[];delays=[];jobs=[]
for mode in ['scripted','native-cold']:
 q=read(p/mode/'qualification.json');assert q['result']=='ok' and q['source']==b['source_commit'];assert read(p/(mode+'-task-result.json'))['exit']==0
 assert q['binaries']['app_sha256']==b['binaries']['Dramatis.app/Contents/MacOS/dramatis-ui']['sha256'] and q['binaries']['host_sha256']==b['binaries']['authentication-host']['sha256'];assert q['controller_sha256']==b['files']['support/dramatis_auth_receipts.py']
 if mode=='native-cold':assert q['reopen_only'] and not q['native_dialogs'] and q['native_provider_selected']
 job=subprocess.check_output(['launchctl','print','gui/'+str(os.getuid())+'/org.merely.dramatis.authentication.m4.20261022.'+mode],text=True);assert 'last exit code = 0' in job and not re.search(r'^\s*pid = ',job,re.M);jobs.append({'mode':mode,'last_exit':0,'running':False})
 for row in q['processes']:assert row['exit']==0;processes.append(row|{'mode':mode})
 for fmt in ['sealed','portable']:
  d=p/mode/fmt;assert read(d/'before-identities.json')==read(d/'cold-identities.json') and(d/'old-rejected').read_text()=='true'
  records=[read(d/'reopen.json')]+([read(d/'seed.json')] if mode=='scripted' else[])
  for record in records:
   assert record['ok'] and not record['errors']
   for row in record['captures']:
    f=Path(row['path']);assert f.parent==d;raw=f.read_bytes();assert raw[:8]==b'\x89PNG\r\n\x1a\n';w,h=struct.unpack('>II',raw[16:24]);assert(w,h)==(row['width'],row['height']);captures.append({'path':f.relative_to(p).as_posix(),'sha256':sha(f),'width':w,'height':h,'public_fields':row['fields'],'visual_review':'Pending current contact sheet review'})
  if mode=='scripted':
   gap=((d/'acknowledged-after-refresh.png').stat().st_mtime_ns-(d/'changed.png').stat().st_mtime_ns)/1e9;assert gap>=3;delays.append({'format':fmt,'seconds':gap})
  else:assert read(d/'native-provider.json')=={'graphic':True,'name':'applescript'}
  for f in d.iterdir():assert f.suffix in ['.png','.log','.json','.scn'] or f.name=='old-rejected';selected[f.relative_to(p).as_posix()]=f
 selected[mode+'/qualification.json']=p/mode/'qualification.json'
 for n in [mode+'-preflight.json',mode+'-task-result.json',mode+'-launch.log',mode+'-agent.log']:selected[n]=p/n
assert len(captures)==18 and len(processes)==12
g=read(p/'software-gates.json');assert sum(x['passed'] for x in g['commands'] if x['name']!='tool-tests')==17 and sum(x['passed'] for x in g['reused_component_gates']['commands'])==30 and all(x['exit']==0 for x in g['commands'])
proof={'source':b['source_commit'],'capture_files':captures,'processes':processes,'gui_jobs':jobs,'acknowledgement_delays':delays,'no_owned_app_or_host_processes':True,'source_inputs_verified':len(b['files']),'generated_inputs_verified':len(b['generated_build_inputs']['files']),'binaries_reverified':len(b['binaries']),'native_changed_encrypted_files_reverified':transfer,'native_dialog_pixels_captured':False,'scope':'Current scripted headed acceptance and cold reopens of exact prior native-changed vault bytes; these runs collect no new native input.'};(p/'integration-post-run.json').write_text(json.dumps(proof,indent=2)+'\n');selected['post-run.json']=p/'integration-post-run.json'
for n in ['build-inputs.json','dramatis-app-tree.txt','djinn-tree.txt']:selected[n]=p/'frozen'/n
for n in ['software-gates.json','interactive.py','start.py','machine.py','integration-resume.py','current-gates.py','collect-integration.py','native-isolation.json']:selected[n]=p/n
for row in g['commands']:selected[row['name']+'.log']=p/(row['name']+'.log')
for n in ['software-gates.json','storage-tests.log','provider-tests.log']:selected['component-gates/'+n]=p/'component-gates'/n
total=0
with zipfile.ZipFile(p/'integration-public.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,f in sorted(selected.items()):
  raw=f.read_bytes()
  if f.suffix=='.log':n+='.gz';raw=gzip.compress(raw,mtime=0)
  total+=len(raw);z.writestr(n,raw)
assert total<100_000_000;print(json.dumps({'source':b['source_commit'],'captures':len(captures),'processes':len(processes),'retained_bytes':total,'all_passed':True}))
