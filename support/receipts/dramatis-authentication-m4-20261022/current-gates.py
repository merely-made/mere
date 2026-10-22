from pathlib import Path
import json,os,re,shutil,subprocess,sys,time
r=Path('/Users/markik/Code/worktrees/mere-authentication-m4');o=Path('/Users/markik/Code/output/dauth-m4-20261022');assert(o/'.dramatis-generated').read_text()=='synthetic authentication qualification'
def git(*args):return subprocess.check_output(['git',*args],cwd=r,text=True).strip()
assert not git('status','--porcelain');assert git('rev-parse','HEAD')=='bb1deba82bc1bbad2eb0e4df6d14779d1e6459a0'
assert(o/'integration-public.zip').exists()
assert not any(str(o/'frozen') in line for line in subprocess.check_output(['ps','-axo','pid=,comm='],text=True).splitlines())
names=['frozen','scripted','native-cold','s','software-gates.json','host-build.jsonl','integration-public.zip','integration-post-run.json']
for row in json.loads((o/'software-gates.json').read_text())['commands']:names.append(row['name']+'.log')
for mode in ['scripted','native-cold']:
 job='gui/'+str(os.getuid())+'/org.merely.dramatis.authentication.m4.20261022.'+mode
 state=subprocess.check_output(['launchctl','print',job],text=True);assert 'last exit code = 0' in state and not re.search(r'^\s*pid = ',state,re.M)
 subprocess.run(['launchctl','bootout',job],check=True)
 names.extend([mode+'.plist',mode+'-preflight.json',mode+'-task-result.json',mode+'-launch.log',mode+'-agent.log'])
for name in names:
 src=o/name;dst=o/('baselinebb1-'+name);assert src.exists(),name;assert not dst.exists();src.rename(dst)
git('fetch',str(o/'latest.bundle'),'HEAD');git('merge','--ff-only','FETCH_HEAD');source=git('rev-parse','HEAD');assert source=='8d8f7baff76e9d484106134c213fd9c89316f6e1'
assert subprocess.run(['git','diff','--quiet','b09af10e1..HEAD','--','ports/djinn/castellan','ports/djinn/src/identity_ui.rs'],cwd=r).returncode==0
component=json.loads((o/'component-gates/software-gates.json').read_text());assert sum(x['passed'] for x in component['commands'])==30
env=os.environ|{'CARGO_TARGET_DIR':'/Users/markik/Code/repos/mere/target','CARGO_BUILD_BUILD_DIR':'/Users/markik/Code/repos/mere/target/build','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUSTUP_TOOLCHAIN':'1.99.1','CMAKE_BUILD_PARALLEL_LEVEL':'2','PYTHONUTF8':'1'};results=[];cargo='/Users/markik/.cargo/bin/cargo'
def gate(name,command):
 (o/'current-step').write_text(name);start=time.monotonic()
 with(o/(name+'.log')).open('wb') as log:cp=subprocess.run(command,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT)
 text=(o/(name+'.log')).read_text(errors='replace');matches=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);row={'name':name,'command':command,'exit':cp.returncode,'seconds':time.monotonic()-start,'passed':5 if name=='tool-tests' else sum(int(x[0]) for x in matches),'ignored':sum(int(x[2]) for x in matches),'source':source};results.append(row);(o/'software-gates.json').write_text(json.dumps({'source':source,'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_BUILD_DIR','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUSTUP_TOOLCHAIN']},'commands':results,'reused_component_gates':component},indent=2)+'\n');print(json.dumps(row),flush=True);assert cp.returncode==0,name
 if name=='tool-tests':assert 'Ran 5 tests' in text
gate('locked-metadata-fetch',[cargo,'+1.99.1','metadata','--locked','--format-version','1']);gate('tool-tests',[sys.executable,'scripts/test_dramatis_auth_tools.py']);gate('resident-tests',[cargo,'+1.99.1','test','--locked','--offline','-j2','-p','djinn','--test','passphrase_change','--message-format=json','--','--test-threads=1']);(o/'host-build.jsonl').write_bytes((o/'resident-tests.log').read_bytes());gate('app-tests',[cargo,'+1.99.1','test','--locked','--offline','-j2','-p','dramatis-app','--','--test-threads=1']);gate('boundary',[sys.executable,'scripts/check_port_boundaries.py']);gate('app-final-build',[cargo,'+1.99.1','build','--locked','--offline','-j2','-p','dramatis-app','--bin','dramatis-ui']);assert not git('status','--porcelain');gate('freeze',[sys.executable,'support/dramatis_auth_freeze.py','--repo',str(r),'--artifacts',env['CARGO_TARGET_DIR'],'--build-dir',env['CARGO_BUILD_BUILD_DIR'],'--host-build',str(o/'host-build.jsonl'),'--output',str(o/'frozen')]);(o/'current-step').write_text('software-complete')