from pathlib import Path
import hashlib,json,subprocess,shutil,plistlib,shlex,os
r=Path('/Users/markik/Code/worktrees/mere-graphshell-coop');o=Path('/Users/markik/Code/output/dauth21');f=o/'frozen';f.mkdir(mode=0o700)
def run(*a,cwd=r):return subprocess.check_output(a,cwd=cwd,text=True).strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
app=f/'Dramatis.app';exe=app/'Contents/MacOS/dramatis-ui';exe.parent.mkdir(parents=True)
shutil.copy2('/Users/markik/Code/output/graphshell-coop-relay-20261020/artifacts/debug/dramatis-ui',exe)
with (app/'Contents/Info.plist').open('wb') as h:plistlib.dump({'CFBundleIdentifier':'made.merely.dramatis.authentication.acceptance','CFBundleName':'Dramatis authentication test','CFBundleExecutable':'dramatis-ui','CFBundlePackageType':'APPL','NSHighResolutionCapable':True},h)
artifacts=[json.loads(l) for l in (o/'host-build.jsonl').read_text().splitlines() if l.startswith('{')]
host=next(x['executable'] for x in artifacts if x.get('reason')=='compiler-artifact' and x['target']['name']=='passphrase_change' and x.get('executable'))
shutil.copy2(host,f/'authentication-host')
run('/usr/bin/codesign','--force','--sign','-','--timestamp=none',str(app))
run('/usr/bin/codesign','--force','--sign','-','--timestamp=none','--identifier','made.merely.authentication.test-host',str(f/'authentication-host'))
for p in [app,f/'authentication-host']:run('/usr/bin/codesign','--verify','--strict',str(p))
info=json.loads(run(str(exe),'--build-info'));names=['Cargo.lock','Cargo.toml','ports/dramatis/Cargo.toml','ports/dramatis/build.rs']+[str(p.relative_to(r)) for p in (r/'ports/dramatis/src').glob('*.rs')];h=hashlib.sha256()
for n in sorted(names):h.update(n.encode()+b'\0'+(r/n).read_text().replace('\r\n','\n').encode()+b'\0')
assert h.hexdigest()==info['source_sha256'],(info,h.hexdigest())
# App top-level depfile includes transitive sources. Test-host artifacts each
# carry rustc depfiles; collect their first rule without phony target rules.
deps=[Path('/Users/markik/Code/output/graphshell-coop-relay-20261020/artifacts/debug/dramatis-ui.d')]
for a in artifacts:
 if a.get('reason')!='compiler-artifact':continue
 for s in a['filenames']:
  p=Path(s);stem=p.stem.removeprefix('lib');dp=Path('/Users/markik/Code/repos/mere/target/build/debug/deps')/(stem+'.d')
  if dp.is_file():deps.append(dp)
files=set(names+['support/dramatis_auth_receipts.py','ports/djinn/tests/passphrase_change.rs'])
for dp in deps:
 for name in shlex.split(dp.read_text().split(': ',1)[1].split('\n\n',1)[0].replace('\\\n','')):
  p=Path(name);p=p if p.is_absolute() else r/p
  if p.is_relative_to(r) and p.is_file():files.add(str(p.relative_to(r)))
metadata=json.loads(run('/Users/markik/.cargo/bin/cargo','+1.99.1','metadata','--locked','--offline','--format-version','1'));suppliers={};externals=[]
for p in metadata['packages']:
 if p['source'] and p['source'].startswith('git+'):
  parent=Path(p['manifest_path']).parent;root=run('git','rev-parse','--show-toplevel',cwd=parent)
  if root not in suppliers:
   head=run('git','rev-parse','HEAD',cwd=root);assert head==p['source'].split('#')[-1]
   assert not run('git','diff','--name-only','HEAD','--','*.rs','*.toml','Cargo.lock','*.wgsl','*.c','*.h','*.cpp','*.hpp',cwd=root)
   suppliers[root]={'source':p['source'],'head':head,'packages':[],'tracked_code_changes':[]}
  suppliers[root]['packages'].append(p['name'])
 elif not p['source'] and not Path(p['manifest_path']).is_relative_to(r):externals.append({'name':p['name'],'manifest':p['manifest_path']})
for p in ['dramatis-app','djinn']:
 tree=run('/Users/markik/.cargo/bin/cargo','+1.99.1','tree','--locked','--offline','-p',p,'--edges','normal,build');(o/(p+'-tree.txt')).write_text(tree+'\n')
 for e in externals:assert str(Path(e['manifest']).parent) not in tree,e
value={'source_commit':run('git','rev-parse','HEAD'),'build_info':info,'files':{n:sha(r/n) for n in sorted(files)},'binaries':{str(p.relative_to(f)):{'sha256':sha(p),'bytes':p.stat().st_size,'architecture':run('/usr/bin/lipo','-archs',str(p)),'signing':'ad-hoc; timestamp none'} for p in [exe,f/'authentication-host']},'git_suppliers':suppliers,'uncompiled_external_path_packages':externals,'rustc':run('/Users/markik/.cargo/bin/rustc','+1.99.1','-Vv'),'supplier_scope':'Resolved Git checkouts, tracked Rust/TOML/lock/WGSL/C/C++ only; untracked and asset-only supplier changes excluded','depfiles_read':len(set(deps))}
(o/'build-inputs.json').write_text(json.dumps(value,indent=2)+'\n');print(json.dumps({'source':value['source_commit'],'files':len(files),'git_suppliers':len(suppliers),'binaries':value['binaries']}))
