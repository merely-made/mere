import getpass,json,os,platform,plistlib,subprocess
from pathlib import Path
r=plistlib.loads(subprocess.check_output(['/usr/sbin/ioreg','-n','Root','-d1','-a']))
r=r[0] if isinstance(r,list) else r
u=next((u for u in r.get('IOConsoleUsers',[]) if u.get('kCGSSessionOnConsoleKey')), {})
v={'hostname':platform.node(),'architecture':platform.machine(),'macos':subprocess.check_output(['sw_vers','-productVersion'],text=True).strip(),'console_user':u.get('kCGSSessionUserNameKey'),'console_present':bool(u),'console_locked':bool(r.get('IOConsoleLocked',False) or u.get('CGSSessionScreenIsLocked',False)),'uid':os.getuid(),'controller_user':getpass.getuser(),'launchctl_gui_available':subprocess.run(['launchctl','print','gui/'+str(os.getuid())],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0}
v.update({'chip':subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip(),'hardware_model':subprocess.check_output(['sysctl','-n','hw.model'],text=True).strip(),'memory_bytes':int(subprocess.check_output(['sysctl','-n','hw.memsize'],text=True).strip()),'python_version':platform.python_version()})
print(json.dumps(v))
