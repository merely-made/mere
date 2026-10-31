import argparse,hashlib,json,pathlib,subprocess,time
p=argparse.ArgumentParser()
p.add_argument('app',choices=['woodshed','knot-editor'])
p.add_argument('attempt')
o=p.parse_args()
b=pathlib.Path('/Users/markik/Code');root=b/'output/context-navigation-chrome';repo=b/'repos'/o.app
output=root/o.attempt
if output.exists():raise RuntimeError('new output required')
binary=b/'targets/lanes/context-navigation-chrome/artifacts/debug'/('woodshed-genet' if o.app=='woodshed' else 'knot')
if not binary.is_file():raise RuntimeError('missing binary: '+str(binary))
runtime=repo/'.cargo/local/context-navigation-chrome'/o.attempt
runtime.mkdir(parents=True,exist_ok=False)
scenario=root/(o.app+'-chrome.scn')
launcher=root/'run_native_direct.py'
env={'WOODSHED_DATA_DIR':str(runtime),'WOODSHED_PRESENTATION_TIMEOUT_SECS':'60'} if o.app=='woodshed' else {'APPDATA':str(runtime),'KNOT_THEME_LIBRARY':str(runtime/'themes.json')}
cmd=['/usr/bin/python3',str(launcher),'--binary',str(binary),'--prefix','WOODSHED' if o.app=='woodshed' else 'KNOT','--scenario',str(scenario),'--output',str(output),'--timeout','600']
for key,value in env.items():cmd+=['--env',key+'='+value]
fixture=repo/'scenarios/fixtures/context_navigation.djot'
fixture_before=None
if o.app=='knot-editor':
 fixture_before=hashlib.sha256(fixture.read_bytes()).hexdigest();cmd+=['--',str(fixture)]
result={'app':o.app,'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'scenario_sha256':hashlib.sha256(scenario.read_bytes()).hexdigest(),'captures':[],'native_success':False,'error':None,'runtime':str(runtime)}
with (root/(o.attempt+'-launch.log')).open('x') as log:
 child=subprocess.Popen(cmd,stdout=log,stderr=subprocess.STDOUT)
 try:
  for size in ['wide','narrow']:
   target=output/(size+'.png');deadline=time.monotonic()+180
   while not target.exists():
    if child.poll() is not None:raise RuntimeError('app ended before '+size)
    if time.monotonic()>deadline:raise RuntimeError('timed out awaiting '+size)
    time.sleep(1)
   time.sleep(1)
   found=subprocess.run(['/usr/sbin/lsof','-t',str(output/'native.log')],capture_output=True,text=True,timeout=20)
   pids=found.stdout.split()
   if not pids:raise RuntimeError('no product log owner')
   info=subprocess.check_output([str(b/'output/turnstone-gui-native-20261020/window_info'),*pids],text=True,timeout=20)
   (output/(size+'-windows.json')).write_text(info)
   windows=json.loads(info)['windows']
   if not windows:raise RuntimeError('no native product window')
   window=max(windows,key=lambda w:w.get('kCGWindowBounds',{}).get('Width',0)*w.get('kCGWindowBounds',{}).get('Height',0))
   capture=output/(size+'-native-window.png')
   shot=subprocess.run(['/usr/sbin/screencapture','-x','-o','-l',str(window['kCGWindowNumber']),str(capture)],capture_output=True,text=True,timeout=30)
   observation={'checkpoint':size,'window_id':window['kCGWindowNumber'],'native_bounds':window.get('kCGWindowBounds'),'exit':shot.returncode,'stderr':shot.stderr,'captured':capture.exists()}
   result['captures'].append(observation)
   (output/(size+'-os-capture.json')).write_text(json.dumps(observation,indent=2)+'\n')
   print(json.dumps(observation),flush=True)
   if shot.returncode or not capture.exists():raise RuntimeError('OS capture failed: '+shot.stderr.strip())
  result['native_success']=True
 except Exception as error:
  result['error']=str(error);print(str(error),flush=True)
 finally:
  try:child.wait(timeout=180)
  except subprocess.TimeoutExpired:
   result['error']=(result['error'] or '')+' launcher remained active; own runner timeout will clean child'
  result['launcher_exit']=child.poll()
  if fixture_before is not None:result['fixture_unchanged']=fixture_before==hashlib.sha256(fixture.read_bytes()).hexdigest()
  (root/(o.attempt+'-result.json')).write_text(json.dumps(result,indent=2)+'\n')
  print(json.dumps(result),flush=True)
raise SystemExit(0 if result['native_success'] and result['launcher_exit']==0 else 1)
