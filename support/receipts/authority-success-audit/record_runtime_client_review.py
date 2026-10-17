"""Checkpoint runtime client/config/guard/report source, with remaining runtime explicit."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='runtime_client_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('remote_transport_review_receipt.json')
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
paths=[p for p in i if p.startswith('support/patches/cubecl-runtime/') and c.get(p,{}).get('through',0)>0]
complete=[p for p in paths if c[p]['through']==i[p]['chars']];partial=[p for p in paths if p not in complete]
assert len(complete)==39 and partial==['support/patches/cubecl-runtime/src/server/base.rs'] and c[partial[0]]['through']==22239
assert sum(c[p]['through'] for p in paths)==287578
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] for p in paths)
probe=load('runtime_scope_probe_receipt.json');assert probe['exit']==0 and probe['passed']==11
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert all(not Path(p['path']).exists() for p in probe['removed_outputs'])
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q428' and authority['entries'][-1]['id']=='A1315'
new=[
 ('Q429','Selected runtime allocation window ignores a pinned client stream and skips reset on unwind',
  'support/patches/cubecl-runtime/src/client.rs:allocation_window,memory_persistent_allocation,memory_dedicated_allocation',
  'Most client operations resolve self.stream_id(), which honors set_stream. allocation_window instead takes StreamId::current() and submits the mode transition there. The exact function with immediate device/server/stream doubles changes stream1 while a client pinned to2 continues observing Auto for its allocation stream. The ordinary unpinned control routes Persistent and resets Auto. If the caller callback unwinds, the exact function never submits its closing Auto transition; the double retains Dedicated and records only one transition. The public safe wrappers promise to bracket the task in a mode and then restore it.',
  'Resolve the same client stream as allocation calls and use a scope guard to balance the transition on unwind. Define nested/previous-mode restoration explicitly rather than silently promising it while always posting Auto. Keep backend state lifetime on its actor; no production change in this audit.',
  'Three CPU cases compile the unchanged allocation_window function with explicit immediate Device/Server and StreamId doubles. They establish routing and missing reset commands, not an actual GPU allocation, leak, FIFO runner execution or backend nested-mode semantics.'),
 ('Q430','Selected runtime RealRun guard can move across threads and corrupt both threads dry-run state',
  'support/patches/cubecl-runtime/src/dry_run.rs:RealRun,real_run::DEPTH',
  'RealRun contains only unit state and is implicitly Send; new increments the creating thread TLS depth and Drop decrements the dropping thread with saturating_sub. Full unchanged dry_run source compiled with std atomics allows std::thread::spawn to move it. After foreign drop the origin still has depth1 and executes during an active DryRun. A second case gives the destination its own live RealRun, then drops the foreign guard there: destination depth becomes0 and skips launches despite its own guard still alive. Four original nesting/default/dry-run control tests pass.',
  'Make a TLS-owning guard unable to cross thread boundaries and ensure all tuners/exclusive callbacks construct it on the launch-issuing thread. Add compile-fail or trait assertions for transferability and retain original nesting tests. Do not constrain the global DryRun guard for this TLS-specific issue.',
  'Two additional real std-thread cases plus four unchanged dry_run tests. Only serial_test test attributes were removed; the test runner serializes execution. No fake TLS, CubeCL lowering, autotune/backend/GPU run or actual shipped cache contamination. Probe manually balances leaked test state afterward to isolate cases.'),
 ('Q431','Launch observations dropped in documented taken order lose an active observer then resurrect an ended observer',
  'support/patches/cubecl-runtime/src/logging/observer.rs:LaunchObservation,new,Drop,notify_launch',
  'Documentation says guards dropped in the order they were taken leave the process as found. The exact replacement/restore implementation actually needs strict reverse order. Install A then B, drop A first: its previous=None clears B while B scope is active, so B receives no launch. Drop B next: its saved A is restored even though A scope ended; after both guards have dropped, A receives another launch and is_observing remains true. The reverse-order control restores A while alive and clears it when finished.',
  'Either explicitly enforce strict nesting and correct the contract, or track live observation registrations so independently overlapping lifetimes cannot restore ended observers. A process-wide observer needs a deliberate overlap owner. Atomically couple the observing fast flag to slot changes under the same ownership decision. Keep source-only concurrent flag/callback timing concerns separate from this sequential witness.',
  'Two CPU cases compile unchanged guard/new/drop/is_observing/notify_launch code with a minimal observer trait and std RwLock adapter. No actual kernel/profile, concurrent install race, logging backend or GPU was run. Sequential documented-order failure and reverse-order control are the executed claims; the adapter does not prove every cubecl_environment lock behavior.')]
for ident,title,source,evidence_text,repair,scope in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence_text,'status':'reproduced unchanged selected-runtime control flow with exact source and stated doubles; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':['runtime_scope_probe_receipt.json']})
P='support/patches/cubecl-runtime/'
rows=[
 ('Runtime API and implementation have an explicit extraction boundary','Client/Server/value contracts live here; pools, streams, drivers and compiler pipeline belong to cubecl-server.',
  [P+'src/lib.rs',P+'Cargo.toml',P+'Cargo.toml.orig',P+'build.rs',P+'src/runtime.rs'],
  'Read39 complete files plus22,239 chars of server/base.rs; the remaining server/throughput/tuner source is not complete. Root selects this0.11.0 patch and says its source is pristine upstream with persistence-default manifest change. Selection/version and current bytes are verified; no independent upstream source archive comparison was available, so pristine is not asserted as an audit result or falsely contradicted by upstream-looking changes. Build cfgs centralize filesystem, persistence, serde, exclusive-memory-only and multithreading. Environment contracts are re-exported rather than locally reimplemented.'),
 ('Erased client still validates concrete service and allocation ownership','Client stamps allocations with full ServiceId and checks every resource crossing back into the actor.',
  [P+'src/client.rs',P+'src/server/base.rs'],
  'local/expect_local guard reads/writes/checks and launch resources, including indirect count and tensor map binding. get_resource checks requested concrete server type before downcast. Graph handle owns actor/id/stream; last clone submits graph destroy. Safe captures return a graph whose replay is explicitly unsafe because buffers/stream use stay caller-owned. Zero static launch grid is a deliberate no-op. These are real ownership checks, not a second mutable active-device cursor.'),
 ('Graph enqueue, buffer claims and device barriers are distinct success domains','replay returns actor enqueue acceptance; sync reports barrier/device failure; check/sync_buffers report named buffer failures.',
  [P+'src/client.rs',P+'src/compiler.rs',P+'src/server/base.rs'],
  'Client docs do not promote enqueue acceptance into GPU completion. Unwritten work carries errors on buffers and device poison remains sticky. Expansion panic is converted to CompilationError::ExpansionPanicked and explicitly not treated as refusal. Compiler generic/validation errors otherwise count as refusals; actual harness/tuner call sites still need review before deciding whether programming defects can be skipped. Server error algebra source is only read through the start of ServerError::is_refusal here; do not claim later code read.'),
 ('Allocation mode window has unfinished stream/lifetime migration','Pinned client operations use self.stream_id but allocation_window uses ambient StreamId.current.',
  [P+'src/client.rs',P+'src/memory_management/policy.rs'],
  'Q429 qualifies wrong-stream transitions and missing close on callback unwind. Public unsafe allocation_mode uses the pinned stream correctly, so separate wrappers currently disagree. Safe wrappers say restore previous mode but close by posting Auto without a client-side mode stack; nested semantics must be established against the server owner. Profile bracket similarly has an unguarded token across func() and manual start/end explicitly require abandonment by caller; token leak on unwind is a source followup, not a newly executed profiler failure.'),
 ('Dry-run policy has one issuance authority but its TLS guard is transferable','Global DryRun depth composes across threads; local RealRun opts measurements back into execution.',
  [P+'src/dry_run.rs',P+'src/client.rs'],
  'LaunchMode is decided on issuing thread and passed to backend, preserving caller context across the actor queue. Dry-run buffer reads are explicitly meaningless and tuning must construct RealRun inside actual issue/exclusive callback. Q430 proves Send guard crosses TLS ownership. Four original CPU tests pass but cover only same-thread nesting, so they do not establish cross-thread safety. no_std RealRun is a no-op and not evaluated by the std probe.'),
 ('Observation is a global projection with replacement lifetimes','One observer slot attributes issued launches; a separate profiling logger aggregates asynchronously.',
  [P+'src/logging/observer.rs',P+'src/logging/server.rs',P+'src/client.rs'],
  'Q431 proves documented-order replacement/restore failure. Callback contract explicitly forbids launch/install/drop under the held read lock, so reentrant deadlock is not reported as an admitted callback use. Launch observation explicitly reports issuance, includes dry-run and capture, and omits per-kernel graph replay. Timing delivery identity is checked across a blocking read, but identity is sampled after initial notify; replacement between launch and measurement can still misattribute, a source-only followup. OBSERVING is set outside slot lock; concurrent flag/slot race also remains unexecuted.'),
 ('Timing absence, timing method and observer fallback are preserved','Missing measurement stays None; observer-only refused profile must not drop the computation.',
  [P+'src/client.rs',P+'src/logging/observer.rs',P+'src/logging/server.rs',P+'src/logging/profiling.rs'],
  'Launch arguments remain in a slot until profile callback takes them; observer-only refusal before callback reissues once, after callback does not duplicate computation. Logger opt-in timing still fails loudly. Deferred observer measurements become resolved when logger also consumes; warning explains lost overlap. Method travels to observer; profiler aggregation table sums durations by name without method separation, so mixed system/device timing is a projection limitation. Absent timing prints not measured and does not enter totals. Full observer test source read but not run; only extracted guard cases ran.'),
 ('Configuration and logging reuse shared environment state but freeze settings at different lifetimes','RuntimeConfig applies environment/stream/record setup; Logger snapshots an Arc while some operations read config again.',
  [P+'src/config/',P+'src/logging/server.rs'],
  'Config source fully read. Decisions have separate sinks and recorder alias; bench clamps samples/factor, adaptive-only knobs explicitly say ignored on wasm. Environment/cache/logger/StreamPolicy definitions delegate shared environment owner. Memory config rejects unknown pool entries; other config structs often accept unknown fields. Some logger methods say current global settings while reading stored Arc snapshot. Exact RuntimeConfig mutation/freeze semantics live outside this source; no config-change bug claimed. Disabled logger still constructs shared sink config; active server log queue is unbounded and errors best-effort.'),
 ('Device identity preserves backend choice with finite ID budgets','Wgpu uses kind type plus13-bit adapter index and3-bit graphics API; externally registered IDs use16bits.',
  [P+'src/device/',P+'src/runtime.rs'],
  'Ordinary oversized indices clamp to an absent high index rather than wrap. Existing Metal/Wgpu IDs areU32 but to_id castsU16; registry constructor limits need tracing before a reachable alias finding. Public Wgpu fields can name backend on Existing despite .on refusing to change it, an equality/canonicalization followup. from_id unknown type defaults to default WgpuDevice. CPU ignores deviceId, CUDA/HIP preserve compact index. Original device roundtrip tests source-read, not run.'),
 ('Metadata/region layout has one declared allocation owner','MemoryLayoutPolicy.apply promises layouts share one binding; create/empty_tensors delegate that policy.',
  [P+'src/client.rs',P+'src/server/base.rs',P+'src/memory_management/handle.rs',P+'src/memory_management/guard.rs'],
  'This resolves the retained Burn quantization followup at the API-contract level: one allocation backs value/scales/global offsets, though backend implementation witnesses remain separate. Lazy controller holds binding, checks host-copy policy, materializes once and explicitly reflects first-access rather than construction-time state. Device subview requires contiguous row-major strides. Managed descriptor uses one packed atomic location, binding keeps allocation, weak binding only observes; pool implementation is external. PageGuard distinguishes page freeze and allocation binding lifetime.'),
 ('Reports centralize pool/stream totals without proving allocator behavior','MemoryReport sums stream dynamic/persistent/dedicated and auxiliary pools.',
  [P+'src/memory_management/base.rs',P+'src/memory_management/policy.rs',P+'src/client.rs'],
  'Memory scope and dedicated/persistent sentinel definitions are shared. Report tests cover auxiliary and totals in source only. Zero reservation/in-use percentages can printNaN; byte formatting uses decimal powers with a sufficiently wide prefix forU64. High-water marks include tuner scratch unless warmed beforehand, as documented. Reporting a single place to read memory does not verify all external backend allocator fields are accurate.'),
 ('Kernel source/cache identity belongs to the kernel author','PrecompiledSource contract explicitly says every cache keys the supplied KernelId and never source text.',
  [P+'src/id.rs',P+'src/kernel.rs','support/patches/burn-cubecl/src/template/base.rs'],
  'Selected runtime contract narrows A1296: retained SourceKernel forwards its source owner id and does not insert template text itself, despite its define comment claiming text rides kernel name for invalidation. Caller may correctly hash its text into id, so no actual stale compiled artifact is claimed. Runtime deliberately delegates source identity to author, and language mismatch to compiler tag. KernelId stable format/hash incorporates type name/address/dim/mode/info. Process uniqueness and finite counters are separate from durable cache identity; overflow catch-and-continue wrap remains a theoretical source followup.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1316):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'runtime client/config/guard/report source complete, server partial; execution scopes explicit','verification':NAME})
notes['review_followups'].extend([
 {'area':'Runtime scope ownership','detail':'Q429 pinned/ambient allocation stream and unwind reset; Q430 transferable TLS RealRun guard; Q431 observer restoration in documented taken order. Qualify nested allocation/profile unwind cleanup, active observer identity between notification and measurement, and atomic flag/slot updates. Do not present unexecuted concurrency/backend effects as proved.','verification':NAME},
 {'area':'Runtime migration and cache authority','detail':'Runtime contracts separate enqueue/buffer trust/barrier/device poison and author-owned source-cache identity. Trace remaining server/tuner/refusal classification and backend implementation separately. Shared MemoryLayoutPolicy contract resolves combined quantized allocation intention, not every backend implementation. Root pristine-upstream comment still needs an independent archive comparison if used as provenance evidence.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3232 and stats['read_chars']==36656287 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'support/patches/cubecl-runtime initial client/config/guard/report slice','files':39,'chars_newly_read':287578,'partial_read':{p:c[p]['through'] for p in partial},'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'39 complete source files plus22,239 chars of server/base.rs.11 scoped CPU cases,4 unchanged dry-run controls. No full runtime/backend/GPU build. Remaining32 runtime files incomplete.'})
next_read={'path':partial[0],'start':22239,'reason':'Finish full runtime Server contract/error/layout/capture/storage/throughput/autotune source, then remaining supplier/support/workflows; preserve original denominator.'}
pending='No audit-owned process running.11 scoped CPU cases pass.3232/3738 source files read;506 remain, including32 incomplete runtime files. All3738 live source membership/hashes unchanged. No production edit, owned generated binary, isolation or backend/GPU run.'
notes['current_area']='Runtime client/config/guard/report complete; remaining Server/tuner source pending';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'runtime_scope_probe_receipt.json'])
addition=f'''

### Selected CubeCL runtime client, configuration and scope review, 2026-10-17

This slice reads39 complete runtime files and22,239 characters of server/base.rs:287,578 characters total. It covers manifests/build cfg, the complete client/lazy controller, compiler/kernel/id/runtime contracts, all configuration/devices, dry-run/launch collection/observer/profiling, memory handles/guards/reports/policy and poison. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters.506 source files remain incomplete, including32 runtime files. All3,738 live source hashes and membership are unchanged.

Q429: allocation_window uses ambient StreamId.current while normal client allocations honor a pinned stream. Exact-function CPU doubles show mode transitions on stream1 while the pinned client's stream2 stays Auto; an unwinding callback also skips the closing transition. Q430: RealRun is implicitly Send despite owning TLS depth. Actual std-thread tests show a foreign drop leaves the origin executing inside DryRun, and can disable the destination's own still-live guard. Q431: observer documentation promises guards dropped in taken order restore the process, but exact code loses active B when A drops first, then resurrects ended A when B drops. Reverse-order control behaves as intended. Eleven CPU cases pass, including4 unchanged dry-run controls. Allocation and observer cases use explicit device/stream/lock/trait doubles; there is no actual GPU allocation, profiling run or full runtime build.

A1316-A1327 track real boundaries alongside gaps: full ServiceId checks survive client erasure; graph enqueue acceptance, named-buffer trust and device completion are explicitly different. Dry-run verdict is issued once before actor handoff. Observers report issuance and omit graph replay; missing timing stays absent, observer-only profile refusal preserves computation, and logger timing can change overlap/method semantics. Configuration/reporting delegate shared environment and layout contracts, while mode/TLS/observer lifetimes remain unfinished seams. MemoryLayoutPolicy promises a single binding across combined layouts, resolving the retained quantization intention at contract level. Kernel source identity is explicitly author-owned: retained template-wrapper invalidation prose cannot substitute for an id hashing its text. Root's pristine-upstream claim was not independently archive-compared and is not promoted into audit provenance.

Prior report bytes, protected source/shared work and index are preserved. Two owned executable/PDB outputs were verified and removed; no audit process or isolation remains. Evidence stays below100MB. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [scope probes](../../../support/receipts/authority-success-audit/runtime_scope_probe_receipt.json). Continue server/base.rs at character22,239; the full runtime/repository review remains open.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
evidence=['record_runtime_client_review.py','runtime_scope_probe_receipt.json',*probe['evidence_sha256']]
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':'remote_transport_review_receipt.json','sha256':sha(HERE/'remote_transport_review_receipt.json')},'coverage':stats,'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[v[0] for v in new],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{p:sha(HERE/p) for p in evidence},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only.39 runtime files complete plus22,239-char Server partial;287578chars.11 scoped CPU cases. Two owned exe/PDB outputs removed. No full runtime/backend/GPU build or production edit. All3738 source membership/hashes unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
