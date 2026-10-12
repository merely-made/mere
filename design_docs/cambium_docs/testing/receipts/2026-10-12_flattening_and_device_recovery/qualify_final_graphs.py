import pathlib, subprocess, json, re,sys,time
out=pathlib.Path(__file__).parent;root=pathlib.Path('/Users/markik/Code/worktrees/mere-vello-release-adoption')
expected={'genet':'2c7ef73bfa44c07744e0643e3c6f98be3cbd7947','netrender':'c74b7cb28090c28b1c6d80b08dcdea18aff48d3e','vello':'2eae6243881d5b24c984d5ee35dde509ca5db782','knot-editor':sys.argv[1]}
files=['Cargo.toml','ports/distillery/probe/Cargo.toml','ports/distillery/probe/remote-fixture/Cargo.toml','ports/graphshell/web/Cargo.toml','crates/cambium/examples/genet_web_smoke/Cargo.toml'];summary=[]
for i,file in enumerate(files):
 start=time.monotonic();p=subprocess.run(['cargo','metadata','--offline','--locked','--manifest-path',file,'--format-version','1'],cwd=root,capture_output=True,text=True,timeout=150)
 label='mere-final-graph-'+str(i);(out/(label+'.log')).write_text(p.stderr)
 if p.returncode:print(p.stderr[-2500:]);raise SystemExit(p.returncode)
 m=json.loads(p.stdout);(out/(label+'.json')).write_text(json.dumps(m));rows=[];names={}
 for pkg in m['packages']:
  source=pkg.get('source') or ''
  if source.startswith('git+'):
   url=source[4:].split('?',1)[0].removesuffix('.git');repo=url.rsplit('/',1)[-1]
   if repo in expected:
    rev=source.split('#')[-1];assert rev==expected[repo],(file,pkg['name'],source,expected[repo])
    rows.append({'name':pkg['name'],'version':pkg['version'],'source':source})
    names.setdefault(pkg['name'],set()).add(source)
 for name in names:
  all_sources={pkg.get('source') for pkg in m['packages'] if pkg['name']==name}
  assert all_sources==names[name],(file,name,all_sources,names[name])
 assert all(len(v)==1 for v in names.values()),(file,names)
 # Embedded Knot must not leave a second immutable Mere contract source.
 if i==0:
  old=[x for x in m['packages'] if x.get('source') and 'github.com/merely-made/mere.git' in x['source']];assert not old,[(x['name'],x['source']) for x in old]
 row={'manifest':file,'elapsed_seconds':round(time.monotonic()-start,3),'resolved_packages':len(m['packages']),'suppliers':rows,'accepted':True};summary.append(row)
 (out/'final-supplier-qualification.json').write_text(json.dumps({'expected':expected,'roots':summary},indent=2)+'\n');print(file,'accepted',len(rows),'supplier packages',flush=True)
