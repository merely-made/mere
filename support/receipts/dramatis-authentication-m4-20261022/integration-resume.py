from pathlib import Path
import json,os,re,shutil,stat,subprocess,sys,time
r=Path('/Users/markik/Code/worktrees/mere-authentication-m4');o=Path('/Users/markik/Code/output/dauth-m4-20261022');assert(o/'.dramatis-generated').read_text()=='synthetic authentication qualification'
def git(*args):return subprocess.check_output(['git',*args],cwd=r,text=True).strip()
assert not git('status','--porcelain');assert git('rev-parse','HEAD')=='cfbdaa88a464613c929b2b172c1c2c22866b5a27'
assert not any(str(o/'qualified-frozen') in line for line in subprocess.check_output(['ps','-axo','pid=,comm='],text=True).splitlines())
def omit_sockets(directory,names):return [n for n in names if stat.S_ISSOCK((Path(directory)/n).lstat().st_mode)]
assert json.loads((o/'native-input-evidence.json').read_text())['both_recovered_cold_reopens_passed'];shutil.copytree(o/'qualified-n',o/'cold',dirs_exist_ok=True,ignore=omit_sockets)
import hashlib
for name,info in json.loads((o/'native-input-evidence.json').read_text())['native_changed_encrypted_files'].items():assert hashlib.sha256((o/'cold'/name).read_bytes()).hexdigest()==info['sha256'],name
for fmt in ['sealed','portable']:
 assert(o/'cold'/fmt/'.authentication-fixture').read_text()=='public synthetic authentication'
 assert not(o/'cold'/fmt/'ready').exists() and not(o/'cold'/fmt/'stop').exists()
git('fetch',str(o/'integration.bundle'),'HEAD');git('merge','--ff-only','FETCH_HEAD');source=git('rev-parse','HEAD');assert source=='6238b29bc63d79a88142ca6aa81f10be32966e41'
assert subprocess.run(['git','diff','--quiet','b09af10e1..HEAD','--','ports/djinn/castellan','ports/djinn/src/identity_ui.rs'],cwd=r).returncode==0
old=json.loads((o/'qualified-software-gates.json').read_text());component={'source':old['rust_gate_source'],'commands':[],'reason':'Storage and native-provider sources/tests are byte-identical; current custody/app and headed integration run again.'};(o/'component-gates').mkdir()
for row in old['commands']:
 if row['name'] in ['storage-tests','provider-tests']:
  shutil.copy2(o/(row['name']+'.log'),o/'component-gates'/(row['name']+'.log'));component['commands'].append(row)
assert sum(x['passed'] for x in component['commands'])==30
(o/'component-gates/software-gates.json').write_text(json.dumps(component,indent=2)+'\n')
env=os.environ|{'CARGO_TARGET_DIR':'/Users/markik/Code/repos/mere/target','CARGO_BUILD_BUILD_DIR':'/Users/markik/Code/repos/mere/target/build','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUSTUP_TOOLCHAIN':'1.99.1','CMAKE_BUILD_PARALLEL_LEVEL':'2','PYTHONUTF8':'1'};results=[];cargo='/Users/markik/.cargo/bin/cargo'
def gate(name,command):
 (o/'current-step').write_text(name);start=time.monotonic()
 with(o/(name+'.log')).open('wb') as log:cp=subprocess.run(command,cwd=r,env=env,stdout=log,stderr=subprocess.STDOUT)
 text=(o/(name+'.log')).read_text(errors='replace');matches=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);row={'name':name,'command':command,'exit':cp.returncode,'seconds':time.monotonic()-start,'passed':5 if name=='tool-tests' else sum(int(x[0]) for x in matches),'ignored':sum(int(x[2]) for x in matches),'source':source};results.append(row);(o/'software-gates.json').write_text(json.dumps({'source':source,'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_BUILD_DIR','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUSTUP_TOOLCHAIN']},'commands':results,'reused_component_gates':component},indent=2)+'\n');print(json.dumps(row),flush=True);assert cp.returncode==0,name
 if name=='tool-tests':assert 'Ran 5 tests' in text
gate('locked-metadata-fetch',[cargo,'+1.99.1','metadata','--locked','--format-version','1']);gate('tool-tests',[sys.executable,'scripts/test_dramatis_auth_tools.py']);gate('resident-tests',[cargo,'+1.99.1','test','--locked','--offline','-j2','-p','djinn','--test','passphrase_change','--message-format=json','--','--test-threads=1']);(o/'host-build.jsonl').write_bytes((o/'resident-tests.log').read_bytes());gate('app-tests',[cargo,'+1.99.1','test','--locked','--offline','-j2','-p','dramatis-app','--','--test-threads=1']);gate('boundary',[sys.executable,'scripts/check_port_boundaries.py']);gate('app-final-build',[cargo,'+1.99.1','build','--locked','--offline','-j2','-p','dramatis-app','--bin','dramatis-ui']);assert not git('status','--porcelain');gate('freeze',[sys.executable,'support/dramatis_auth_freeze.py','--repo',str(r),'--artifacts',env['CARGO_TARGET_DIR'],'--build-dir',env['CARGO_BUILD_BUILD_DIR'],'--host-build',str(o/'host-build.jsonl'),'--output',str(o/'frozen')]);(o/'current-step').write_text('software-complete')
f=o/'interactive.py';s=f.read_text();assert "'s' if mode=='scripted' else 'n'" in s;s=s.replace("'s' if mode=='scripted' else 'n'", "'s' if mode=='scripted' else 'cold' if mode=='native-cold' else 'n'");f.write_text(s)
