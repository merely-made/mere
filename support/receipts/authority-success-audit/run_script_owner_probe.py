"""Run current exporter and extracted smoke classifier; never run setup/native drivers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,os,sqlite3,subprocess,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sys.dont_write_bytecode=True
p=ROOT/'scripts/firefox_history_export.py';spec=importlib.util.spec_from_file_location('export_audit',p);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
cases=[]
def match_case(name,stamps,metadata,tolerance,expected):
 c=sqlite3.connect(':memory:')
 try:
  c.executescript('create table moz_historyvisits(id integer,place_id integer,visit_date integer); create table moz_places_metadata(place_id integer,created_at integer,total_view_time integer);')
  c.executemany('insert into moz_historyvisits values(?,1,?)',[(n+1,t*1000) for n,t in enumerate(stamps)])
  c.executemany('insert into moz_places_metadata values(1,?,?)',[(t,n+1) for n,t in enumerate(metadata)])
  actual=module.visit_view_times(c,tolerance);assert len(actual)==expected,(name,actual)
  unclaimed=[{'visit_id':n+1,'timestamp':t} for n,t in enumerate(stamps) if n+1 not in actual]
  cases.append({'name':name,'visit_stamps_ms':stamps,'metadata_stamps_ms':metadata,'tolerance_ms':tolerance,'matched':actual,'unclaimed':unclaimed})
 finally:c.close()
match_case('dense_valid_fifth_match_dropped',[1000,1001,1002,1003,1004],[1002]*5,10,4)
assert cases[0]['unclaimed']==[{'visit_id':5,'timestamp':1004}]
match_case('ordinary_two_match_control',[1000,1001],[1000,1001],10,2)
match_case('out_of_tolerance_control',[1000],[2000],10,0)
(HERE/'script_export_match_cases.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
commands=[
 ('script_original_unit_tests.txt',[sys.executable,'-B','-m','unittest','test_firefox_history_export','test_mere_doc_judgment_audit','-v'],ROOT/'scripts'),
 ('script_local_source_run.txt',['pwsh','-NoProfile','-File',str(HERE/'script_owner_probe.ps1')],ROOT),
]
for log,command,cwd in commands:
 r=subprocess.run(command,cwd=cwd,text=True,capture_output=True,timeout=30)
 (HERE/log).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr);assert r.returncode==0,(log,r.returncode)
local=json.loads((HERE/'script_local_source_cases.json').read_text(encoding='utf-8-sig'));assert len(local['cases'])==4
genet=ROOT.parent/'genet/Cargo.toml';text=genet.read_text(encoding='utf-8-sig');assert 'Pelt (pelt, pelt-core, pelt-desktop) and Tabard left for mere' in text
assert not (ROOT/'scripts/remote-receipt.ps1').exists()
assert (ROOT.parent/'genet/scripts/remote-receipt.ps1').is_file()
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'export_cases_passed':3,'source_classifier_cases_passed':4,'original_unit_tests_passed':4,
 'source_sha256':{p:sha(ROOT/p) for p in ['scripts/cross-repo-smoke.ps1','scripts/firefox_history_export.py','scripts/test_firefox_history_export.py','scripts/test_mere_doc_judgment_audit.py','scripts/mere_doc_judgment_audit.py']},
 'external_context_sha256':{str(genet.resolve()):sha(genet)},'membership_observation':{'mere_remote_receipt_exists':False,'genet_remote_receipt_exists':True,'genet_manifest_pelt_extraction_comment_present':True},
 'evidence_sha256':{p:sha(HERE/p) for p in ['run_script_owner_probe.py','script_owner_probe.ps1','script_local_source_cases.json','script_export_match_cases.json',*[c[0] for c in commands]]},
 'qualification':'Three calls to unmodified current exporter using only synthetic in-memory SQLite. Four actual PowerShell Assert-Local function executions parsed from source with mocked Cargo tree output; no full cross-repo builds. Four original Python unit tests executed. Missing remote helper and Genet extraction checked read-only; no native window, SSH, account setup, browser profile, receipt driver mutation or physical input executed. No binaries, isolation or live process retained.'}
with (HERE/'script_owner_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'export_cases':3,'classifier_cases':4,'original_tests':4,'exit':0}))
