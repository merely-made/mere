from pathlib import Path
import hashlib,json,os,platform,plistlib,subprocess,sys,time,traceback
r=Path('/Users/markik/Code/worktrees/mere-authentication-m4');o=Path('/Users/markik/Code/output/dauth-m4-20261022');mode=sys.argv[1];assert mode in ['scripted','native','native-cold'];assert(o/'.dramatis-generated').read_text()=='synthetic authentication qualification'
try:
 assert platform.machine()=='arm64';assert not any(k in os.environ for k in ['SSH_TTY','SSH_CLIENT','SSH_CONNECTION']),'SSH environment entered the GUI job'
 facts=json.loads(subprocess.check_output([sys.executable,str(o/'machine.py')],text=True));assert facts['console_present'] and not facts['console_locked'] and facts['console_user']=='markik' and facts['launchctl_gui_available'],facts
 facts['bootstrap_manager']=subprocess.check_output(['launchctl','managername'],text=True).strip();assert facts['bootstrap_manager']=='Aqua',facts
 f=o/'frozen';b=json.loads((f/'build-inputs.json').read_text());source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip();assert source==b['source_commit'];assert not subprocess.check_output(['git','status','--porcelain'],cwd=r,text=True).strip()
 for name,h in b['files'].items():assert hashlib.sha256((r/name).read_bytes()).hexdigest()==h,name
 for name,v in b['binaries'].items():assert hashlib.sha256((f/name).read_bytes()).hexdigest()==v['sha256'],name
 facts.update({'mode':mode,'source':source,'pid':os.getpid(),'controller_sha256':hashlib.sha256((r/'support/dramatis_auth_receipts.py').read_bytes()).hexdigest(),'native_prompt_pixels_captured':False,'renderer':'native host default; no GPU backend claim','environment_has_ssh_variables':False,'sleep_inhibitor':'caffeinate -di, bound to this GUI runner lifetime'})
 (o/(mode+'-preflight.json')).write_text(json.dumps(facts,indent=2)+'\n')
 command=[sys.executable,str(r/'support/dramatis_auth_receipts.py'),'--binary',str(f/'Dramatis.app/Contents/MacOS/dramatis-ui'),'--host',str(f/'authentication-host'),'--runtime',str(o/('s' if mode=='scripted' else 'cold' if mode=='native-cold' else 'n')),'--output',str(o/mode)]
 if mode=='native':command+=['--native','--native-cancel']
 if mode=='native-cold':command+=['--native','--reopen-only']
 with (o/(mode+'-launch.log')).open('w',buffering=1) as log:
  inhibitor=subprocess.Popen(['/usr/bin/caffeinate','-di','-w',str(os.getpid())])
  try:result=subprocess.run(command,cwd=r,stdout=log,stderr=subprocess.STDOUT)
  finally:
   inhibitor.terminate();inhibitor.wait(timeout=5)
 (o/(mode+'-task-result.json')).write_text(json.dumps({'exit':result.returncode,'source':source,'command':command},indent=2)+'\n');sys.exit(result.returncode)
except Exception:
 traceback.print_exc();(o/(mode+'-task-result.json')).write_text(json.dumps({'exit':1,'error':'See '+mode+'-agent.log'})+'\n');raise
