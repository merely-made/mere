"""Run only AST-extracted receipt helpers with explicit process/source doubles."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,os,types
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pieces={}
def function(path,name):
 text=(ROOT/path).read_text(encoding='utf-8-sig');tree=ast.parse(text)
 node=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name==name)
 pieces[path+':'+name]=ast.get_source_segment(text,node)
 return compile(ast.Module(body=[node],type_ignores=[]),str(ROOT/path),'exec')
package='support/dramatis_package_receipts.py';review='support/dramatis_review_receipts.py'
records=[];return_code=0
def fake_process(command,**kwargs):
 kwargs['stdout'].write('running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 1 filtered out\n')
 return types.SimpleNamespace(returncode=return_code)
scope={'OUT':HERE,'ROOT':ROOT,'records':records,'subprocess':types.SimpleNamespace(run=fake_process,STDOUT=-2)}
exec(function(package,'run'),scope)
scope['run']('support_driver_zero_tests',['cargo','test','--ignored','--exact','real_signed_package_and_tamper_controls'])
assert records[-1]['exit_code']==0
return_code=9
try:scope['run']('support_driver_nonzero_control',['audit-process-double'])
except AssertionError:pass
else:raise AssertionError('nonzero process must be rejected')
class MemorySource:
 def __init__(self):self.bytes=b'// audit-only synthetic source\nif !self.synthetic_denial {}\n';self.writes=[]
 def read_bytes(self):return self.bytes
 def write_bytes(self,value):self.bytes=value;self.writes.append(value)
class MemoryRoot:
 def __init__(self,source):self.source=source
 def __truediv__(self,path):assert path=='ports/djinn/src/keeper/review.rs';return self.source
cases=[]
for concurrent in [False,True]:
 source=MemorySource();original=source.bytes;other=b'// synthetic concurrent owner edit\n';calls=[]
 def fake_run(name,*args,**kwargs):
  calls.append(name)
  if name=='negative-consent':
   if concurrent:source.bytes=other
   return 'cards_are_secret_free_scoped_and_denial_preserves_the_item ... FAILED\n'
  return ''
 scope={'ROOT':MemoryRoot(source),'cargo':lambda *args:list(args),'os':os,'run':fake_run}
 exec(function(review,'consent_control'),scope);returned=scope['consent_control']()
 assert returned==original and source.bytes==original
 assert len(source.writes)==2 and b'DRAMATIS_SYNTHETIC_CONSENT_CONTROL' in source.writes[0]
 cases.append({'concurrent_edit_injected':concurrent,'concurrent_bytes_preserved':source.bytes==other,'saved_original_restored':source.bytes==original,'write_count':len(source.writes),'calls':calls})
data={'zero_test_process_accepted':records[0],'nonzero_process_control':records[1],'consent_control_cases':cases}
(HERE/'support_driver_cases.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'passed':4,'source_sha256':{p:sha(ROOT/p) for p in [package,review]},
 'extracted_function_sha256':{k:hashlib.sha256(v.encode()).hexdigest() for k,v in pieces.items()},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_support_driver_probe.py','support_driver_cases.json','support_driver_zero_tests.log','support_driver_nonzero_control.log']},
 'qualification':'Only original AST-extracted run and consent_control functions evaluated. Process double emits a successful zero-test log and nonzero control; source double exists entirely in memory and simulates a concurrent edit during the negative-control call. Four cases. No actual cargo, signed installer, source mutation, consent backdoor in product, Djinn, native window, account, credential or existing receipt operated. No binary/isolation/live process retained.'}
with (HERE/'support_driver_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'passed':4,'zero_tests_accepted':True,'synthetic_concurrent_edit_overwritten':cases[1]['saved_original_restored']}))
