#!/usr/bin/env python3
"""Headed local authentication through Dramatis and an owned synthetic Djinn authority.
The ignored-test host refuses existing seed roots and requires its fixture marker on reopen.
"""
import argparse, getpass, hashlib, json, os, platform, subprocess, time
from pathlib import Path

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def wait_file(path, process, timeout=180):
    end=time.monotonic()+timeout
    while not path.exists():
        assert process.poll() is None, ('fixture exited', process.returncode)
        assert time.monotonic()<end, ('fixture readiness',str(path))
        time.sleep(.05)
def desktop_environment():
    """Use the existing Linux user session; never configure or unlock it."""
    assert platform.system()=='Linux'
    raw=subprocess.check_output(['systemctl','--user','show-environment'],text=True)
    allowed={'DISPLAY','WAYLAND_DISPLAY','XDG_RUNTIME_DIR','XDG_SESSION_TYPE',
             'XDG_CURRENT_DESKTOP','DBUS_SESSION_BUS_ADDRESS','XAUTHORITY'}
    graphical={k:v for line in raw.splitlines() if '=' in line
               for k,v in [line.split('=',1)] if k in allowed}
    assert graphical.get('DISPLAY') or graphical.get('WAYLAND_DISPLAY'),'no graphical session'
    session=subprocess.check_output(['loginctl','show-user',getpass.getuser(),'-p','Display','--value'],text=True).strip()
    raw=subprocess.check_output(['loginctl','show-session',session,'-p','Name','-p','Active',
        '-p','State','-p','Type','-p','Remote','-p','LockedHint'],text=True)
    state=dict(line.split('=',1) for line in raw.splitlines() if '=' in line)
    assert state.get('Active')=='yes' and state.get('LockedHint')=='no',state
    return graphical,{'session':session,'state':state,'environment':graphical,
        'hostname':platform.node(),'architecture':platform.machine(),'kernel':platform.release(),
        'os_release':Path('/etc/os-release').read_text()}
def main():
    p=argparse.ArgumentParser()
    for n in ['binary','host','runtime','output']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--native',action='store_true')
    p.add_argument('--desktop',action='store_true',help='use the existing unlocked Linux user desktop')
    p.add_argument('--format',choices=['sealed','portable','both'],default='both')
    a=p.parse_args();a.runtime.mkdir(mode=0o700,parents=True);a.output.mkdir(parents=True)
    repo=Path(__file__).resolve().parents[1];processes=[];results=[];graphical={}
    if a.desktop:
        graphical,desktop=desktop_environment()
        (a.output/'desktop.json').write_text(json.dumps(desktop,indent=2)+'\n')
    for fmt in (['sealed','portable'] if a.format=='both' else [a.format]):
        root=a.runtime/fmt;out=(a.output/fmt).resolve();out.mkdir()
        for phase in ['seed','reopen']:
            host_log=(out/(phase+'-host.log')).open('w')
            env=os.environ.copy()
            for key in list(env):
                if key.startswith(('DRAMATIS_','GRAPHSHELL_','PERSONAE_','MERE_','D_AUTH_')):env.pop(key)
            env.update(graphical)
            env.update(D_AUTH_ROOT=str(root.resolve()),D_AUTH_FORMAT=fmt,D_AUTH_PHASE=phase,D_AUTH_NATIVE='1' if a.native else '0')
            host=subprocess.Popen([str(a.host.resolve()),'--ignored','--exact','headed_authentication_host','--nocapture'],cwd=repo,env=env,stdout=host_log,stderr=subprocess.STDOUT)
            try:
                wait_file(root/'ready',host)
                endpoint=str((root/'app.sock').resolve())
                def await_(fields):return 'await '+json.dumps(fields)+'\nwait 300000\n'
                def capture(name):return 'capture '+str(out/(name+'.png'))+'\n'
                script=await_({'ready':'true','profiles':'2','locked':'false'})
                if phase=='seed':
                    script+=capture('before')
                    cases=[] if a.native else [('cancelled','cancelled'),('wrong-current','current passphrase was not accepted'),('mismatch','did not match'),('empty','cannot be empty')]
                    for name,notice in cases:
                        script+='click role:button Change passphrase…\n'+await_({'notice':'~'+notice})+capture(name)
                    script+='click role:button Change passphrase…\n'+await_({'notice':'~Passphrase changed.'})+capture('changed')
                    script+='settle 240\n'+capture('acknowledged-after-refresh')
                else:script+=capture('cold-reopened')
                scn=out/(phase+'.scn');scn.write_text(script)
                shown=env|{'GRAPHSHELL_APP_ENDPOINT':endpoint,'DRAMATIS_SCENARIO':str(scn),'DRAMATIS_RECEIPT':str(out/(phase+'.json'))}
                with (out/(phase+'-app.log')).open('w') as log:
                    app=subprocess.Popen([str(a.binary.resolve())],cwd=repo,env=shown,stdout=log,stderr=subprocess.STDOUT)
                    try:app.wait(timeout=600)
                    finally:
                        if app.poll() is None:app.terminate();app.wait(timeout=15)
                assert app.returncode==0,('native app failed',fmt,phase,app.returncode)
                receipt=json.loads((out/(phase+'.json')).read_text());assert receipt['ok'] and not receipt['errors'],receipt
                assert int(receipt['final']['pid'])==app.pid
                processes.append({'kind':'dramatis','pid':app.pid,'format':fmt,'phase':phase,'exit':app.returncode})
            finally:
                if root.exists():(root/'stop').touch()
                try:host.wait(timeout=20)
                except subprocess.TimeoutExpired:host.terminate();host.wait(timeout=10)
                host_log.close()
                processes.append({'kind':'djinn-fixture','pid':host.pid,'format':fmt,'phase':phase,'exit':host.returncode})
            assert host.returncode==0
            for name in ['before-identities.json','cold-identities.json','old-rejected','native-provider.json']:
                if (root/name).exists():(out/name).write_bytes((root/name).read_bytes())
            for name in ['ready','stop']:(root/name).unlink()
        before=json.loads((out/'before-identities.json').read_text());cold=json.loads((out/'cold-identities.json').read_text())
        assert before==cold and (out/'old-rejected').read_text()=='true'
        results.append({'format':fmt,'identities_preserved':True,'old_rejected_after_cold_reopen':True})
    result={'result':'ok','source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
      'native_dialogs':a.native,'public_synthetic_vaults_only':True,'formats':results,'processes':processes,
      'binaries':{'app_sha256':sha(a.binary),'host_sha256':sha(a.host)},'controller_sha256':sha(Path(__file__))}
    (a.output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
if __name__=='__main__':main()
