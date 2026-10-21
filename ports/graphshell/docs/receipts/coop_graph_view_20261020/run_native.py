#!/usr/bin/env python3
"""Existing resident graphs, the ordinary Graphshell view, two custody roots."""
import argparse,hashlib,json,os,plistlib,shutil,signal,subprocess,sys,time,uuid
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'coop_20261020'))
from resident import Resident


def write(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value,indent=2)+'\n')
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def until(path,processes=(),seconds=180):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        if path.exists():return
        for process in processes:
            assert process.poll() is None,f'{process.pid} exited {process.returncode} before {path}'
        time.sleep(.1)
    raise TimeoutError(path)
def await_(**fields):return 'await '+json.dumps(fields)+'\nwait 30000\n'
def click(id):return 'click role:button @id='+id+'\n'
def capture(path):return 'wheel 0 -10000 px on .coop-panel\nsettle 2\ncapture '+str(path)+'\n'
def input_(i,text):return f'click role:textbox @id=coop-field-{i}\ninput-text {text}\n'


def main():
    parser=argparse.ArgumentParser()
    for key in ['binary','djinn','prepare','output']:parser.add_argument('--'+key,required=True,type=Path)
    args=parser.parse_args();out=args.output.resolve();assert not out.exists();out.mkdir(mode=0o700)
    public=out/'evidence';public.mkdir();private=out/'private';private.mkdir()
    owner=Resident(args.djinn,private/'owner','GraphViewOwner',public/'owner-resident')
    guest=Resident(args.djinn,private/'guest','GraphViewGuest',public/'guest-resident')
    # Select a public reservoir identity explicitly in this otherwise empty
    # test root. Signing keys still come from the real passphrase-held vault.
    owner_settings={'reservoir':{'enabled':True,'persona':str(uuid.uuid4())}}
    write(Path(owner.environment['XDG_DATA_HOME'])/'graphshell/settings/GraphViewOwner.json',owner_settings)
    write(public/'owner-settings.json',owner_settings)
    processes=[];apps={};active={};domain='coop-acceptance';export=private/'invitation';guest_contact=private/'guest-contact.json'
    def command(resident,binary,*cmd):
        result=subprocess.run([str(binary.resolve()),*map(str,cmd)],env=resident.environment,capture_output=True,text=True,timeout=90)
        if result.returncode:
            print(result.stderr,file=sys.stderr,flush=True)
        result.check_returncode()
        return result.stdout
    def prepare(*cmd):return command(owner,args.prepare,*cmd)
    def launch(name,resident,script,visit=False):
        scenario=public/(name+'.scn');scenario.write_text(script)
        env=resident.environment|{'GRAPHSHELL_MERE_SCENARIO':str(scenario),'GRAPHSHELL_MERE_RECEIPT':str(public/(name+'.json')),'GRAPHSHELL_MERE_JSON_RECEIPT':'1','GRAPHSHELL_MERE_PRESENTATION_TIMEOUT_SECS':'60','RUST_LOG':'warn'}
        role='guest' if visit else 'owner';app=private/(role+'.app');exe=app/'Contents/MacOS/graphshell_mere'
        if not exe.exists():
            exe.parent.mkdir(parents=True);shutil.copy2(args.binary,exe)
            with (app/'Contents/Info.plist').open('wb') as f:plistlib.dump({'CFBundleIdentifier':'made.merely.graphshell.view.coop.'+uuid.uuid4().hex,'CFBundleName':'Graphshell '+role,'CFBundleExecutable':'graphshell_mere','CFBundlePackageType':'APPL','CFBundleVersion':'1','NSHighResolutionCapable':True},f)
        assert sha(exe)==sha(args.binary)
        apps[role]=app
        active[role]=name
        cmd=['/usr/bin/open','-n','-W','--stdout',str(public/(name+'.log')),'--stderr',str(public/(name+'.log'))]
        for key,value in env.items():
            if key.startswith(('XDG_','MERE_','GRAPHSHELL_','TURNSTONE_')) or key in {'LOCALAPPDATA','TMPDIR','RUST_LOG'}:
                cmd+=['--env',key+'='+value]
        cmd+=['-a',str(app),'--args']
        cmd+=['--invitation',str(export/'invitation.json'),'--contact',str(export/'contact.json')] if visit else [domain]
        p=subprocess.Popen(cmd,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);processes.append(p)
        write(public/(name+'-launch.json'),{'launcher_pid':p.pid,'role':role,'binary_sha256':sha(exe),'scenario':scenario.name})
        return p
    def activate(role):
        # A completed phase may already have closed its window. Never relaunch
        # it without the next phase's explicit arguments and isolated environment.
        if (public/(active[role]+'.json')).exists():return
        executable=str(apps[role]/'Contents/MacOS/graphshell_mere')
        commands=subprocess.check_output(['ps','-axo','command='],text=True).splitlines()
        if not any(command.startswith(executable) for command in commands):return
        subprocess.run(['/usr/bin/open','-a',str(apps[role])],capture_output=True,check=True,timeout=10)
    def finish(name,p):
        until(public/(name+'.json'),(p,));p.wait(timeout=25)
        receipt=json.loads((public/(name+'.json')).read_text());assert receipt['ok'],receipt
        assert receipt['captures'] and all(c['width']>200 and c['height']>200 for c in receipt['captures']);return receipt
    try:
        owner.start();guest.start()
        ids=json.loads(prepare('seed',domain));write(public/'existing-graphs.json',ids)
        chosen,private_graph=ids['chosen'],ids['private']
        write(public/'private-before.json',json.loads(prepare('read',domain,private_graph)))
        write(public/'chosen-before.json',json.loads(prepare('read',domain,chosen)))
        # Read the stable surface identity from the ordinary snapshot's sources.
        source=next(s for s in json.loads((public/'chosen-before.json').read_text())['snapshot']['scene']['tables']['sources'] if s and s['adapter']=='mere.graph')
        existing=source['id']
        command(guest,args.binary,'--contact',guest_contact)
        script=await_(status='ready',attached=chosen)+'click role:button @data-session-step=switch:'+private_graph+'\n'+await_(attached=private_graph)+capture(public/'private-before.png')+'click role:button @data-session-step=switch:'+chosen+'\n'+await_(attached=chosen)+capture(public/'existing-graph.png')
        p=launch('baseline',owner,script);finish('baseline',p)
        own=await_(status='ready',attached=chosen)+click('coop-use')+await_(**{'coop-nodes':'1'})+capture(public/'owner-existing.png')+input_(2,guest_contact)+input_(3,export)+'click role:button Invite editor\n'+await_(**{'coop-access':'editor'})+capture(public/'owner-invited.png')+await_(**{'coop-nodes':'2','nodes':'2'})+capture(public/'owner-saw-guest.png')+'click role:button @data-session-step=switch:'+private_graph+'\n'+await_(attached=private_graph,**{'coop-available':'false'})+capture(public/'owner-private-during-coop.png')+await_(file=str(public/'guest-still-scoped.png'))+'click role:button @data-session-step=switch:'+chosen+'\n'+await_(attached=chosen)+click('coop-use')+await_(**{'coop-nodes':'2','coop-access':'editor'})+capture(public/'owner-returned-to-chosen.png')+await_(**{'coop-nodes':'3','nodes':'3'})+capture(public/'owner-ordinary-push.png')+await_(file=str(public/'guest-return.png'))+await_(file=str(private/'revoke-now'))+click('coop-revoke')+await_(**{'coop-access':'revoked'})+capture(public/'owner-revoked.png')+await_(file=str(public/'guest-withdrawn.png'))
        po=launch('owner-live',owner,own);until(public/'owner-invited.png',(po,))
        original_invitation=sha(export/'invitation.json')
        gs=await_(**{'coop-nodes':'1','sessions':'0','archives':'0'})+capture(public/'guest-joined.png')+input_(0,'https://shared.invalid/guest')+input_(1,'Guest source')+click('coop-add')+await_(**{'coop-nodes':'2','coop-status':'~Saved'})+capture(public/'guest-added.png')+await_(file=str(public/'owner-private-during-coop.png'))+capture(public/'guest-still-scoped.png')+await_(**{'coop-nodes':'3','relations':'1'})+capture(public/'guest-push.png')+'click role:button @data-key='+existing+'\n'+await_(selected=existing)+input_(1,'Guest revised existing source')+click('coop-title')+await_(**{'coop-status':'~Saved','coop':'~Guest revised existing source'})+capture(public/'guest-renamed.png')
        pg=launch('guest-live',guest,gs,True);until(public/'guest-added.png',(po,pg));activate('owner');until(public/'owner-private-during-coop.png',(po,pg));activate('guest');until(public/'guest-still-scoped.png',(po,pg));activate('owner');until(public/'owner-returned-to-chosen.png',(po,pg));assert sha(export/'invitation.json')==original_invitation
        prepare('ordinary-add',domain,chosen,'https://shared.invalid/ordinary-owner')
        activate('guest');finish('guest-live',pg)
        pg2=launch('guest-return',guest,await_(**{'coop-nodes':'3','coop':'~Guest revised existing source'})+capture(public/'guest-return.png')+await_(**{'coop-available':'false','nodes':'0','selected':''})+capture(public/'guest-withdrawn.png'),True)
        until(public/'guest-return.png',(po,pg2));(private/'revoke-now').touch();activate('owner');until(public/'owner-revoked.png',(po,pg2));activate('guest');finish('guest-return',pg2);finish('owner-live',po)
        write(public/'chosen-after-live.json',json.loads(prepare('read',domain,chosen)))
        owner.stop();guest.stop();owner.start();guest.start()
        cold=await_(status='ready',attached=chosen)+click('coop-use')+await_(**{'coop-nodes':'3','coop-access':'revoked','coop':'~Guest revised existing source'})+capture(public/'owner-cold-retained.png')+input_(3,export)+click('coop-export')+await_(**{'coop-nodes':'3'})+input_(0,'https://shared.invalid/owner-after-revoke')+input_(1,'Owner after revoke')+click('coop-add')+await_(**{'coop-nodes':'4'})+capture(public/'owner-cold-edited.png')+await_(file=str(public/'guest-cold-denied.png'))+capture(public/'owner-cold-refused.png')
        pc=launch('owner-cold',owner,cold);until(public/'owner-cold-edited.png',(pc,));assert sha(export/'invitation.json')==original_invitation
        gc=launch('guest-cold',guest,await_(**{'coop-status':'~Revoked','coop-available':'false','nodes':'0','sessions':'0','archives':'0'})+capture(public/'guest-cold-denied.png'),True)
        finish('guest-cold',gc);finish('owner-cold',pc)
        write(public/'chosen-after-cold.json',json.loads(prepare('read',domain,chosen)))
        p=launch('private-return',owner,await_(status='ready')+'click role:button @data-session-step=switch:'+private_graph+'\n'+await_(attached=private_graph)+capture(public/'private-after.png'));finish('private-return',p)
        write(public/'private-after.json',json.loads(prepare('read',domain,private_graph)))
        before=json.loads((public/'private-before.json').read_text())['graphs'][0];after=json.loads((public/'private-after.json').read_text())['graphs'][0]
        for field in ['session','graph','facets']:assert before[field]==after[field],field
        def changes(path):
            cards=json.loads(path.read_text())['cards']
            return next(v['value'] for card in cards for v in card['values'] if v['label']=='Changes')
        assert changes(public/'private-before.json')==changes(public/'private-after.json')
        owner.stop();guest.stop()
        phases=['baseline','owner-live','guest-live','guest-return','owner-cold','guest-cold','private-return']
        receipts=[json.loads((public/(name+'.json')).read_text()) for name in phases]
        pids=[int(receipt['final']['pid']) for receipt in receipts];assert len(set(pids))==7
        write(public/'qualification.json',{'result':'ok','native_processes':pids,'original_invitation_sha256':original_invitation,'cold_invitation_sha256':sha(export/'invitation.json'),'binary_sha256':sha(args.binary),'djinn_sha256':sha(args.djinn),'prepare_sha256':sha(args.prepare),'graphs':ids,'existing_surface':existing,'phases':phases,'custodian_launches':owner.starts+guest.starts,'all_owned_processes_stopped':all(p.poll() is not None for p in processes)})
        print(json.dumps({'result':'ok','evidence':str(public)}))
    finally:
        table=subprocess.check_output(['ps','-axo','pid=,command='],text=True)
        for line in table.splitlines():
            fields=line.strip().split(None,1)
            if len(fields)==2 and any(fields[1].startswith(str(private/(role+'.app')/'Contents/MacOS/graphshell_mere')) for role in ['owner','guest']):
                try:os.kill(int(fields[0]),signal.SIGTERM)
                except ProcessLookupError:pass
        for p in processes:
            if p.poll() is None:p.terminate()
        owner.terminate_owned();guest.terminate_owned()

if __name__=='__main__':main()
