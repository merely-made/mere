from pathlib import Path
import json,os,plistlib,subprocess,sys
p=Path('/Users/markik/Code/output/dauth-m4-20261022');assert(p/'.dramatis-generated').read_text()=='synthetic authentication qualification';mode=sys.argv[1];assert mode in ['scripted','native','native-cold'];assert(p/'current-step').read_text()=='software-complete';assert not(p/mode).exists()
if mode=='native':assert json.loads((p/'scripted-task-result.json').read_text())['exit']==0
label='org.merely.dramatis.authentication.m4.20261022.'+mode
plist=p/(mode+'.plist');assert not plist.exists()
value={'Label':label,'ProgramArguments':['/usr/bin/python3',str(p/'interactive.py'),mode],'WorkingDirectory':'/Users/markik/Code/worktrees/mere-authentication-m4','RunAtLoad':True,'ProcessType':'Interactive','StandardOutPath':str(p/(mode+'-agent.log')),'StandardErrorPath':str(p/(mode+'-agent.log')),'EnvironmentVariables':{'PATH':'/usr/bin:/bin:/usr/sbin:/sbin:/Users/markik/.cargo/bin'}}
plist.write_bytes(plistlib.dumps(value));subprocess.run(['launchctl','bootstrap','gui/'+str(os.getuid()),str(plist)],check=True);print(json.dumps({'started':label,'mode':mode,'plist':str(plist)}))
