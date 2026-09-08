from pathlib import Path
import subprocess,os,json,time,hashlib,sys
p=Path(__file__).resolve().parent
env=os.environ.copy()
env['CARGO_HOME']=str(p.parent/'arena/cargo-home')
env['CARGO_TARGET_DIR']=str(p/'target')
cmd=['cargo','test','--manifest-path',str(p/'Cargo.toml')]
if '--online' not in sys.argv: cmd+=['--offline']
if (p/'Cargo.lock').exists(): cmd+=['--locked']
cmd+=['--','--nocapture']
t=time.time()
r=subprocess.run(cmd,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(p/'run.log').write_bytes(r.stdout)
meta={'command':cmd,'exit_code':r.returncode,'elapsed_seconds':time.time()-t,'engine':'none','renderer':'none','server':'none','features':'muniment default json + redb','profile':'dev test','compiler':subprocess.check_output(['rustc','--version'],text=True).strip(),'sources':{str(f.relative_to(p)):hashlib.sha256(f.read_bytes()).hexdigest() for f in [p/'Cargo.toml',p/'Cargo.lock',p/'src/lib.rs',p/'source-manifest.json',p/'run.py'] if f.exists()}}
(p/'run.json').write_text(json.dumps(meta,indent=2),encoding='utf-8')
print(json.dumps(meta))
print(r.stdout.decode('utf-8',errors='replace')[-2800:])
