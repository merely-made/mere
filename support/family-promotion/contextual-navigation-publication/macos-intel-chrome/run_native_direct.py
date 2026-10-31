import argparse,hashlib,json,os,shutil,subprocess,tempfile,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--prefix',required=True);p.add_argument('--scenario',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--env',action='append',default=[]);p.add_argument('--timeout',type=float,default=60);p.add_argument('args',nargs=argparse.REMAINDER);o=p.parse_args()
out=o.output.resolve();out.mkdir(parents=True,exist_ok=False);receipt=out/'scenario.done'
env=os.environ.copy();env.update(dict(x.split('=',1) for x in o.env));env.update({o.prefix+'_SCENARIO':str(o.scenario.resolve()),o.prefix+'_CAPTURE_DIR':str(out),o.prefix+'_RECEIPT':str(receipt)})
t=time.monotonic();timed=False
with tempfile.TemporaryDirectory(prefix='context-chrome-native-') as scratch:
 binary=Path(scratch)/'product';shutil.copy2(o.binary,binary);digest=hashlib.sha256(binary.read_bytes()).hexdigest()
 with (out/'native.log').open('w') as log:
  child=subprocess.Popen([str(binary),*(o.args[1:] if o.args[:1]==['--'] else o.args)],env=env,stdout=log,stderr=subprocess.STDOUT)
  try:code=child.wait(timeout=o.timeout)
  except subprocess.TimeoutExpired:
   timed=True;child.terminate()
   try:code=child.wait(timeout=5)
   except subprocess.TimeoutExpired:child.kill();code=child.wait(timeout=5)
result=receipt.read_text() if receipt.exists() else ''
success=not timed and code==0 and result.splitlines()[:1]==['RESULT ok']
j={'launch_method':'direct native executable; same product/scenario/renderer','launcher_exit':code,'timeout':timed,'success':success,'elapsed_seconds':round(time.monotonic()-t,3),'binary_sha256':digest,'scenario_sha256':hashlib.sha256(o.scenario.read_bytes()).hexdigest(),'receipt':receipt.name if receipt.exists() else None,'captures':len(list(out.glob('*.png')))}
(out/'process.json').write_text(json.dumps(j,indent=2)+'\n');print(json.dumps(j),flush=True);print(result,flush=True);raise SystemExit(0 if success else 1)
