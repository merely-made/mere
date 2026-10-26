#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Synthetic macOS H1 fixture. Missing prerequisites never count as denials."""
from __future__ import annotations
import argparse, hashlib, json, os, pathlib, plistlib, secrets, shutil, socket
import stat, subprocess, sys, time, uuid, re
HERE = pathlib.Path(__file__).resolve().parent
TEAM = 'HWVZ5S7ACJ'
IDENTITY = 'BF9216740E1A5CCB99CB26B0D632D37344BF8608'
PREFIX = 'com.merelymade.djinn.h1'
# This constant is passed directly to a trusted isolated interpreter. It verifies
# the root-owned copy before executing any byte of the user-authored installer.
INSTALL_VERIFY_LAUNCHER = ('import hashlib,os,sys; p=sys.argv[1]; '
    'actual=hashlib.sha256(open(p,"rb").read()).hexdigest(); '
    'assert actual==sys.argv[2],"installer digest mismatch"; '
    'os.execv("/usr/bin/python3",["/usr/bin/python3","-I",p]+sys.argv[3:])')


def write_json(path, value):
    pathlib.Path(path).write_text(json.dumps(value, indent=2, sort_keys=True)+'\n')


def command(args, *, timeout=30, check=True, env=None):
    try:
        p = subprocess.run([str(x) for x in args], text=True, stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, timeout=timeout, env=env)
    except OSError as e:
        if check: raise
        return {'returncode': None, 'spawn_errno': e.errno, 'stdout': '', 'stderr': str(e)}
    except subprocess.TimeoutExpired:
        if check: raise
        return {'returncode': None, 'timeout': True, 'stdout': '', 'stderr': 'deadline exceeded'}
    if check and p.returncode:
        raise RuntimeError(f'{args[0]} failed ({p.returncode}): {p.stderr[-2000:]} {p.stdout[-1000:]}')
    return {'returncode': p.returncode, 'stdout': p.stdout, 'stderr': p.stderr}


