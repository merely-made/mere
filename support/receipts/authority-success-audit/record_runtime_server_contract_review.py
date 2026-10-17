"""Finish the entire Server contract, without crediting unread implementation helpers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='runtime_server_contract_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('runtime_client_review_receipt.json')
assert git('rev-parse','HEAD')==old['basis_commit'] and git('rev-parse','origin/main')==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(Path(p))==h for p,h in old['external_context_sha256'].items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');assert load('pending_read.json')==[]
path='support/patches/cubecl-runtime/src/server/base.rs'
assert sha(ROOT/path)==i[path]['sha256']==c[path]['sha256'] and c[path]['through']==i[path]['chars']==82411
assert old['read_through'][path]==22239;delta=60172
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q431' and authority['entries'][-1]['id']=='A1327'
rows=[
 ('Error classification recursively separates refusals from poisoned devices','Several is refusal only for a nonempty all-refusal group; device poison is any poisoned root.',
  'Entire server/base.rs now read, superseding the earlier partial-read qualification. Unwritten walks to root in both predicates; IO StorageMappingFailed also walks poison source. Tests explicitly cover expansion panic, fault versus refusal, mixed and empty groups, but are source-read only. Generic compilation refusal remains a caller classification followup, not disproved by those targeted controls. IoError.may_succeed_after_reclaim excludes only BufferTooBig/PageUpdateForbidden and therefore includes poison/unsupported IO; trace caller poison ordering and retry policy before declaring a bad retry.'),
 ('Shared collective contract names an unfinished cross-device failure migration','Send refusal preserves source trust but cannot recall the posted peer recv.',
  'ServerCommunication defaults explicitly return NoDeviceTransport before touching buffers; capable implementations owe input claim check and destination taint. Client already refuses missing transport and host-stages cross-runtime copies. The known send/recv liveness gap is explicitly documented: two actors post independently, failed source send leaves peer receive waiting. This is a real remaining design boundary, not a hidden successful transfer. No live collective/device deadlock witness was run.'),
 ('Kernel argument IO has one shared fallback chain','Compiled visibility wins, caller declaration follows, absent data conservatively reads and writes.',
  'KernelArguments.buffers_read/buffers_written share io_attr for buffers and tensor-map backing bindings. Mixing declared/undeclared resources keeps declaration indices aligned; tests check this and partial compiled visibility. A write-only output is not checked as an input, permitting a good write to repair a previously tainted output. The contract delegates actual taint store to cubecl-server. No production failure recovery or visibility tests executed here.'),
 ('Graph lifecycle contracts separate unsupported capture and cheap abandon from backend guarantees','prepare defaults harmlessly; begin/end/replay unsupported; graph destroy and profile abandon are backend responsibilities.',
  'Default graph_prepare Ok is not proof graph support: begin/end/replay refuse. Default graph_destroy is harmless for unknown/unsupported id. Destroy must order after in-flight replay; graph memory/page liveness and concrete recordability belong to implementation. Default abandon_profile calls end_profile and discards timing, explicitly expensive; supported backends are asked to override. Stream_ids default derives actor-thread current stream, so single-stream implementations still need consistent canonical reporting; no backend mismatch established.'),
 ('Dispatch spreading is a helper rather than full shape admission','CubeCountSelection labels exact/approximate and CubeDim estimates hardware occupancy.',
  'cube_count_spread reduces x then y by powers of2 and can overdispatch, relying on kernel bounds checks. It does not explicitly refuse requested total beyond x*y*z capacity or validate maxZ after spreading; hardware/property and launcher admission require followup. IoError distinctions separate permanent BufferTooBig from temporary OutOfMemory, deferred StorageMappingFailed, graph PageUpdateForbidden and PageSizesExhausted. No numerical bound/counterexample or GPU allocation test was run in this contract-only extension.')]
ids=[]
for number,(domain,claim,assessment) in enumerate(rows,1328):
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':[path],'assessment':assessment,'state':'entire Server contract/error/argument/dispatch/test source read; no additional execution','verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==3233 and stats['read_chars']==36716459 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':path,'files_completed':1,'chars_newly_read':delta,'total_file_chars':82411,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'Entire file completed after previous22,239-character partial. No new test/backend execution.'})
next_read={'path':'support/patches/cubecl-runtime/src/server/capture.rs','start':0,'reason':'Read remaining capture/handle/storage/throughput/tuning implementation source, then remaining supplier/support/workflows; keep full-source objective.'}
pending='No audit-owned process running.3233/3738 source files complete;505 remain, including31 incomplete runtime files. Entire runtime Server contract now read. All3738 live source membership/hashes unchanged; no new execution, production edit, binary or isolation.'
notes['current_area']='Runtime Server contract complete; capture/storage/throughput/autotune pending';notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False});notes['checkpoint']['new_receipts'].append(NAME)
addition=f'''

### Runtime Server contract completion, 2026-10-17

The entire82,411-character server/base.rs is now read, adding60,172 characters beyond the prior partial. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters;505 files remain incomplete. All3,738 live source membership/hashes remain unchanged. A1328-A1332 record recursive refusal/poison classification, the shared IO visibility fallback chain, graph lifecycle defaults and the explicitly unfinished collective failure path. A refused send can leave its posted peer receive waiting; the contract names that cross-device liveness gap instead of claiming successful propagation. IO reclaim classification and dispatch capacity admission remain source followups. Existing argument/error/dispatch tests were read, not executed.

This extends the prior checkpoint; Q429-Q431 and their11 scoped CPU cases retain their original qualifications. Prior report bytes, protected shared source and index are preserved. No additional binary/process/isolation or production edit; evidence stays below100MB. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}). Continue server/capture.rs; full-runtime and full-repository completion remain unproven.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':'runtime_client_review_receipt.json','sha256':sha(HERE/'runtime_client_review_receipt.json')},'coverage':stats,'source_sha256':{path:sha(ROOT/path)},'preserved_production_and_test_hashes':guards,'read_through':{path:c[path]['through']},'new_findings':[],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{**old['evidence_sha256'],'record_runtime_server_contract_review.py':sha(Path(__file__))},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only. Entire82,411-char Server contract read,60,172 new chars. No new execution. Previous11 CPU cases retain original scopes. All3738 live source membership/hashes unchanged; no production edits or isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
