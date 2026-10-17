"""Checkpoint capture/storage/throughput and initial tuning-plan source."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='runtime_throughput_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load('runtime_server_contract_review_receipt.json')
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
paths=[p for p in i if p.startswith('support/patches/cubecl-runtime/') and c.get(p,{}).get('through',0)>0 and p not in guards]
assert len(paths)==25 and sum(i[p]['chars'] for p in paths)==219281
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
probe_names=['runtime_throughput_probe_receipt.json','runtime_checksum_probe_receipt.json'];evidence=['record_runtime_throughput_review.py']
for name,passed in zip(probe_names,[17,3]):
 probe=load(name);assert probe['exit']==0 and probe['passed']==passed
 assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
 assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
 assert all(not Path(p['path']).exists() for p in probe['removed_outputs'])
 evidence.extend([name,*probe['evidence_sha256']])
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q431' and authority['entries'][-1]['id']=='A1332'
new=[
 ('Q432','Shared roofline helpers admit invalid peaks despite their documented refusal policy',
  'support/patches/cubecl-runtime/src/throughput/roofline.rs:ResourceBound::time_at_peak,score_resources,binding_achieved',
  'time_at_peak documents None for negative rate, but checks only is_normal. A negative normal peak(-4) with amount8 reaches Duration::from_secs_f64(-2) and panics. A tiny positive normal finite peak1e-100 also panics because the derived duration is unrepresentable. The scoring path has a different validity rule: score_resources produces fraction-2 for a negative peak and0 for an infinite peak; binding_achieved only checks fraction.is_finite and selects both, though its docs say those peaks are skipped. Zero-peak scoring control is refused. Eight unchanged original roofline tests pass but include neither negative nor duration-overflow rate cases.',
  'Centralize validity and representable-duration conversion for resource bounds; reject nonpositive/nonfinite peaks and handle an unrepresentable duration without panic. Make scoring expose invalid model entries consistently so binding selection cannot name them. Keep over-100-percent valid scores visible as documented. AutotuneBound already rejects negative thresholds, but delegates the peak check to this flawed shared helper.',
  'Full unchanged roofline.rs compiled with8 original tests and5 added numeric cases. No GPU/model measurement, autotune decision, product configuration or persisted result was exercised. Demonstrates public numeric helper panic/invalid binding admission, not ordinary measured hardware producing a negative rate.',probe_names[0]),
 ('Q433','Throughput ranking narrows iteration count before normalizing duration',
  'support/patches/cubecl-runtime/src/throughput/benchmarker.rs:rank,retarget',
  'rank accepts usize iteration counts and min_iterations, then divides fastest Duration by iterations as u32. On a64-bit host a floor of2^32 with a20ms sample reaches a zero divisor and panics. Separately retarget can grow a1000-iteration1ns observation to20,000,000,000 iterations, beyond warmup ceilings; a deterministic20s sample then normalizes through wrapped2,820,130,816 and reports7ns rather than1ns per iteration. Ordinary1000ns/iteration and zero-minimum controls behave as intended.',
  'Keep iteration normalization in a representation that can name the requested count and bound retargeting consistently with the probe/callback contract. Reject or safely normalize counts outside a backend-supported loop range; do not truncate silently. Preserve the minimum-pass constraint while preventing division by zero or reporting a different per-iteration duration.',
  'Four CPU cases compile exact rank/retarget bodies and target duration with shape-equivalent config/result structs and deterministic duration callbacks. No actual large iteration workload, hardware timing, current concrete probe-caller reachability, cache insertion or GPU/backend build. The result is a helper count/normalization defect on64-bit, not an observed current GPU benchmark error.',probe_names[0]),
 ('Q434','Autotune candidate-list checksum loses name boundaries and candidate count',
  'support/patches/cubecl-runtime/src/tune/operation.rs:TunableSet::compute_checksum',
  'compute_checksum concatenates candidate names into one String before StableHasher. Exact unchanged accumulation prefix gives the identical preimage abc for[ab,c] and[a,bc], as well as for the one-candidate list[abc]. Both names and list arity can change without changing its claimed invalidation key. Distinct concatenation control differs. Equal input necessarily gives the same deterministic hasher output; this is an encoding collision before hashing, not a probabilistic hash collision.',
  'Hash the structured ordered name sequence, including boundaries/count, rather than its concatenation. Require names/identities to cover configurable candidate semantics. Trace cache-load/index admission before deciding migration/version invalidation; a source checksum by itself cannot establish a winner still names the same operation.',
  'Three CPU cases compile exact accumulation prefix with minimal name-bearing candidate/set structs and inspect the String preimage. No replacement hasher was used; StableHasher crate itself was not built/run. No actual persistent-cache load, stale index execution, tuner/GPU run or current persistence-feature selection proved. Remaining TuneCache/Tuner code is explicitly unread at this checkpoint.',probe_names[1])]
for ident,title,source,evidence_text,repair,scope,probe in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence_text,'status':'reproduced selected-runtime numeric/encoding behavior with exact source and stated doubles; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':[probe]})
P='support/patches/cubecl-runtime/src/'
rows=[
 ('Capture phase and published status share one transition owner','DeviceCaptures publishes StreamCaptureState transitions and gives clients read-only CaptureStatus.',
  [P+'server/capture.rs'],
  'The logical NoCapture/Prepare/Capture state and atomic summaries are centralized and recomputed under one record lock; rejected transitions do not change it, Drop removes it. Foreign end deliberately abandons rather than seals and can combine an earlier doomed reason. Pooled stream neighbours share physical costs. Backend open/close work still must complete or call abort; publication alone is not proof the driver agrees. begin has no caller parameter, so same-logical-owner requirement relies on caller admission until end verifies ownership. Complete transition test source read, not run.'),
 ('Region ownership is finer than allocation identity','BufferBinding.claim_key names managed allocation plus exact start/end byte range.',
  [P+'server/handle.rs',P+'storage/base.rs'],
  'This prevents sibling regions in one batched allocation swallowing each others taint claims. Full ServiceId, originating stream and offsets travel with strong/weak bindings. StorageId selects native storage while managed id selects pool reservation, so the two IDs are not redundant authorities. Offset arithmetic is publicly mutable and unchecked for underflow/overflow here; downstream bounds admission still needs tracing. BufferBinding rustdoc references removed Server.initialize_bindings, an obsolete API link rather than an implemented initialization path.'),
 ('Native resource lifetime is retained by its binding, not raw resource size','ManagedResource retains ManagedMemoryBinding while exposing a potentially larger native resource.',
  [P+'storage/base.rs',P+'server/handle.rs'],
  'Only the requested region is guaranteed live; other portions may be reused, explicitly documented. ComputeStorage owns allocation/deallocation/flush and bytes_allocated includes pending frees. No concrete storage implementation exists in this crate to prove those guarantees for a device. Resource-mut access is a backend-native seam, not a replacement allocator authority.'),
 ('Throughput reports a measured ceiling with explicit work/access identity','MemoryAccess separates read/write/copy; ThroughputKey includes working-set size and probe version.',
  [P+'throughput/base.rs',P+'throughput/benchmarker.rs',P+'throughput/cmma.rs'],
  'Work count/duration are explicit; zero timing reports NaN/N/A rather than instantaneous speed. Public callbacks can still supply ZERO to a cacheable Ok result, so validity is not enforced at the cache boundary. Warmup/plateau/sample budgets and min-pass semantics are explicit; rank is a brief ordering measurement, not peak acceptance, and Q433 finds its wide-count hole. Tile selection shares feature authority and exact operand/accumulator match. Concrete kernels live elsewhere, so no read/write eviction/hardware ceiling claim was executed.'),
 ('Throughput cache keys hardware part and capacity rather than device ordinal','Runtime/fingerprint/name/capacity/parallelism identify reusable peaks; physical ordinal is excluded intentionally.',
  [P+'throughput/cache.rs'],
  'Names are path-sanitized; unlike exact sequence encoding this mapping can collapse different text labels to one namespace and remains an identity followup. Persistent cache adds probe generation, purges only earlier generations in this crate scope and keeps existing conflicting measurements; in-memory insertion overwrites. Store epoch/environment rebinding is external and not yet inspected, so process-global cache keyed only by device is a lifetime followup, not proved stale-environment reuse. Persistence is feature-gated despite the broad stores-on-disk-when-std comment.'),
 ('Memory curve is a projection of qualified measurements, not generic bandwidth truth','Finite positive points feed octave interpolation and endpoint clamping.',
  [P+'throughput/curve.rs'],
  'Invalid/empty points are dropped and sweep uses checked doubling. The code labels log2 interpolation but deliberately uses piecewise linear mantissa approximation between octaves. Duplicate points are promised first-wins after sort_unstable, which does not guarantee stable equal-key order; unexecuted followup. Rates are neither enforced monotonic nor verified cache-cold here; flat saturation beyond the end is an assumed model. ResourceBound/roofline share arithmetic with bounds but Q432 shows validity policy is incomplete.'),
 ('Bound threshold validation and resource-peak validation diverge','AutotuneBound rejects nonpositive/non-normal thresholds then delegates ResourceBound time.',
  [P+'tune/bounds_generator.rs',P+'throughput/roofline.rs'],
  'The threshold code and its negative-threshold regression test show the exact guard missing from peak validity in Q432. Bounds compose max resource time plus launch overhead, and Full uses unbounded0.0. Tiny positive threshold and Duration addition can still overflow; no executed bounds overflow case here. Ad-hoc Minimal/Balanced/Extensive fractions are identified as such. Bitwise float Eq preserves reflexivity. Shared Work alias is re-exported from cubecl-common, not duplicated arithmetic payload.'),
 ('Tune groups separate stable internal identity from user-facing names','GroupId, not name, governs cross-level dedup and per-group patience.',
  [P+'tune/base.rs',P+'tune/patience.rs'],
  'One TunePlan prices priorities once, interleaves ordered/cutoff batches and returns each candidate once with all admitting groups. Negative priorities skip; ungrouped trail first batch; exhausted groups fall through. Distinct groups sharing a name remain distinct. Existing test commentary still describes older Arc-name dedup, but actual code and same-name regression test use IDs. Group counter is finiteU32; no overflow exercise. Plan/patience tests fully read but not executed. Q434 concerns candidate-list encoding, a different identity seam.'),
 ('Local tuner set ownership is device-scoped while winner state is only id-scoped','Initializer type plus device ID keys built sets; per-ID tuner state returns an index into the caller set.',
  [P+'tune/local.rs',P+'tune/operation.rs'],
  'This fixes first-device capability capture at the set layer. fastest_identity explicitly requires the matching set, since another initializer under the same id maps the same winner index differently. Set initialization runs callback under write lock; reentrant same-tuner init remains a liveness followup. clear removes winner/tuner state but leaves built sets. Fast cache hits execute without a new checksum request; persistent load/epoch/settled validation is in remaining unread Tuner/TuneCache, so no stale-winner execution is yet asserted.'),
 ('Eviction, generated benchmark inputs and real inputs retain separate ownership','TuneInputs generators clone/synthesize while Eviction receives reference call inputs and is a measurement aid.',
  [P+'tune/input_generator.rs',P+'tune/key_generator.rs',P+'tune/eviction.rs',P+'tune/operation.rs'],
  'Shared HRTB traits support owned/borrowed families without local data copies as authority. Default cloning does not itself guarantee a fresh output allocation; caller generator owns aliasing. Eviction must issue on sample stream and cover cache capacity; failure logs and samples warm by documented policy. Therefore an eviction failure can change measurement meaning while tune still answers. Actual scheduling/error classification is not reviewed until the remaining driver source.'),
 ('Tune logs, decisions and environment records are distinct projections','Human/JSON decision sinks and persistent TuneRecord report different evidence.',
  [P+'tune/log.rs',P+'tune/record.rs',P+'tune/local.rs'],
  'JSON decision requires a measured first result; all-failed unmeasured winner is explicitly not logged as a measurement, but Schedule behavior remains unread next. TuneRecord includes stored boolean, plan/priorities, wall/trials/short circuit/dry-run and abandons a span when environment changes. Non-std_io decision sink emits unavailable error. autotune-checks dispatch is attached through optional log context, so disabling logger/decision context can bypass that diagnostic path; trace remaining check helper before claiming a promised mandatory check. These are records of decisions, not direct GPU execution receipts.'),
 ('Adaptive sample evidence is distinct from one-shot fastest observation','SampleSet tracks count/convergence and ages first sample only after three samples.',
  [P+'tune/sampler.rs',P+'tune/patience.rs'],
  'Short-circuit counts independently under-limit samples; best/reliable sample use differs deliberately. Stalled count resets when first sample ages out; improvement threshold2percent and convergence two rounds. Patience skips a candidate only if every admitting group exhausted, and waits at least one measurement. Original source tests are fully read, not run; driver must still apply these policies and no valid-result fallback is qualified until Schedule/Tuner read.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1333):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'capture/storage/throughput/initial-plan source complete; remaining tuner drivers explicitly pending','verification':NAME})
notes['review_followups'].extend([
 {'area':'Throughput validity and checksum admission','detail':'Q432 inconsistent peak validity and duration conversion; Q433 wide-count ranking; Q434 ambiguous candidate-name sequence encoding. Follow cache admission and concrete probe callers before claiming deployed stale-winner/GPU measurement failures.','verification':NAME},
 {'area':'Tuner completion versus measurement','detail':'Next read Schedule all-failed unmeasured winner path, TuneCache/Tuner checksum/index/environment checks, autotune-checks optional logging context and final candidate retry/failure. Preserve difference between source policy, measurement, selected index and successful actual execution.','verification':NAME},
 {'area':'Model and resource scope followups','detail':'Capture begin caller ownership/backends abort, bound Duration overflow, curve unstable duplicate ordering/model coldness/plateau assumptions, throughput environment rebinding/key sanitization, mutable region bounds and typed-set/winner-key lifetime mismatch remain source-qualified followups.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==3258 and stats['read_chars']==36935740 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Runtime capture/storage/throughput and initial tune plan/local/log/record/sampler source','files':25,'chars_newly_read':219281,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'25 files fully read;20 CPU cases,8 original roofline controls. Remaining6 runtime files83,499 characters unread. No backend/GPU/full-runtime build or actual cache-load witness.'})
next_read={'path':'support/patches/cubecl-runtime/src/tune/schedule.rs','start':0,'reason':'Complete Schedule, tune benchmark/cache/inputs/Tuner/util source and qualify all-failed/unchecked winner paths; then remaining supplier/support/workflows.'}
pending='No audit-owned process running.3258/3738 source files complete;480 remain, including6 unread runtime files83,499 chars.20 scoped CPU cases pass. All3738 live source membership/hashes unchanged. No production edit, generated owned binary, isolation or actual GPU/cache-load run.'
notes['current_area']='Runtime capture/storage/throughput/initial tuning plan complete; final6 tuner files pending';notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False});notes['checkpoint']['new_receipts'].extend([NAME,*probe_names])
addition=f'''

### Runtime capture, storage, throughput and initial tuning-plan review, 2026-10-17

All25 files in this slice are fully read (219,281 characters). Runtime coverage is65/71 files, leaving6 driver/cache/input/helper files83,499 characters. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters;480 source files remain. Full live3,738-file membership/hashes remain unchanged.

Q432: roofline time_at_peak admits a negative normal peak and panics despite promising None; tiny positive finite peaks can overflow Duration. Separate scoring can select negative/infinite-peak entries the docs say are skipped. Q433: rank divides by iterations asU32; a64-bit2^32 count panics, and an uncapped retargeted20-billion count reports7ns instead of1ns in the deterministic callback probe. Q434: candidate names are concatenated without boundaries/count before hashing, so[ab,c], [a,bc] and[abc] share the exact checksum input. These are public helper/encoding witnesses, not an observed GPU ceiling error, deployed stale-cache execution or current concrete caller reachability claim.

Twenty CPU cases pass:8 unchanged roofline controls,5 additional invalid-rate cases,4 exact rank/retarget cases with explicit duration callbacks, and3 unchanged checksum-accumulation-prefix cases. The latter inspect identical hasher input; the StableHasher crate was not replaced or run. Four exact owned executable/PDB outputs were verified and removed. No runtime/backend/GPU build, live cache load, server or integration run occurred.

A1333-A1344 track the shared capture state and region claim key, binding-backed resource lifetime, part/capacity/probe-version throughput identity, curve/model limits, group IDs versus labels, device-scoped set building versus id-scoped winner state, eviction/input ownership and separate decision/record projections. Capture publication is a logical state receipt; backend transition success still requires completion/abort. The tuning logger explicitly describes an all-failed unmeasured-winner path, which is the next driver source to examine. Shared ResourceBound arithmetic exists, but peak and threshold validity are incomplete in different ways. Remaining source suspicions stay distinct from the reproduced helper cases.

Prior report bytes, protected production/shared source and Git index remain preserved; retained evidence is below100MB. No audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [numeric probe](../../../support/receipts/authority-success-audit/{probe_names[0]}) and [checksum probe](../../../support/receipts/authority-success-audit/{probe_names[1]}). Continue tune/schedule.rs. The full-source objective remains active and incomplete.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':'runtime_server_contract_review_receipt.json','sha256':sha(HERE/'runtime_server_contract_review_receipt.json')},'coverage':stats,'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[v[0] for v in new],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{p:sha(HERE/p) for p in dict.fromkeys(evidence)},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only.25 runtime files219281chars complete.20 scoped CPU cases with explicit numeric/callback/preimage limits. Four owned exe/PDB outputs removed. No actual GPU/probe/cache-load execution or production edit; all3738 live source membership/hashes unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
