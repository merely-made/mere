"""Append-only full-runtime and retained-backend review checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='runtime_tuner_wgpu_review_receipt.json';OLD='runtime_throughput_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists();old=load(OLD)
assert git('rev-parse','HEAD')==old['basis_commit'] and git('rev-parse','origin/main')==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in {**guards,**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(Path(p))==h for p,h in old['external_context_sha256'].items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'));index=index if index.is_absolute() else ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json');assert load('pending_read.json')==[]
paths=[p for p in i if p.startswith(('support/patches/cubecl-runtime/','support/patches/cubecl-wgpu/')) and c.get(p,{}).get('through',0)>0 and p not in guards]
assert len(paths)==24 and sum(i[p]['chars'] for p in paths)==287539
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
probe_names=['runtime_tuner_probe_receipt.json','retained_wgpu_probe_receipt.json'];evidence=['record_runtime_tuner_wgpu_review.py']
for name,passed in zip(probe_names,[3,12]):
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
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q434' and authority['entries'][-1]['id']=='A1344'
new=[
 ('Q435','Autotune calls a pre-execution refusal an executed fallback and settles its index',
  'support/patches/cubecl-runtime/src/tune/schedule.rs:Schedule::run_batch,Schedule::run_plan;src/tune/tuner.rs:process_request;src/tune/tune_cache.rs:cache_insert,settled;src/client.rs:Client::exclusive',
  'Client::exclusive explicitly says its error means nothing ran. run_batch maps that error to Unknown for every candidate. run_plan treats Unknown as executed-but-unmeasured and, after all batches fail, picks the first such index. process_request inserts that decision into in-memory cache as Match and returns Hit; settled then exposes it as a settled winner before any real candidate execution. The exact-body probe refuses exclusive acquisition, proves drive was entered zero times, and observes Hit/settled index0. All-NotMeasured instead panics; a measured-score control selects its actual result index. LocalTuner then attempts the selected operation and expects success, but that execution occurs after cache insertion and does not provide evidence for the earlier executed claim.',
  'Keep exclusive-admission failure separate from executed-but-untimed output. If an unmeasured fallback is necessary, preserve its evidence state and admit/retry candidates explicitly instead of describing a never-run index as settled fastest. Keep the existing prohibition on persisting unmeasured guesses. Converge adaptive and fixed all-failed behavior under explicit failure categories rather than broad Unknown.',
  'Three CPU cases compile exact run_batch/run_plan/process_request and full tune_cache.rs under non-persistence cfg, using an explicitly refusing client and shape-equivalent surrounding types. No real contention, kernel, GPU, browser, persistent Store or full runtime build. Source confirms disk insertion is suppressed for unmeasured decisions; no disk defect asserted.',probe_names[0]),
 ('Q436','Retained Vulkan backend reads 16-bit features to advertise 8-bit storage',
  'support/patches/cubecl-wgpu/src/backend/vulkan.rs:register_types;src/backend/vulkan/features.rs:ExtendedFeatures',
  'Both storage16 and storage8 predicates read ext_feat.buf_16.uniform_and_storage_buffer16_bit_access. ExtendedFeatures separately queries/enables buf_8, but register_types never consults it for storage8. The latter gates I8/U8 and float8 buffer usage. Exact predicates report eight-bit storage true for sixteen-only capabilities and false for eight-only capabilities; both-present/both-absent controls behave normally.',
  'If this retained backend is ever reactivated, derive storage8 from buf_8 and its own eight-bit buffer feature. Retire or explicitly label obsolete source and avoid lifting its capability table into current code without conformance coverage.',
  'Four CPU predicate cases with explicit feature struct doubles, not actual Vulkan capability queries. Retained cubecl-wgpu0.10 is root-unselected; current Cargo.lock selects registry0.11. No current shipped GPU failure or concrete retained consumer proven.',probe_names[1]),
 ('Q437','Retained WGSL helper discovery skips switch cases and default bodies',
  'support/patches/cubecl-wgpu/src/compiler/wgsl/compiler.rs:register_extensions,compile_branch;src/compiler/wgsl/instructions.rs:Instruction::Switch;src/compiler/wgsl/shader.rs:Display',
  'Compiler lowers Switch scopes recursively and Instruction display emits their helper calls, but register_extensions descends only through If/IfElse/Loop/RangeLoop and ignores Switch. Exact discovery over a switch with IsNan in its case or default returns no helpers while the exact instruction formatter emits is_nan_1_f32. Direct and other recursive branch controls register both required helpers. ComputeShader writes only the collected extensions; it has no second discovery pass.',
  'Use one exhaustive recursive instruction visitor for extension discovery, descending into switch cases/default as well as other scope-bearing variants. Keep compiler lowering and emitted helper dependencies checked together.',
  'Exact register_extensions plus full unchanged WGSL base/extension/instructions source. Core Id/constant formatting/f16 storage/Subgroup types doubled; Subgroup never executed. CPU helper-list and emitted-call witness, no Naga/parser/device validation. Retained0.10 root-unselected; no current deployed caller claim.',probe_names[1]),
 ('Q438','Retained WGSL f64 NaN and Infinity helpers swap exponent and absolute-value masks',
  'support/patches/cubecl-wgpu/src/compiler/wgsl/extension.rs:select_inf_bits_abs_mask_uint,format_is_nan_primitive,format_is_inf_primitive',
  'The f64 tuple gives inf_bits=0x7fffffffffffffff and abs_mask=0x7ff0000000000000. Generated helpers therefore compare exponent-only bits against an all-non-sign-bits threshold. Exact selector and emitted text confirm f64::INFINITY and f64::NAN both classify as neither infinity nor NaN; the f32 control classifies both correctly. The mask also discards the fraction that distinguishes NaN from infinity.',
  'Correct the f64 constants in the shared helper selector and cover signed infinities, quiet/signaling NaN payloads and finite limits at the emitted-code boundary before reusing this retained backend.',
  'CPU IEEE bit evaluation uses the exact selector output and verifies actual emitted predicate strings. No shader execution, f64 device-support probe or current consumer reachability. Retained0.10 root-unselected.',probe_names[1]),
 ('Q439','Retained Vec4 indexed assignment emits no WGSL statement terminator',
  'support/patches/cubecl-wgpu/src/compiler/wgsl/instructions.rs:index_assign,Instruction::IndexAssign',
  'The Item::Vec4 index branch uses write! with a closing parenthesis but no semicolon/newline. Vec3 and Vec2 siblings use terminated statements. A complete exact formatter witness constructs a vector-index assignment and confirms its emitted assignment lacks a final semicolon; the Vec3 control includes it. The outer Instruction::IndexAssign arm returns the helper directly, adding no terminator.',
  'Terminate the Vec4 statement consistently and include formatter-level or parser validation for every indexed vector width. Establish a concrete lowered-IR caller before treating vector-index reachability as current product behavior.',
  'CPU emitted-string witness using full unchanged base/extension/instructions source with stated unused-type doubles. No Naga/parser validation, actual lowered kernel, GPU/backend build or current product failure; retained0.10 root-unselected.',probe_names[1])]
for ident,title,source,evidence_text,repair,scope,probe in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence_text,'status':'reproduced scoped CPU behavior with exact source and stated doubles; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':[probe]})
R='support/patches/cubecl-runtime/src/';W='support/patches/cubecl-wgpu/src/'
rows=[
 ('Adaptive survivor authority is distinct from fixed-count scoring','Schedule decides eligibility; process_request respects its decided index rather than rescoring eliminated candidates.',
  [R+'tune/schedule.rs',R+'tune/tuner.rs'],
  'The adaptive plan admits batches/groups/patience, retains two survivors, disqualifies late failures and refuses missing timestamps as NotMeasured. Fixed samples benchmark every admitted candidate; browser uses only fixed strategy and resolves profiles asynchronously. Different strategies share TunePlan, TuneFn sampling, result/cache commit and configuration, rather than independent operation identity. Short-circuit rules still differ: adaptive requires configured confirmations, fixed native compares full-sample median. Tests read; only the separately specified exact-body CPU cases ran.'),
 ('Fallback evidence collapses into a settled winner','Unknown is treated as executed-but-unmeasured even when exclusive access failed before execution.',
  [R+'tune/schedule.rs',R+'tune/tuner.rs',R+'tune/tune_cache.rs',R+'client.rs'],
  'Q435 follows actual native scheduling/commit bodies to Hit and settled. Unmeasured guesses remain memory-only; disk is explicitly suppressed and a TuneRecord may say stored=false. Fastest and settled still erase the guess distinction. NotMeasured and fixed all-error paths panic instead. These are source policies with a refusing-client CPU witness, not actual device contention or driver acceptance.'),
 ('Checksum guards loaded answers only once and does not validate index range','Persistent key is structured operation key plus candidate-list checksum; hydration stores an index and pending checksum verdict.',
  [R+'tune/tune_cache.rs',R+'tune/tuner.rs',R+'tune/local.rs',R+'tune/operation.rs'],
  'Q434 ambiguous preimage now has a source-qualified admission path: loaded ToBeVerified compares that string and becomes Match; the stored fastest_index is otherwise accepted. No independent candidate-count/index-range or result evidence check exists. Match fast paths do not rehash changing sets. This assumes stable per-ID candidate identity and trusted stored records. No actual persistence-feature tree/load or stale-index execution was run, so the earlier encoding witness is not promoted to a deployed-cache incident.'),
 ('TuneCache owns in-memory verdicts while Store owns persistence','One in_memory_cache contains Pending, verified picks and checksum verdicts; persistent Store uses lazy caching.',
  [R+'tune/tune_cache.rs',R+'tune/tuner.rs'],
  'Environment generation clears this map and hydration state only when persistent_cache exists; disabled/unbound process-local state intentionally survives switches. Persistent insertion reports success/failure distinctly and handles concurrent/out-of-sync writers. Hydration collapses multiple checksum-specific stored rows to the first row per K, so valid newer rows may be ignored and cause retuning. hydrated becomes true after one scan; async Store delivery semantics remain an external dependency to trace, not a proved failed browser hydration.'),
 ('Pending tuning state has no cancellation owner in these drivers','check_tune marks Pending before generation/planning/measurement and returns other callers immediately.',
  [R+'tune/tuner.rs',R+'tune/local.rs'],
  'No rollback guard removes Pending if a generator, planner, callback or all-failed path unwinds before commit. LocalTuner handles Pending by trying every operation. Browser spawn completion owns eventual insertion, with no cancellation/error transition represented here. Unrecoverable panic is documented for no viable kernel, so continuing after catching it is a followup lifetime contract rather than an asserted ordinary recovery defect. Single-candidate fast path caches0 without benchmarking by deliberate policy.'),
 ('Correctness checks are coupled to optional observability context','autotune-checks calls are supplied through Option<AutotuneLogContext>::set_checks.',
  [R+'tune/local.rs',R+'tune/log.rs',R+'tune/tuner.rs',R+'tune/tune_benchmark.rs'],
  'new returns None if both logger and decision sink are disabled; set_checks then does not invoke the closure. This precedes both cache paths and record-created contexts, so later recording cannot restore the skipped check. With a context, no-decision mismatches panic, decision-enabled std_io catches/reports them, and CheckResult does not itself exclude candidates from measurement/selection. This is a diagnostic boundary tied to logging configuration, not independent mandatory kernel correctness admission. No feature-enabled real backend equivalence test ran.'),
 ('Anchor level snapshots a second configuration lifetime','anchor uses a process-global AtomicI32 initialized once from runtime autotune level.',
  [R+'tune/util.rs',R+'tune/tune_cache.rs'],
  'Tuner persistence responds to environment generation but anchoring retains its first configured level. Full returns the input before min/max bounds; other levels quantize via floating powers and apply max then min. This can be intentional process-wide policy, but runtime-config refresh semantics are external and not yet a reproduced reconfiguration bug. Input marker HRTB remains shared in TuneInputs, not duplicated input ownership.'),
 ('Retained backend is a retired implementation rather than current root authority','Root selector is commented out; Cargo.lock selects registry cubecl-wgpu0.11 while retained package is0.10.',
  ['Cargo.toml',W+'../Cargo.toml',W+'backend/base.rs',W+'compiler/base.rs'],
  'The copy retains its own WGSL lowering, native Vulkan/Metal device and feature code, and build environment switches. Root comments explicitly retire it for archaeology but the until-stable-0.22 retention condition has passed according to the current0.22 lock. Record as obsolete ownership/provenance debt, not concurrent active WGPU authority. No deletion or upstream archival equivalence comparison was performed; all its source remains inside full-audit denominator.'),
 ('Shader compilation result has different native and browser evidence','Native WGSL validation errors return CompilationError; browser create_module schedules error logging then returns Ok(module).',
  [W+'backend/base.rs'],
  'Error-scope pop detaches before awaiting. Browser async error can arrive after pipeline creation/returned module, so Ok means handle creation, not validated executable shader. SPIR-V/MSL paths use unsafe passthrough and bypass this WGSL validation branch. Trusted WGSL disables bounds checks based on checked-IO lowering; checked mode only forces loop bounding at wgpu level. No live invalid shader/browser/backend case was exercised.'),
 ('Native capability query and generic feature projection must agree','ExtendedFeatures owns queried/enabled Vulkan feature chains; register_types projects their supported operations.',
  [W+'backend/vulkan.rs',W+'backend/vulkan/features.rs',W+'backend/metal.rs',W+'backend/wgsl.rs'],
  'Q436 demonstrates a projection consulting the wrong feature owner. Extended chains deliberately zero p_next after querying and replace duplicate structure types when composing device creation. Vulkan queue family is hardcoded0 TODO; Metal matrix combinations are hardcoded Metal3. Device limits can be widened for64-bit indexing using local heap capacity. These are platform capability followups without actual HAL query/device witnesses; generic WGSL and native passthrough paths are intentional separate backends.'),
 ('Backend creation does not prove reuse of the GUI device','request_device chooses Vulkan/Metal passthrough where supported and otherwise creates a WGSL device.',
  [W+'backend/base.rs',W+'backend/vulkan.rs',W+'backend/metal.rs',W+'backend/wgsl.rs'],
  'Each path can create a new Device/Queue and requests nearly all adapter features except MAPPABLE_PRIMARY_BUFFERS. MemoryUsage hints avoid a second batching layer; representation-aware pipeline binding visibility is conservative with sub-slices. Whether a caller imports an existing unified device is in remaining runtime/server source. No present violation of the unified-device architecture is inferred from these constructors alone.'),
 ('WGSL lowering shares external IR and checked-IO processors','Compiler performs unroll, checked-IO and saturating transforms before local instruction lowering.',
  [W+'compiler/base.rs',W+'compiler/wgsl/compiler.rs',W+'compiler/wgsl/base.rs',W+'compiler/wgsl/body.rs'],
  'AutoCompiler delegates representation-specific compilation; local formatter types are projections, not the product graph/model authority. Unsupported packed/semantic/matrix/TMA/barrier/type cases still panic rather than typed CompilationError in several branches. Body comment says2D Z=1 although emitted absolute index includes global_id.z. Feature/type admission must ensure unsupported cases cannot reach this lowering. No full compiler/device build was run.'),
 ('Recursive helper discovery has incomplete scope ownership','One register_extensions scan supplies ComputeShader extension definitions, but its scope visitor omits Switch.',
  [W+'compiler/wgsl/compiler.rs',W+'compiler/wgsl/instructions.rs',W+'compiler/wgsl/extension.rs',W+'compiler/wgsl/shader.rs'],
  'Q437 proves case/default helper calls survive exact formatting without registered definitions, while If/Loop/RangeLoop recurse and dedup. A shared exhaustive scope traversal would reduce this parallel-maintenance obligation. CPU actual formatter and scanner only; no parser or actual kernel reachability proof.'),
 ('Numeric helper text and IEEE meanings disagree for f64','One select_inf_bits_abs_mask_uint supplies both NaN and infinity generation.',
  [W+'compiler/wgsl/extension.rs'],
  'Q438 is a bug within a real shared helper: it centralizes the wrong f64 tuple rather than duplicate divergent owners. F16 intentionally converts to F32 before bit inspection. Negative odd power handling uses remainder==1 and needs a signed-exponent policy followup; it was not executed. This distinction keeps genuine shared authority visible despite arithmetic defects.'),
 ('Instruction formatting owns syntax but one width skips completion','IndexAssign directly returns index_assign output; Vec4 differs from Vec2/Vec3 punctuation.',
  [W+'compiler/wgsl/instructions.rs',W+'compiler/wgsl/base.rs'],
  'Q439 proves the missing terminator at emitted text. Atomic compare-exchange weak returns only old_value by stated CUDA compatibility policy, so spurious failure semantics remain an unexecuted cross-backend followup. Index length checking accepts optional len but the lowering arms here pass None after external checked-IO transform; safety depends on that prior transform, not this helper select alone. No parser/runtime claim.'),
 ('WGSL compiler instance retains mutable lowering state','compile_shader overwrites kernel/metadata/strategy but does not clear accumulated arrays and feature flags.',
  [W+'compiler/wgsl/compiler.rs',W+'compiler/base.rs'],
  'Repeated use of one compiler instance could carry declarations/flags into another kernel, while fresh per-compile construction would avoid it. Concrete retained server construction remains unread, so keep this as a caller-lifetime followup, not a reproduced shipped compilation bug. Constant/shared/local array IDs are scope-generated and projected locally, with shared/local dedup by ID.'),
 ('Shader metadata and bindings project shared Info into WGSL','ComputeShader formats buffer bindings, scalar/static/dynamic metadata and invocation builtins from collected state.',
  [W+'compiler/wgsl/shader.rs',W+'compiler/wgsl/base.rs',W+'backend/base.rs',W+'backend/wgsl.rs'],
  'Info is externally shared metadata layout authority; this formatter and pipeline layout must preserve its agreement with server argument encoding, still unread. Atomic bindings force read_write; sub-slice mode conservatively makes all storage read_write. The shader appends only its precomputed extension list after the body. Unused alignment field on SharedArray and type sizes versus WGSL layout remain source-qualified followups, not tested allocator/device facts.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1345):
 implementations=[str(Path(p).as_posix()) for p in implementations]
 # Normalize the retained manifest spelling rather than citing a path through src/..
 implementations=[p.replace('src/../Cargo.toml','Cargo.toml') for p in implementations]
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'runtime source complete; retained WGPU backend/compiler source through shader complete; remaining backend/runtime/tests pending','verification':NAME})
notes['review_followups'].extend([
 {'area':'Native tuner evidence and ownership','detail':'Q435 qualifies Unknown pre-execution refusal becoming a settled memory-only pick. Q434 now traced through checksum validation in source, without actual Store load. Logging-gated correctness checks, Pending unwind ownership, first-row checksum hydration and anchor configuration lifetime remain explicit followups.','verification':NAME},
 {'area':'Retained backend source and current reachability','detail':'Q436-Q439 cover capability predicate, switch helper visitor, f64 masks and Vec4 statement completion in retired0.10 source. Current root chooses registry0.11; do not promote these to current shipped failures. Finish subgroup, compute/runtime/server/storage/device and test source; trace compiler reuse and shared-device import before concluding ownership.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==3282 and stats['read_chars']==37223279 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Final6 CubeCL runtime tuner/cache/input/util files and retained WGPU backend/compiler through shader','files':24,'chars_newly_read':287539,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'Runtime71/71 source files650530chars complete. Retained WGPU18/33 files204040/313218chars complete.15 scoped CPU cases, no actual GPU/backend/browser/Store or parser build.'})
next_read={'path':'support/patches/cubecl-wgpu/src/compiler/wgsl/subgroup.rs','start':0,'reason':'Finish retained WGPU backend runtime/server/storage/device/tests, then selected matmul, retained reduce and remaining support/workflows.'}
pending='No audit-owned process running.3282/3738 source files complete;456 remain. Runtime71/71 complete; retained WGPU15 files109178chars remain.15 scoped CPU cases pass. All3738 live source membership/hashes unchanged. No production edit, owned binary, isolation, actual GPU/Store or parser run.'
notes['current_area']='CubeCL runtime complete; retained WGPU backend/compiler through shader complete';notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False});notes['checkpoint']['new_receipts'].extend([NAME,*probe_names])
addition=f'''

### Runtime tuning completion and retained WGPU backend/compiler review, 2026-10-17

Read another 24 complete files, 287,539 characters. All 71 CubeCL runtime source files are now read (650,530 characters). The retired WGPU copy is read through its shader formatter: 18/33 files, 204,040/313,218 characters. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. 456 files remain. Full live 3,738-file membership and hashes still match the inventory.

Q435: native adaptive tuning maps failure to acquire exclusive access to Unknown for every candidate, then treats that category as executed-but-unmeasured. Exact scheduling and commit bodies with a refusing CPU client enter the driver zero times yet produce a memory Hit and settled winner. A later actual operation execution cannot justify that earlier evidence claim. Source explicitly suppresses disk storage for unmeasured decisions, so this is not a demonstrated disk-cache defect. Missing-timestamp and fixed-strategy all-failed paths use different failure policies.

Q436-Q439 concern the retained, root-unselected cubecl-wgpu 0.10 implementation. Its 8-bit Vulkan storage predicate reads 16-bit features; helper discovery ignores switch scopes; f64 NaN/Infinity masks are swapped; Vec4 indexed assignment omits its terminator. Current root Cargo.lock selects registry cubecl-wgpu 0.11, and these findings are not promoted to current deployed failures. Exact predicates, full formatter/helper source and scoped surrounding doubles provide CPU witnesses. No actual Vulkan query, Naga/parser validation, current concrete kernel caller or GPU execution occurred.

Fifteen CPU cases pass: three exact scheduling/cache-commit cases and twelve retained-backend predicate, helper-discovery, bit-classification and emitted-string cases. Four owned executable/PDB outputs were verified and removed. A1345-A1361 track survivor selection versus result scoring, process-local fallback evidence, checksum/index admission, environment/hydration ownership, Pending cancellation, logging-gated correctness checks, anchor configuration lifetime, retired backend authority, browser validation evidence, native feature projections, compiler lifetime and metadata layout dependencies. Q434 now has a source trace through persisted checksum admission, still without a real Store-load witness.

Prior report bytes, protected source/shared files and the Git index are preserved. All current source hashes remain unchanged, and retained receipts remain below 100 MB. No production edits, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [tuner probe](../../../support/receipts/authority-success-audit/{probe_names[0]}) and [retained backend probe](../../../support/receipts/authority-success-audit/{probe_names[1]}). Continue subgroup.rs, then the remaining backend runtime/server/storage/device/tests. The full-source objective remains active and incomplete.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':OLD,'sha256':sha(HERE/OLD)},'coverage':stats,'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[v[0] for v in new],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{p:sha(HERE/p) for p in dict.fromkeys(evidence)},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only.24 complete files287539chars.15 CPU cases with explicit refusal/formatting/predicate limits. Four owned exe/PDB outputs removed. Runtime source complete;15 retained backend files remain. No actual GPU, Store or parser execution or production edit; all3738 live source membership/hashes unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
