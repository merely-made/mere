"""Checkpoint complete OPFS probe source reading and bounded verifier tests."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='opfs_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('gazette_moot_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256'])
runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=old.get('external_context_sha256',{})
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files'];coverage=load('read_coverage.json')
paths=[p for p in inventory if p.startswith('ports/muniment-opfs-probe/')]
assert len(paths)==19 and sum(inventory[p]['chars'] for p in paths)==250796
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():
    assert h==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars']
assert load('pending_read.json')==[]
native=load('opfs_owner_probe_receipt.json');browser=load('opfs_browser_probe_receipt.json');locked=load('opfs_locked_metadata_receipt.json')
assert native['exit']==0 and native['passed']==19 and native['original_tests']==17
assert all(sha(ROOT/p)==h for p,h in native['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in native['evidence_sha256'].items())
assert len(native['removed_outputs'])==4 and all(not Path(item['path']).exists() for item in native['removed_outputs'])
assert all(sha(ROOT/p)==h for p,h in browser['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in browser['fixture_sha256'].items())
assert len(browser['runner_cases'])==6 and browser['page_cases']['storage_states'][2]['result']['state']=='persistent'
assert browser['page_cases']['run_all'][1]['lane1']['ok'] is False and browser['page_cases']['run_all'][1]['state']=='complete'
assert browser['runner_cases'][4]['lane5_ok'] is True and browser['runner_cases'][4]['portability']['round_trip_both_ways_ok'] is False
assert locked['exit']==101 and '--locked' in locked['stderr'] and sha(ROOT/'ports/muniment-opfs-probe/Cargo.lock')==locked['lock_sha256_before_after']
for p in ['ports/muniment-opfs-probe/README.md','ports/muniment-opfs-probe/Cargo.lock','crates/eidetic/muniment/Cargo.toml']:
    aux[p]=sha(ROOT/p)
original_lock=tomllib.loads((ROOT/'ports/muniment-opfs-probe/Cargo.lock').read_text())
original_muniment=[p['version'] for p in original_lock['package'] if p['name']=='muniment']
current_muniment=tomllib.loads((ROOT/'crates/eidetic/muniment/Cargo.toml').read_text())['package']['version']
assert original_muniment==['0.1.1'] and current_muniment=='0.1.2'
# The live standard-source membership scan was also performed in this pass:
# no additions; only the three previously explicit diff/extension-manifest extras.
# Recheck every inventoried content hash here before preserving read credit.
assert all((ROOT/p).is_file() and sha(ROOT/p)==v['sha256'] for p,v in inventory.items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q399' and authority['entries'][-1]['id']=='A1195'
new=[
 ('Q400','Fatal-write half cases repeat full writes in the OPFS cut sweep','ports/muniment-opfs-probe/src/fault.rs:223,493; web/probe.js:321-323,907-909',
  'Actual FaultBackend over SharedMemory receives an eight-byte write. Native sweep policies [0,usize::MAX/2,usize::MAX] and browser policies [0,2^31,u32::MAX] land [0,8,8] bytes. Explicit torn_bytes=4 lands four bytes. min(len) interprets the intended half sentinel as a huge absolute byte count.',
  'Represent none/half/all explicitly or derive half from the actual failing buffer length. Existing short_write_at really writes half but returns WriteZero without declaring the backend dead; it is not the missing fatal half-write cut. Original cut/error sweeps pass for the cases they actually execute, not the advertised three distinct fatal shapes.','opfs_owner_probe_receipt.json'),
 ('Q401','OPFS overall completion and runner success omit failed lane predicates','ports/muniment-opfs-probe/web/probe.js:1261-1280; run-browser.mjs:93-123,177-190',
  'Whole unchanged page source with injected lane boundaries sets data-probe-state=complete after lane1 returns ok=false; the false lane remains visible in JSON. Whole unchanged CLI exits zero for false lanes, thrown lane errors and false page-error gate. A native verifier exit1/ok=false updates round_trip_both_ways_ok=false but leaves lane5.ok and runner per-lane ok=true.',
  'Compute explicit required-lane, page-gate and native-verification acceptance after all stages, update every aggregate and use a distinct failed/unsupported outcome plus appropriate CLI status. Preserve detailed JSON. Lane throwing does correctly reject page runAll; positive controls complete. This is verifier/UI aggregation, not a claim that real browser storage failed.','opfs_browser_probe_receipt.json'),
 ('Q402','OPFS content oracle hashes a missing listed value as an empty value','ports/muniment-opfs-probe/src/workload.rs:140-152',
  'Actual digest helper returns identical hash and key count for real MemoryBackend containing ghost with an empty byte value and an explicitly defective Backend listing ghost but returning None. unwrap_or_default substitutes missing data before hashing, so the alleged complete key/value content oracle does not detect the inconsistency.',
  'Refuse a listed key that cannot be read, retaining the empty-value distinction and diagnostic key. This counterexample injects a bad backend; no actual redb/IndexedDB loss or historical digest mismatch is asserted. Multi-call snapshot consistency remains a separate host/benchmark lifetime assumption.','opfs_owner_probe_receipt.json'),
 ('Q403','Rejected persistence query is reported as persistent storage','ports/muniment-opfs-probe/web/probe.js:180-200',
  'Whole unchanged page source executes storageSnapshot with explicit navigator boundary. persisted() true yields persistent, false yields best_effort, and rejection also yields persistent with no error field. withTimeout catches rejection into {__error:...}; the caller then treats that object as truthy.',
  'Recognize typed query errors and return unknown/error with diagnostic rather than treating them as a granted boolean. Keep timeout and true/false controls distinct. No real browser permission request or change to persistence state was made.','opfs_browser_probe_receipt.json'),
 ('Q404','Committed OPFS probe lock no longer resolves under its advertised locked runner','ports/muniment-opfs-probe/Cargo.lock; run-probe.ps1:77-80,130-138',
  'Real offline cargo metadata --locked against the original standalone manifest/config from neutral C:/t refuses with exit101 because the lock needs updating. Original lock still records path Muniment0.1.1 while the actual local path manifest is0.1.2. Both bytes remain unchanged. Audit-only native manifest can compile current source after resolving its own copied lock.',
  'Refresh/review the standalone lock with current local dependency versions and requalify named gates before claiming current reproducibility. The separately passing current-source CPU suite does not repair or validate the original locked runner. No production lock changed and historical pinned receipts remain historical evidence.','opfs_locked_metadata_receipt.json')]
for ident,title,source,evidence,repair,diagnostic in new:
    notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence,
      'status':'reproduced current source/CLI behavior within recorded boundaries; documented, not repaired',
      'scope':'Native redb/MemoryBackend CPU tests, Node VM and whole CLI copies with explicit injected browser boundaries, or real offline locked metadata as named. No real browser/OPFS/IndexedDB/worker kill/power-loss execution.',
      'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':[diagnostic]})
rows=[
 ('Probe storage adapters are research implementations alongside production','Standalone redb4.2 and IndexedDBRangeBackend deliberately coexist with Muniment redb2 and its shipping IndexedDB adapter.',
  ['ports/muniment-opfs-probe/Cargo.toml','ports/muniment-opfs-probe/src/redb_backend.rs','ports/muniment-opfs-probe/src/indexeddb_range.rs','crates/eidetic/muniment/src/backend.rs'],
  'The split isolates incompatible redb majors and a fair range-query baseline. These copies are not current production migrations. In particular neither probe adapter implements current transact; typed default refusal preserves safety but they are not complete drop-in production replacements. Q404 shows independent lock drift despite source-level native compilation.'),
 ('OPFS token registry keeps browser handles in their owning realm','OpfsBackend holds plain data tokens while worker-local registry owns sync-access handles.',
  ['ports/muniment-opfs-probe/src/opfs_backend.rs'],
  'Source has explicit realm mismatch refusal for storage and close, no unsafe Send/Sync assertion, JS safe-integer guards and short read/write checks. Handle registry slots are never reused; close on a foreign thread reports retained ownership rather than success. No browser/atomics/threaded close execution here. Registry lifetime/tombstone growth remains probe-local followup.'),
 ('Range baseline states and enforces narrower key admission','ASCII contract is checked at every key-bearing operation and whole apply batch before writes.',
  ['ports/muniment-opfs-probe/src/idb_keys.rs','ports/muniment-opfs-probe/src/indexeddb_range.rs','ports/muniment-opfs-probe/src/worker.rs'],
  'Five real native arithmetic/order tests pass. Source includes browser positive ASCII write/get/scan and exact final-key controls, so refusing everything cannot pass. Production IndexedDB full-list/filter behavior and restricted indexed range baseline must remain separately named. Browser ASCII/transaction event paths not executed here.'),
 ('Recovery model distinguishes failure kinds but misses a fatal torn shape','Shared churn invariant underlies native and browser fault/cut trials.',
  ['ports/muniment-opfs-probe/src/churn.rs','ports/muniment-opfs-probe/src/fault.rs','ports/muniment-opfs-probe/web/probe.js'],
  'Native originals actually complete464+500 cut trials and error sweeps. Q400 means nominal half and all fatal-write rows duplicate full landing. Short-write error still exercises real half bytes with a different model. Worker termination is explicitly not power-loss/flush durability. Generation verifier checks current values, next generation and stale range; arbitrary distant future keys and concurrent snapshots remain bounded source followups.'),
 ('Same Backend workload is real reuse, content oracle has a missing-value gap','Small slots, logs, atomic batches and blobs share workload code across all backends.',
  ['ports/muniment-opfs-probe/src/workload.rs','ports/muniment-opfs-probe/src/redb_backend.rs'],
  'Actual original five-workload memory/redb digest equality passes. Shipping LogBatched versus deliberately unbatched OrderedLog is explicitly distinguished. Q402 weakens missing-value detection only; no observed supplier corruption. Browser timing orders, medians, paired identical-write controls and worst-observed envelopes are honestly qualified, not fresh performance measurements.'),
 ('Overall outcome needs one acceptance authority after every stage','Page lane results, conclusions, receipt gate, native verifier and CLI status each own partial success flags.',
  ['ports/muniment-opfs-probe/web/probe.js','ports/muniment-opfs-probe/run-browser.mjs','ports/muniment-opfs-probe/src/bin/fixture.rs'],
  'Q401 shows divergent aggregate states. The native fixture itself enforces integrity/generation/digest/key count and exits1 for false,2 for errors, so its failure is stronger than the JS aggregate consuming it. Native fixture source read but original portability binary not run. JSON retains failures; zero runner status is not acceptance.'),
 ('Staging and promotion have separate authority and evidence','Worker creates staging file and publishes only completed generation; move and copy fallback record different semantics.',
  ['ports/muniment-opfs-probe/src/opfs_backend.rs','ports/muniment-opfs-probe/src/worker.rs','ports/muniment-opfs-probe/web/probe.js','ports/muniment-opfs-probe/README.md'],
  'Source and docs explicitly say promotion-boundary kills cannot prove a kill inside move or spec crash atomicity. Copy/delete fallback is recorded non-atomic. Actual-counter-derived staging sweep is stronger than guessed indices, but Q400 affects fatal byte coverage. Stable-name observations and full generation checks remain qualified; no browser staging trial rerun.'),
 ('Persistence capability reporting must preserve errors as errors','StorageManager estimate/persisted/persist responses feed receipt provenance separately from OPFS bytes.',
  ['ports/muniment-opfs-probe/web/probe.js'],
  'Q403 proves rejected persisted query becomes persistent through truthiness. Timeout path records unknown, explicit true/false controls work. Persist request can also retain an error object in granted; do not mistake that for permission. Actual browser grant, quota durability or eviction resistance was not measured.'),
 ('Probe build and receipt tooling owns mutable outputs, not product state','Build runner regenerates fixture/package, receipt sink writes selected names and browser runner exports bytes and receipts.',
  ['ports/muniment-opfs-probe/run-probe.ps1','ports/muniment-opfs-probe/run-browser.mjs','ports/muniment-opfs-probe/receipt-sink.py','ports/muniment-opfs-probe/web/worker.js'],
  'Tools were read, not launched against cited output directories. Legacy target defaults and evidence overwrite paths need caller isolation/current workspace hygiene. Build source/lock hashes and served bundle hashes are useful provenance but do not complete result acceptance. Q404 real metadata refusal precedes original build gates. No server/sink or real browser launched.'),
 ('Worker and cross-tab lifetimes remain probe-owned followups','One worker/command boundary and BroadcastChannel holder protocol coordinate storage ownership.',
  ['ports/muniment-opfs-probe/web/probe.js','ports/muniment-opfs-probe/web/worker.js'],
  'holdUntil remembers worker failure but does not settle its holding promise without a holding state, leaving a hang path. Worker timeout, holder path correlation, restored receipt/build consistency and read_all short-read diagnostics remain source-only followups. Current complete source reading is not a fresh headed multi-tab/reload acceptance run.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1196):
    ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'all probe source read; actual CPU and injected JS/CLI scope recorded separately; full audit ongoing','verification':NAME})
for area,detail in [
 ('OPFS adapter convergence','Research redb4 and range IndexedDB copies still differ from production, including missing transact override and ASCII admission. Do not promote passing native old-method workloads into full modern Backend adoption.'),
 ('OPFS probe lifetime and provenance','Investigate holdUntil rejection settlement, per-command deadlines, BroadcastChannel holder path matching and sessionStorage receipt/build binding before a revived browser gate. Larger coordinated repair remains documented; none was attempted.'),
 ('OPFS measurement ownership','Retain shipping-adapter versus indexed-range and unbatched versus real batched log distinctions. Quote exact engine/host and worst observed envelopes; original native CPU timings are not browser performance acceptance.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2827 and stats['read_chars']==33019887 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'ports/muniment-opfs-probe/','files':19,'chars':250796,'verification':NAME,'basis_commit':head,'qualification':'Every inventoried probe source/manifest/style/script read.19 native CPU tests pass (17 original2 audit),12 explicit injected JS cases and real --locked metadata refusal. No real browser/OPFS/IndexedDB/kills/power-loss/portability execution.'})
next_read={'path':'ports/pelt','start':0,'reason':'Continue remaining Pelt, Signalman, Tabard desktop, scripts and supplier patch source without narrowing inventory.'}
pending='No audit-owned process running.19 native CPU tests pass,12 explicit injected JS cases complete, original locked metadata refuses. Four exact owned exe/PDB outputs removed. Entire3738 source hash inventory matches current live bytes, no new standard source; three explicit diff/extension-manifest inclusions retained. Main7c718105/origin00e44428 unchanged after fetch; eight reviewed upstream commits still uninstalled due shared report overlap. Full audit incomplete.'
notes['current_area']='Complete OPFS probe source and verifier review; continue Pelt'
notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending})
notes['checkpoint']['new_receipts'].extend([NAME,'opfs_owner_probe_receipt.json','opfs_browser_probe_receipt.json','opfs_locked_metadata_receipt.json'])
addition=f'''

### OPFS probe source and acceptance ownership review, 2026-10-17

All 19 OPFS probe source/manifest/style/script files are read (250,796 characters). Overall coverage: {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. Every inventoried file was rehashed against current live bytes: all 3,738 match, no new standard source was found, and the three explicit diff/extension-manifest inclusions remain. No stale or pending read credit. Fetch confirms unchanged main/upstream {head}/{upstream}; the eight reviewed upstream commits remain uninstalled because of the shared report overlap. Full audit is not complete.

Q400: fatal cut policies advertised as none/half/all actually land none/all/all because a huge absolute torn_bytes count is clamped to buffer length. Native and browser policies both reproduce [0,8,8] on an eight-byte write; explicit4 is a real half. Original short-write error injection still does half bytes, with different post-error behavior. Q401: overall page state becomes complete after a lane returns false, and the CLI returns zero for false/thrown lanes or a failed page-error gate. Native verifier exit1/false produces round_trip_both_ways_ok=false while lane5.ok and runner lane status stay true. JSON retains those failures; aggregate success must not conceal them. Q402: digest equates a listed-but-missing key with a present empty value through unwrap_or_default. Q403: rejected persisted() query becomes persistent through truthiness of the caught error object. Q404: real offline --locked metadata for the original probe refuses with exit101; its lock records local Muniment0.1.1 while current source is0.1.2. Passing a separately resolved audit CPU graph does not fix that original runner.

Actual unchanged native root passed all17 original unit tests plus2 audit cases. The originals include464+500 cut trials, error injection, generation invariants, ASCII range arithmetic, real redb backend operations and all five shared workload digest comparisons against MemoryBackend. This verifies the cases executed, with Q400 limiting the claimed fatal-byte coverage. Four exact completed-run exe/PDB files were marker-checked and removed. The JavaScript evidence executes whole unchanged page source in Node VM and six unchanged runner CLI processes: three storage booleans/errors, three overall-run controls, six CLI/native-verifier outcomes. DOM, browser lanes and native child outputs are explicit fixtures; no real browser, OPFS/IndexedDB, worker termination, promotion, power loss, portability binary, server or network acceptance is claimed. Original source, original lock and historical receipts remain unchanged.

A1196-A1205 keep research copies and migration ownership visible. The redb4 adapter and ASCII-only range IndexedDB baseline are intentionally isolated from production redb2/shipping IndexedDB; neither overrides current transact, whose typed refusal prevents accidental racy fallback but also prevents full modern-backend substitution. OPFS token ownership stays worker-local with realm checks, including failure to release on the wrong thread; this is stronger than an unsafe thread assertion, not fresh threaded browser proof. Shared workloads are real reuse, but result flags, native verification and process status still lack one complete acceptance predicate. Staging isolation and atomic promotion remain separate; documentation already disclaims proof of a kill inside move. Performance comparison correctly distinguishes shipping versus indexed range and deliberately unbatched versus shipping batched log paths. Hold-until rejection settlement, command deadlines, holder correlation, restored receipt/build identity and source snapshot assumptions remain documented followups.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [native CPU tests](../../../support/receipts/authority-success-audit/opfs_owner_probe_receipt.json), [injected JS cases](../../../support/receipts/authority-success-audit/opfs_browser_probe_receipt.json), [original locked metadata](../../../support/receipts/authority-success-audit/opfs_locked_metadata_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. No production edits or new target/home/worktree. Continue remaining port and supplier source.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_opfs_review.py','opfs_owner_probe_receipt.json','opfs_browser_probe.py','opfs_browser_probe_receipt.json','opfs_locked_metadata.log','opfs_locked_metadata_receipt.json',*native['evidence_sha256'],*browser['fixture_sha256']]
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'gazette_moot_review_receipt.json','sha256':sha(HERE/'gazette_moot_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[r[0] for r in new],'authority_entries':ids,'all_opfs_probe_inventory_source_read':True,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'live_inventory_revalidation':{'all_file_sha256_match':True,'files':len(inventory),'new_standard_source':[],'explicit_inclusions':['design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json']},
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress, not full repository completion.19 source files read;19 native CPU tests,12 explicit injected JS cases, real original locked metadata refusal. Four run-owned binaries removed. No real browser storage/worker/power-loss acceptance, production edits or new isolation.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