def plist(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('wb') as f: plistlib.dump(value, f, sort_keys=True)


def native_json(exe, mode, *args):
    raw = command([exe, mode, *args], check=False, timeout=12)
    lines = raw['stdout'].splitlines()
    try: value = json.loads(lines[-1]) if lines else None
    except ValueError: value = None
    return {'process': raw, 'value': value}


def executable(app): return app/'Contents/MacOS/Fixture'


def local_module(name):
    # -I deliberately excludes the script directory from sys.path.
    import importlib.util
    spec=importlib.util.spec_from_file_location('h1_'+name,HERE/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module


def prepare(args):
    out = pathlib.Path(args.output).resolve()
    if out.exists(): raise SystemExit('Refusing an existing output directory')
    out.mkdir(parents=True, mode=0o700)
    run = uuid.uuid4().hex
    receiver = getattr(args,'receiver_user',None)
    if receiver and not re.fullmatch('djinnh1',receiver):
        raise SystemExit('The separate-account fixture is restricted to the dedicated djinnh1 account')
    runtime = (pathlib.Path('/Users')/receiver/'Library/Application Support/Djinn H1 Fixtures'/run
               if receiver else out)
    ids = {k: PREFIX+'.'+k+'.r'+run for k in ['controller','custodian','worker','outsider','fake']}
    ids['custodian'] = PREFIX+'.custodian'
    # Protected deployment always uses this root-owned parent, never /Applications.
    installed = pathlib.Path('/Library/Application Support/Djinn H1 Fixtures')/run/'DjinnH1.app'
    app = out/'DjinnH1.app'
    custodian = app/'Contents/Library/Helpers/Custodian.app'
    worker = app/'Contents/XPCServices'/f'{ids["worker"]}.xpc'
    outsider = out/'Unrelated.app'
    fake = out/'FakeCustodian.app'
    state = out/'synthetic-state'; (state/'custody').mkdir(parents=True,mode=0o700)
    (state/'blobs').mkdir(mode=0o700)
    for p in [state/'custody/canary',state/'blobs/canary']:
        p.write_bytes(secrets.token_bytes(32)); p.chmod(0o600)
    (out/'vault-symlink').symlink_to(state/'custody/canary')
    agent = ids['custodian']+'.r'+run+'.plist'
    c = {'runID':run,'team':args.team,'controllerID':ids['controller'],'custodianID':ids['custodian'],
         'workerID':ids['worker'],'machService':ids['custodian']+'.r'+run,'fakeService':ids['fake'],
         'protectedTargets':{'binary':str(installed/'Contents/Library/Helpers/Custodian.app/Contents/MacOS/Fixture'),
          'config':str(installed/'Contents/Library/Helpers/Custodian.app/Contents/Resources/fixture.json'),
          'agent':str(installed/'Contents/Library/LaunchAgents'/agent)},'protectedDirectory':str(installed.parent),
         'keyGroup':args.team+'.'+ids['custodian'],'keyService':PREFIX+'.synthetic-envelope',
         'agentPlist':agent,'vaultCanary':str(runtime/'synthetic-state/custody/canary'),'vaultSymlink':str(runtime/'vault-symlink'),
         'blobCanary':str(runtime/'synthetic-state/blobs/canary'),'vaultCanaryHash':hashlib.sha256((state/'custody/canary').read_bytes()).hexdigest(),'sealedVault':str(runtime/'synthetic-state/custody/vault.aesgcm'),
         'selectedInput':'Synthetic selected input for this worker only.',
         'signingIdentity':args.identity,'forgedApp':str(runtime/'AttackerMintedCustodian.app'),
         'forgedEntitlements':str(runtime/'forged-custodian.entitlements.plist')}
    if receiver:
        keychain=pathlib.Path(command(['/usr/bin/security','default-keychain','-d','user'])['stdout'].strip().strip('"'))
        key_stat=keychain.lstat()
        if keychain.is_symlink() or not stat.S_ISREG(key_stat.st_mode) or key_stat.st_uid!=os.getuid():
            raise SystemExit('Signing account default Keychain must be its own regular file')
        c['signerKeychainFile']=str(keychain)
    parent = out/'launchd-parent.plist'
    plist(parent, {'signing-identifier':'com.apple.xpc.launchd','validation-category':1})
    command(['/usr/bin/codesign','--validate-constraint',parent])
    roles = [('CONTROLLER',app,ids['controller']),('CUSTODIAN',custodian,ids['custodian']),
             ('WORKER',worker,ids['worker']),('OUTSIDER',outsider,ids['outsider']),('OUTSIDER',fake,ids['fake'])]
    build_logs = {}
    profile_info = None
    if args.profile:
        profile = pathlib.Path(args.profile).resolve()
        decoded = subprocess.check_output(['/usr/bin/security','cms','-D','-i',str(profile)])
        metadata = plistlib.loads(decoded)
        ent = metadata.get('Entitlements',{})
        groups = ent.get('keychain-access-groups',[])
        application = ent.get('com.apple.application-identifier',ent.get('application-identifier',''))
        def matches(pattern, value):
            return pattern == value or (pattern.endswith('*') and value.startswith(pattern[:-1]))
        if args.team not in metadata.get('TeamIdentifier',[]): raise SystemExit('Profile team mismatch')
        if not any(matches(g,c['keyGroup']) for g in groups): raise SystemExit('Profile does not authorize custodian group')
        if not matches(application,c['keyGroup']): raise SystemExit('Profile does not authorize custodian app ID')
        from datetime import timezone
        if args.identity.upper() not in {hashlib.sha1(x).hexdigest().upper() for x in metadata.get('DeveloperCertificates',[])}: raise SystemExit('Profile certificate does not match the selected signing identity')
        if metadata['ExpirationDate'].replace(tzinfo=timezone.utc).timestamp() <= time.time(): raise SystemExit('Expired provisioning profile')
        profile_info = {'sha256':hashlib.sha256(profile.read_bytes()).hexdigest(),
                        'expiration':metadata['ExpirationDate'].isoformat(), 'group_authorized':True}
    for role,bundle,identifier in roles:
        binary = executable(bundle); binary.parent.mkdir(parents=True)
        resources = bundle/'Contents/Resources'; resources.mkdir()
        info = {'CFBundleIdentifier':identifier,'CFBundleExecutable':'Fixture','CFBundleName':'Djinn H1 Synthetic '+role.title(),
                'CFBundleDisplayName':'Djinn H1 Synthetic '+role.title(),
                'CFBundleVersion':'1','CFBundleShortVersionString':'0.1',
                'CFBundlePackageType':'XPC!' if role=='WORKER' else 'APPL',
                'LSMinimumSystemVersion':'14.0','H1ControllerID':ids['controller'],'H1Team':args.team}
        if role=='WORKER': info['XPCService']={'ServiceType':'Application'}
        plist(bundle/'Contents/Info.plist',info)
        write_json(resources/'fixture.json',c)
        build_logs[role+identifier] = command(['xcrun','swiftc','-swift-version','5','-warnings-as-errors','-O',
            '-D',role,HERE/'Fixture.swift','-o',binary,'-framework','Foundation','-framework','Security',
            '-framework','CryptoKit','-framework','ServiceManagement'],timeout=120)
        entitlements = {}
        if role=='WORKER': entitlements={'com.apple.security.app-sandbox':True}
        if role=='CUSTODIAN' and args.profile:
            entitlements={'com.apple.application-identifier':c['keyGroup'],
                          'com.apple.developer.team-identifier':args.team,
                          'keychain-access-groups':[c['keyGroup']]}
            shutil.copyfile(args.profile,bundle/'Contents/embedded.provisionprofile')
        ent_file=out/(role.lower()+identifier+'.entitlements.plist');plist(ent_file,entitlements)
        sign=['/usr/bin/codesign','--force','--sign',args.identity,'--options','runtime',
              '--timestamp=none','--entitlements',ent_file]
        if role=='CUSTODIAN': sign += ['--launch-constraint-parent',parent,'--enforce-constraint-validity']
        # Controller must be signed last after its nested helpers/services exist.
        if role!='CONTROLLER':
            command(sign+[bundle]); command(['/usr/bin/codesign','--verify','--strict',bundle])
    adhoc=out/'AdHocUnrelated.app'
    shutil.copytree(outsider,adhoc)
    command(['/usr/bin/codesign','--force','--sign','-','--options','runtime',adhoc])
    if args.profile:
        # Prepared unsigned payload: only the unrelated app's own codesign subprocess may authorize it.
        forged=out/'AttackerMintedCustodian.app'
        shutil.copytree(outsider,forged)
        forged_info=plistlib.loads((forged/'Contents/Info.plist').read_bytes())
        forged_info['CFBundleIdentifier']=ids['custodian']
        plist(forged/'Contents/Info.plist',forged_info)
        shutil.rmtree(forged/'Contents/_CodeSignature')
        shutil.copyfile(args.profile,forged/'Contents/embedded.provisionprofile')
        plist(out/'forged-custodian.entitlements.plist',{
             'com.apple.application-identifier':c['keyGroup'],
             'com.apple.developer.team-identifier':args.team,'keychain-access-groups':[c['keyGroup']]})
    injection=out/'SyntheticInjection.dylib';injection_control=out/'UnhardenedInjectionControl'
    command(['xcrun','clang','-Wall','-Wextra','-Werror','-dynamiclib',HERE/'Injection.c','-o',injection])
    command(['xcrun','clang','-Wall','-Wextra','-Werror',HERE/'InjectionControl.c','-o',injection_control])
    for binary in [injection,injection_control]:
        command(['/usr/bin/codesign','--force','--sign',args.identity,'--timestamp=none',binary])
    hardened_probe=out/'HardenedLaunchProbe'
    command(['xcrun','clang','-Wall','-Wextra','-Werror',HERE/'InjectionControl.c','-o',hardened_probe])
    command(['/usr/bin/codesign','--force','--sign',args.identity,'--timestamp=none','--options','runtime',
             '--launch-constraint-parent',parent,'--enforce-constraint-validity',hardened_probe])
    agent_data={'Label':c['machService'],'BundleProgram':'Contents/Library/Helpers/Custodian.app/Contents/MacOS/Fixture',
                'MachServices':{c['machService']:True},'RunAtLoad':False,
                'SpawnConstraint':{'team-identifier':args.team,'signing-identifier':ids['custodian']}}
    plist(app/'Contents/Library/LaunchAgents'/agent,agent_data)
    controller_ent=out/('controller'+ids['controller']+'.entitlements.plist')
    command(['/usr/bin/codesign','--force','--sign',args.identity,'--options','runtime','--timestamp=none',
             '--entitlements',controller_ent,app])
    command(['/usr/bin/codesign','--verify','--deep','--strict',app])
    # Exercise the installer's exact read-only command before requesting root.
    verification_command=local_module('install_fixture').verification_command
    installation_preflight={}
    for role,bundle,identifier in roles[:3]:
        argv=verification_command(bundle,args.team,identifier)
        installation_preflight[role]={'argv':argv,'result':command(argv)}
    wrong_id=verification_command(app,args.team,ids['controller']+'.wrong')
    rejection=command(wrong_id,check=False)
    if rejection['returncode']!=3 or 'failed to satisfy specified code requirement' not in rejection['stderr']:
        raise RuntimeError('Installer identity negative control did not reject the wrong identity')
    installation_preflight['wrong_identity_control']={'argv':wrong_id,'result':rejection}
    signing={}
    for role,bundle,identifier in roles:
        signing[identifier]=command(['/usr/bin/codesign','-d','--verbose=4','--entitlements',':-',bundle])
    manifest={'schema':1,'runID':run,'app':str(app),'custodian':str(custodian),'outsider':str(outsider),
              'fake':str(fake),'adhoc':str(adhoc),'installedApp':str(installed),'config':c,
              'profile':profile_info,'profileRequired':not bool(args.profile),'owner_uid':os.getuid(),
              'source_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [HERE/n for n in ['Fixture.swift','run.py','install_fixture.py','Injection.c','InjectionControl.c','test_evidence_gates.py']]},
              'signing':signing,'build':build_logs,'installation_preflight':installation_preflight}
    if receiver:
        import tempfile
        # Prove the same unsigned attack payload can be signed on the signing
        # workstation. Use a disposable copy, preserving the receiving payload.
        with tempfile.TemporaryDirectory(prefix='h1-signer-control-',dir=out.parent) as control:
            target=pathlib.Path(control)/'AttackerMintedCustodian.app'
            shutil.copytree(out/'AttackerMintedCustodian.app',target)
            argv=['/usr/bin/codesign','--force','--sign',args.identity,'--options','runtime','--timestamp=none',
                  '--entitlements',str(out/'forged-custodian.entitlements.plist'),str(target)]
            positive={'argv':argv,'signing':command(argv),
                'verification':command(verification_command(target,args.team,ids['custodian'])),
                'unsigned_binary_sha256':hashlib.sha256(executable(out/'AttackerMintedCustodian.app').read_bytes()).hexdigest()}
        tools=out/'receiver-tools';tools.mkdir()
        for name in ['run.py','install_fixture.py','bootstrap_receiver.py']:
            shutil.copyfile(HERE/name,tools/name)
        for name in ['bootstrap_receiver.py','test_receiver_setup.py']:
            manifest['source_sha256'][name]=hashlib.sha256((HERE/name).read_bytes()).hexdigest()
        manifest['receiver']={'user':receiver,'home':'/Users/'+receiver,'builder_uid':os.getuid(),
                             'fixture':str(runtime),'package':str(installed.parent/'ReceivingPayload'),
                             'signer_keychain_metadata':{'path':str(keychain),'uid':key_stat.st_uid,
                                 'mode':oct(stat.S_IMODE(key_stat.st_mode))},'signer_positive_control':positive}
        # Runtime app paths refer to the receiving account; preparation still
        # retains the original bundle source path separately for provenance.
        manifest['prepared_app']=str(app)
        for name in ['app','custodian','outsider','fake','adhoc']:
            manifest[name]=str(runtime/pathlib.Path(manifest[name]).relative_to(out))
    # Exact tree hashes bind the installer review and later receiving run.
    manifest['bundle_sha256']={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest()
                              for p in out.rglob('*') if p.is_file() and not p.is_symlink()}
    manifest['executable_paths']=[str(p.relative_to(out)) for p in out.rglob('*')
                                  if p.is_file() and not p.is_symlink() and p.stat().st_mode&0o111]
    write_json(out/'manifest.json',manifest)
    # Installation uses a separately reviewed root-owned program, never this user-writable runner.
    helper=HERE/'install_fixture.py'
    helper_hash=hashlib.sha256(helper.read_bytes()).hexdigest()
    manifest_hash=hashlib.sha256((out/'manifest.json').read_bytes()).hexdigest()
    protected_helper='/private/var/root/djinn-h1-installer-'+run+'.py'
    import shlex
    q=shlex.quote
    instructions=(f"sudo /usr/bin/install -o root -g wheel -m 500 {q(str(helper))} {q(protected_helper)}\n"
        f"sudo /usr/bin/python3 -I -c {q(INSTALL_VERIFY_LAUNCHER)} {q(protected_helper)} {helper_hash} "
        f"--fixture {q(str(out))} --uid {os.getuid()} --program-sha256 {helper_hash} --manifest-sha256 {manifest_hash}\n")
    (out/'INSTALL-COMMANDS.txt').write_text(instructions)
    if receiver:
        account_command=('/usr/sbin/sysadminctl -addUser djinnh1 -fullName '+q('Djinn H1 Synthetic Receiver')+
            ' -home /Users/djinnh1 -shell /bin/zsh -password -')
        (out/'SETUP-COMMANDS.txt').write_text(
            '# Run in the signing account. The password prompt is for the new local standard account.\n'
            '# Do not reuse a personal password. No password is included in these files.\n'
            'sudo '+account_command+'\n\n'+instructions)
        (out/'RECEIVER-COMMANDS.txt').write_text(local_module('install_fixture').receiving_commands(manifest,manifest_hash))
    print(json.dumps({'fixture':str(out),'profile':bool(args.profile),'installedApp':str(installed)}))


def check_hashes(out, m):
    for rel,expected in m['bundle_sha256'].items():
        p=out/rel
        if p.is_symlink() or not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=expected:
            raise RuntimeError('Prepared artifact changed: '+rel)


def launch_plist(path,exe,service,stdout,stderr):
    plist(path,{'Label':service,'ProgramArguments':[str(exe)],'MachServices':{service:True},'ThrottleInterval':1,
                'StandardOutPath':str(stdout),'StandardErrorPath':str(stderr)})


def run(args):
    out=pathlib.Path(args.fixture).resolve();manifest_bytes=(out/'manifest.json').read_bytes();m=json.loads(manifest_bytes)
    if m.get('receiver'):
        if hashlib.sha256(manifest_bytes).hexdigest()!=getattr(args,'manifest_sha256',None):
            raise RuntimeError('The reviewed receiving manifest digest is required')
        bootstrap=local_module('bootstrap_receiver')
        bootstrap.validate_package(pathlib.Path(m['receiver']['package']),args.manifest_sha256)
        context=bootstrap.receiving_context(m)
        if not context['valid']:raise RuntimeError('Separate-account prerequisites failed: '+json.dumps(context))
        if args.command!='run-protected':raise RuntimeError('Separate-account evidence requires protected mode')
    else:context=None
    check_hashes(out,m)
    c=dict(m['config']);domain=f'gui/{os.getuid()}'
    protected=args.command=='run-protected'
    app=pathlib.Path(m['installedApp'] if protected else m['app'])
    if protected and (m['profileRequired'] or not app.exists()): raise SystemExit('Protected prerequisites missing')
    result_path=out/('protected-results.json' if protected else 'local-results.json')
    if result_path.exists(): raise SystemExit('Refusing to overwrite a receiving result')
    result={'schema':1,'syntheticOnly':True,'protectedInstallation':protected,'profilePresent':not m['profileRequired'],
            'tests':{},'limits':[],'macOS':command(['/usr/bin/sw_vers']), 'uid':os.getuid()}
    if context is not None:
        result['separateAccountRequired']=True
        result['receivingContext']=context
    exe=executable(app);cust=executable(app/'Contents/Library/Helpers/Custodian.app')
    if protected:
        p=app
        while p!=p.parent:
            st=p.lstat()
            if p.is_symlink() or st.st_uid!=0 or st.st_mode&0o022:
                raise RuntimeError('Writable or non-root install ancestor: '+str(p))
            p=p.parent
        for rel,expected in m['bundle_sha256'].items():
            if rel.startswith('DjinnH1.app/'):
                installed_file=app/rel.removeprefix('DjinnH1.app/')
                if installed_file.is_symlink() or hashlib.sha256(installed_file.read_bytes()).hexdigest()!=expected:
                    raise RuntimeError('Installed artifact mismatch: '+rel)
        result['installed_signature']=command(['/usr/bin/codesign','--verify','--deep','--strict',app])
        result['registration']=native_json(exe,'register')
        if result['registration']['process']['returncode']:raise RuntimeError('SMAppService registration failed')
        if (result['registration']['value'] or {}).get('status')!=1:
            pending=out/('pending-registration-'+str(time.time_ns())+'.json')
            write_json(pending,result)
            print(json.dumps({'pending_registration':str(pending),'status':result['registration']['value'],
                             'action':'Approve Djinn H1 Synthetic Controller in macOS Login Items, then rerun run-protected.'}))
            return 3
    jobs=[]
    try:
        if protected:
            pass
        else:
            result['limits'] += ['User-writable local launchctl registration proves launch origin only, not protected installation or SMAppService.',
                                 'Without successful legitimate Keychain use, attacker key failures are not a key-isolation pass.']
            agent=out/'local-agent.plist'
            launch_plist(agent,cust,c['machService'],out/'custodian.stdout',out/'custodian.stderr')
            command(['/bin/launchctl','bootstrap',domain,agent]);jobs.append(c['machService'])
        fake_agent=out/'fake-agent.plist'
        fake_args=[str(executable(pathlib.Path(m['fake']))),'fake-server']
        plist(fake_agent,{'Label':c['fakeService'],'ProgramArguments':fake_args,
                         'MachServices':{c['fakeService']:True},'StandardOutPath':str(out/'fake.stdout'),
                         'StandardErrorPath':str(out/'fake.stderr')})
        command(['/bin/launchctl','bootstrap',domain,fake_agent]);jobs.append(c['fakeService'])
        result['tests']['authorized_client']=native_json(exe,'client')
        hello=(result['tests']['authorized_client']['value'] or {}).get('hello',{})
        pid=hello.get('pid')
        if pid:
            # Native probes take only a fixture target PID; paths/identity remain bundle config.
            result['custodian_launch_record']=command(['/bin/launchctl','print',domain+'/'+c['machService']],check=False)
        with socket.socket(socket.AF_INET,socket.SOCK_STREAM) as net:
            net.bind(('127.0.0.1',0));net.listen(4)
            port=net.getsockname()[1]
            result['tests']['worker']=native_json(exe,'worker',str(pid or 0),str(port))
            result['tests']['unrelated_same_user']=native_json(executable(pathlib.Path(m['outsider'])),'inspect',str(pid or 0),str(port))
        result['tests']['adhoc_same_user']=native_json(executable(pathlib.Path(m['adhoc'])),'inspect')
        result['tests']['adhoc_custody_call']=native_json(executable(pathlib.Path(m['adhoc'])),'client')
        result['tests']['unrelated_custody_call']=native_json(executable(pathlib.Path(m['outsider'])),'client')
        result['tests']['fake_server']=native_json(exe,'fake-client')
        result['tests']['unsafe_fake_control']=native_json(exe,'unsafe-fake-control')
        if not m['profileRequired']:
            result['tests']['same_user_signing_oracle']=native_json(executable(pathlib.Path(m['outsider'])),'signer-oracle')
            oracle=(result['tests']['same_user_signing_oracle']['value'] or {})
            if oracle.get('signerUnattended'):
                forged=pathlib.Path(c['forgedApp'])
                result['tests']['attacker_minted_identity']=native_json(executable(forged),'inspect')
                result['tests']['attacker_minted_identity']['signatureVerification']=command([
                    '/usr/bin/codesign','--verify','--strict',forged],check=False)
        direct=command([cust],check=False,timeout=8)
        result['tests']['direct_child_launch']=direct
        result['tests']['injected_child_launch']=command([cust,'--config','/tmp/not-a-custody-config'],check=False,
            env={**os.environ,'DYLD_INSERT_LIBRARIES':'/tmp/h1-nonexistent-injection.dylib'},timeout=8)
        marker=out/'injection-control.marker'
        injected_environment={'DYLD_INSERT_LIBRARIES':str(out/'SyntheticInjection.dylib'),
                              'H1_FIXTURE_INJECT_MARKER':str(marker)}
        result['tests']['injection_control']=command([out/'UnhardenedInjectionControl'],
            env={**os.environ,**injected_environment},check=False)
        result['tests']['injection_control']['markerCreated']=marker.exists()
        hostile_label=c['machService']+'.hostile-launch'
        hostile_agent=out/'hostile-agent.plist'
        hostile_marker=out/'hardened-injection.marker'
        plist(hostile_agent,{'Label':hostile_label,'ProgramArguments':[str(out/'HardenedLaunchProbe')],'RunAtLoad':True,
              'EnvironmentVariables':{**injected_environment,'H1_FIXTURE_INJECT_MARKER':str(hostile_marker)},
              'StandardOutPath':str(out/'hostile.stdout'),'StandardErrorPath':str(out/'hostile.stderr')})
        command(['/bin/launchctl','bootstrap',domain,hostile_agent]);jobs.append(hostile_label)
        deadline=time.monotonic()+4
        hostile_log=''
        while time.monotonic()<deadline:
            if (out/'hostile.stdout').exists():hostile_log=(out/'hostile.stdout').read_text()
            if '"launchProbeReady":true' in hostile_log:break
            time.sleep(0.05)
        result['tests']['hostile_launchd_environment']={'ready': '"launchProbeReady":true' in hostile_log,
            'markerCreated':hostile_marker.exists(),'stdout':hostile_log,
            'stderr':(out/'hostile.stderr').read_text() if (out/'hostile.stderr').exists() else ''}
        # Restart is requested through the fixed OS service identity, never a child spawn.
        result['restart']=command(['/bin/launchctl','kickstart','-k',domain+'/'+c['machService']],check=False)
        result['tests']['cold_reopen']=native_json(exe,'client')
    finally:
        result['key_cleanup']=native_json(exe,'cleanup-key')
        for job in reversed(jobs): command(['/bin/launchctl','bootout',domain+'/'+job],check=False)
        if protected: result['unregister']=native_json(exe,'unregister')
        result['fake_observations']=(out/'fake.stdout').read_text() if (out/'fake.stdout').exists() else ''
        result['evaluation']=evaluate(result)
        write_json(result_path,result)
    print(json.dumps({'result':str(result_path),'evaluation':result['evaluation']}))
    return 0 if result['evaluation']['all_required_pass'] else 2


def evaluate(r):
    t=r['tests'];v=lambda name: (t.get(name,{}).get('value') or {})
    w=v('worker');a=v('unrelated_same_user');client=v('authorized_client');hello=client.get('hello',{})
    key_good=hello.get('key',{}).get('keyStatus')==0 and hello.get('key',{}).get('keyBytes')==32
    client_good=client.get('signatureVerified') is True
    hello_good=hello.get('ok') is True and isinstance(hello.get('pid'),int) and hello.get('ppid')==1
    def launch_denied(name):
        p=t.get(name,{})
        return hello_good and (p.get('returncode')==-9 or p.get('spawn_errno') in [1,153]) and not p.get('timeout')
    try: fake_events=[json.loads(line) for line in r.get('fake_observations','').splitlines()]
    except ValueError: fake_events=[]
    deny_errno={1,13}
    def protected_write_denied(probe, kind):
        value=probe.get('protectedWrites',{}).get(kind,{})
        return r.get('protectedInstallation') is True and value.get('writable') is False and value.get('errno') in deny_errno
    checks={
      'worker_protected_binary_write_denied':protected_write_denied(w,'binary'),
      'unrelated_protected_binary_write_denied':protected_write_denied(a,'binary'),
      'unrelated_protected_config_write_denied':protected_write_denied(a,'config'),
      'unrelated_protected_agent_write_denied':protected_write_denied(a,'agent'),
      'unrelated_install_directory_create_denied':r.get('protectedInstallation') is True and a.get('protectedCreate',{}).get('created') is False and a.get('protectedCreate',{}).get('errno') in deny_errno,
      'worker_vault_denied': w.get('vault',{}).get('opened') is False and w.get('vault',{}).get('errno') in deny_errno,
      'worker_symlink_denied': w.get('symlink',{}).get('opened') is False and w.get('symlink',{}).get('errno') in deny_errno,
      'same_user_symlink_positive_control':a.get('symlink',{}).get('opened') is True,
      'worker_blob_denied': w.get('blob',{}).get('opened') is False and w.get('blob',{}).get('errno') in deny_errno,
      'same_user_file_positive_control':a.get('vault',{}).get('opened') is True and a.get('blob',{}).get('opened') is True,
      'worker_own_file_positive_control':w.get('ownFileControl') is True,
      'worker_no_inherited_fixture_handles':w.get('inheritedFixturePaths')==[] and w.get('inheritedVaultCanary') is False,
      'worker_selected_input_positive_control':w.get('selectedInputHash')==hashlib.sha256(b'Synthetic selected input for this worker only.').hexdigest(),
      'worker_network_denied':w.get('network',{}).get('connected') is False and w.get('network',{}).get('errno') in deny_errno,
      'same_user_network_positive_control':a.get('network',{}).get('connected') is True,
      'independent_launch_parent':hello.get('ppid')==1 and isinstance(hello.get('pid'),int),
      'direct_child_launch_denied':launch_denied('direct_child_launch'),
      'hostile_child_launch_denied':launch_denied('injected_child_launch'),
      'injection_positive_control':t.get('injection_control',{}).get('returncode')==0 and t.get('injection_control',{}).get('markerCreated') is True,
      'hardened_launchd_environment_probe':hello_good and t.get('hostile_launchd_environment',{}).get('ready') is True and t.get('hostile_launchd_environment',{}).get('markerCreated') is False,
      'authorized_key_access':key_good,
      'authorized_signature_control':client_good,
      'worker_key_access_denied':key_good and w.get('keyObtained') is False and w.get('keyStatus') in [-34018,-25300],
      'unrelated_key_access_denied':key_good and a.get('keyObtained') is False and a.get('keyStatus') in [-34018,-25300],
      'worker_custodian_memory_denied':hello_good and a.get('selfMemoryControl',{}).get('taskPortObtained') is True and w.get('memory',{}).get('taskPortObtained') is False,
      'unrelated_custodian_memory_denied':hello_good and a.get('selfMemoryControl',{}).get('taskPortObtained') is True and a.get('memory',{}).get('taskPortObtained') is False,
      'adhoc_custody_denied':hello_good and v('adhoc_custody_call').get('secretTransmitted') is False,
      'adhoc_key_access_denied':key_good and v('adhoc_same_user').get('keyObtained') is False and v('adhoc_same_user').get('keyStatus') in [-34018,-25300],
      'unrelated_custody_denied':hello_good and v('unrelated_custody_call').get('secretTransmitted') is False,
      'fake_server_rejected_before_secret':hello_good and v('fake_server').get('secretTransmitted') is False and not any(e.get('lane')=='pinned' and e.get('secretPresent') for e in fake_events),
      'fake_server_capture_positive_control':v('unsafe_fake_control').get('secretTransmitted') is True and any(e.get('lane')=='unsafe-control' and e.get('secretPresent') for e in fake_events),
      'adjacent_scope_denied':hello_good and client.get('adjacentScope',{}).get('refused')=='operation-or-scope',
      'sealed_cold_reopen':client_good and v('cold_reopen').get('signatureVerified') is True and v('cold_reopen').get('publicKey')==client.get('publicKey') and v('cold_reopen').get('hello',{}).get('instance')!=hello.get('instance') and v('cold_reopen').get('hello',{}).get('ppid')==1,
      'protected_installation':r.get('protectedInstallation') is True,
    }
    oracle=v('same_user_signing_oracle'); forged=v('attacker_minted_identity')
    signer_output=oracle.get('output','')
    signer_denial=(oracle.get('signerUnattended') is False and oracle.get('signerExit')==1 and not oracle.get('timedOut')
         and any(message in signer_output for message in ['The specified item could not be found in the keychain',
                                                          'no identity found','User interaction is not allowed']))
    checks['signer_authority_not_available_to_same_user_app']=key_good and signer_denial and forged.get('keyObtained') is not True
    if r.get('separateAccountRequired'):
        context=r.get('receivingContext',{})
        checks['separate_nonadmin_gui_account']=context.get('valid') is True
        f=a.get('signerKeychainFile',{})
        checks['unrelated_signer_keychain_file_denied']=key_good and f.get('opened') is False and f.get('errno') in deny_errno
        checks['signer_payload_positive_control']=context.get('signerPositiveControl') is True
    signer_finding={'tested':bool(oracle),'unattended_signing':oracle.get('signerUnattended'),
         'synthetic_key_obtained':forged.get('keyObtained') is True,
         'synthetic_root_recovered':bool(client.get('publicKey')) and forged.get('syntheticRootPublicKey')==client.get('publicKey')}
    return {'checks':checks,'signing_authority_finding':signer_finding,
            'same_user_resistance_qualified':all(checks.values()),'all_required_pass':all(checks.values()),
            'passed':sum(checks.values()),'total':len(checks)}


def main():
    p=argparse.ArgumentParser(description=__doc__);subs=p.add_subparsers(dest='command',required=True)
    prep=subs.add_parser('prepare');prep.add_argument('--output',required=True);prep.add_argument('--receiver-user')
    prep.add_argument('--identity',default=IDENTITY);prep.add_argument('--team',default=TEAM);prep.add_argument('--profile')
    for name in ['run-local','run-protected']:
        sub=subs.add_parser(name);sub.add_argument('--fixture',required=True)
        sub.add_argument('--manifest-sha256')
    args=p.parse_args()
    if args.command=='prepare':prepare(args)
    else:return run(args)
    return 0

if __name__=='__main__':sys.exit(main())
