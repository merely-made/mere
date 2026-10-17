import pathlib,shutil,subprocess,re,json,hashlib
base=pathlib.Path('/Users/markik/Code/testing/cleromancy/natal-cap-20261017')
home=base/'narrow-cargo-home'
home.mkdir(exist_ok=False)
original=pathlib.Path('/Users/markik/.cargo')
(home/'git/db').mkdir(parents=True)
for db in (original/'git/db').iterdir():
 if not db.is_dir():continue
 dest=home/'git/db'/db.name
 shutil.copytree(db,dest,ignore=shutil.ignore_patterns('objects','*.lock'))
 (dest/'objects/info').mkdir(parents=True)
 (dest/'objects/pack').mkdir()
 (dest/'objects/info/alternates').write_text(str(db/'objects')+'\n')
(home/'git/checkouts').mkdir()
for co in (original/'git/checkouts').iterdir():
 if not co.is_dir():continue
 dest=home/'git/checkouts'/co.name
 dest.mkdir()
 for child in co.iterdir():
  (dest/child.name).symlink_to(child,target_is_directory=child.is_dir())
(home/'registry').symlink_to(original/'registry',target_is_directory=True)
lock=pathlib.Path('Cargo.lock')
shutil.copyfile(lock,base/'narrow-Cargo.lock.before')
inputs={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in pathlib.Path('.').rglob('Cargo.toml') if '.git' not in p.parts and 'target' not in p.parts}
for p in [pathlib.Path('.cargo/config.toml'),pathlib.Path('ports/djinn/src/resident_mere.rs'),*pathlib.Path('ports/djinn/tests/fixtures').rglob('*')]:
 if p.is_file():inputs[str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
(base/'narrow-input-hashes.json').write_text(json.dumps(inputs,indent=2)+'\n')
branches={}
for url,branch,rev in re.findall(r'source = "git\+([^"?]+)\?branch=([^"#]+)#([0-9a-f]+)"',lock.read_text()):branches.setdefault((url,branch),set()).add(rev)
for (url,branch),revs in branches.items():
 name=url.rstrip('/').split('/')[-1].removesuffix('.git')
 rev=next(iter(revs)) if len(revs)==1 else '8960024f7b1f2b7a5ba382972f08b2f75eb7a0f0'
 if name=='cleromancy':rev='da495cfc75b2ee6bbb9476c8c03386e9057c36d3'
 for db in (home/'git/db').glob(name+'-*'):
  ok=subprocess.run(['git',f'--git-dir={db}','cat-file','-e',rev],capture_output=True).returncode==0
  if ok:
   subprocess.run(['git',f'--git-dir={db}','update-ref','refs/remotes/origin/'+branch,rev],check=True)
   print(db.name,branch,rev)
print('CARGO_HOME='+str(home))
