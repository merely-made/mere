"""Preserve source and append backend completion/matmul definition findings."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='wgpu_completion_matmul_review_receipt.json';OLD='runtime_tuner_wgpu_review_receipt.json'
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
paths=[p for p in i if p.startswith(('support/patches/cubecl-wgpu/','support/patches/cubek-matmul/')) and c.get(p,{}).get('through',0)>0 and p not in guards]
assert len(paths)==36 and sum(c[p]['through'] for p in paths)==181107 and sum(c[p]['through']==i[p]['chars'] for p in paths)==35
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']<=i[p]['chars'] for p in paths)
probe_names=['retained_wgpu_completion_probe_receipt.json','matmul_definition_probe_receipt.json'];evidence=['record_wgpu_completion_matmul_review.py'];external=dict(old['external_context_sha256'])
for name,passed in zip(probe_names,[7,13]):
 probe=load(name);assert probe['exit']==0 and probe['passed']==passed
 assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
 assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
 assert all(sha(Path(p))==h for p,h in probe['external_context_sha256'].items());external.update(probe['external_context_sha256'])
 assert all(not Path(p['path']).exists() for p in probe['removed_outputs'])
 evidence.extend([name,*probe['evidence_sha256']])
builder=Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cubek-test-utils-0.3.0/src/test_tensor/base.rs');external[str(builder)]=sha(builder)
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i) and all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json');assert notes['findings'][-1]['id']=='Q439' and authority['entries'][-1]['id']=='A1361'
W='support/patches/cubecl-wgpu/src/';M='support/patches/cubek-matmul/src/'
new=[
 ('Q440','Retained write-only flush reports success without submitting queued writes',
  [W+'compute/stream.rs:write_to_buffer,flush,sync'],
  'write_to_buffer increments pending_write_count, not tasks_count. flush returns flush_errors immediately when tasks_count==0, regardless of pending writes. One aligned write followed by exact flush returns Ok with zero submissions and unchanged model buffer; the compute-task and64-write threshold controls submit it. sync calls the same flush then on_submitted_work_done, whose actual wgpu API contract covers the previous submission. Thus its source path has no admission of queued write-only work before declaring submitted work done.',
  'Make flush submit when either encoded tasks or pending queue writes exist, and preserve ordering/error attribution. Keep write staging progress distinct from GPU completion, including the final partial batch after the64-write threshold.',
  'Three CPU cases compile exact write_to_buffer/flush with an explicit queue/device/stream double. Model semantics match inspected local wgpu30.0.1 Queue docs. No real sync callback, actual GPU, retained backend build or deployed failure; root-unselected0.10 source.',probe_names[0]),
 ('Q441','Retained mixed-device enumeration and selection use incompatible index spaces',
  [W+'runtime.rs:enumerate_all_devices,enumerate_devices,select_from_adapter_list',W+'device.rs'],
  'enumerate_all_devices and enumerate_devices(type_id4) number the combined adapter list before returning type-specific DeviceIds. Selection then numbers only matching device types. Exact code over[integrated,discrete0,discrete1] assigns discrete0 the ID DiscreteGpu(1), which selects discrete1; DiscreteGpu(2) panics when loaded. Explicit type-filtered enumeration produces0/1 and round-trips correctly.',
  'Assign per-type ordinals when constructing typed device IDs, or use a stable adapter identity throughout enumeration and selection. Share one mapping rather than independently enumerating global and filtered lists.',
  'Three CPU cases use exact enumeration/selection bodies and deterministic adapter inventory doubles. No real platform adapters or client/device creation; retained0.10 root-unselected.',probe_names[0]),
 ('Q442','Retained imported setup advertises adapter capabilities beyond its enabled device',
  [W+'runtime.rs:WgpuSetup,init_device,create_server',W+'backend/wgsl.rs:register_types',W+'backend/base.rs:register_features'],
  'init_device accepts an externally created WgpuSetup. create_server reads actual device limits for memory/shared size but adapter limits for workgroup counts/invocations/dimensions, adapter features for timing/subgroups, and passes the adapter to backend type-feature registration. An imported device may enable fewer features and lower limits. Inspected wgpu30.0.1 Device docs explicitly say only requested features/limits are usable, even if the adapter supports more. The source therefore projects physical possibility as executable device authority; enabling device timing from an adapter-only TIMESTAMP_QUERY is one concrete unsupported configuration.',
  'Use the enabled Device as the execution feature/limit owner. Retain adapter-only hardware facts separately, intersect native extension capability with enabled setup state and validate imported setup backend/queue/device consistency.',
  'Source and primary local wgpu API contract finding only. No constructed low-feature actual device, timestamp query failure, full backend build or current selected0.11 comparison; retained root-unselected0.10. Do not call this an observed current shared-device failure.',None),
 ('Q443','Retained existing-device IDs collide long before their documented overflow guard',
  [W+'runtime.rs:init_device',W+'device.rs:Device::to_id'],
  'init_device uses an AtomicU32 and guards only u32::MAX, describing generated IDs as globally unique. WgpuDevice::Existing(u32).to_id converts the value to u16. Exact device implementation maps Existing(0) and Existing(65536) to the same DeviceId even though the public WgpuDevice values differ. Actual registry behavior after collision remains external and was not run.',
  'Use a device ID representation wide enough for the counter, or guard allocation at the actual index domain limit before creating/registering a server. Keep public ID generation and registry identity width under one contract.',
  'One CPU case compiles full device.rs with a shape-equivalent DeviceId/Device trait. No65,537 actual devices, registry insertion, resource exhaustion or live aliasing witnessed; retained0.10 root-unselected.',probe_names[0]),
 ('Q444','Selected matmul batch broadcast turns an empty dimension into a nonempty one',
  [M+'definition/base.rs:broadcast_batches,MatmulProblem::from_parameters'],
  'broadcast_batches accepts singleton compatibility then takes max(l,r). For[0] with[1], symmetric[1] with[0], and implicit-leading-singleton[] with[0], exact source returns[1]. NumPy2.4.3 broadcast_shapes returns[0] for all three. Equal-zero, ordinary positive broadcast and conflicting-dimension controls pass. from_parameters consumes this helper for out_batches and output shape, so its declared NumPy-style rule loses empty-batch identity.',
  'Select the nonsingleton extent when one extent is1; do not use numeric maximum to implement singleton broadcast. Preserve zero extents and qualify zero-size launch/selection separately.',
  'Five CPU tests compile exact helper with Shape as Vec; installed NumPy2.4.3 is the independent broadcast oracle. Current root selects this local cubek-matmul0.3. No actual tensor/kernel launch, allocation or current product empty-input incidence.',probe_names[1]),
 ('Q445','Matmul benchmark column layouts can exceed allocation or alias logical elements',
  [M+'eval/benchmarks/gemm/benchmark.rs:GemmBench::prepare',M+'eval/benchmarks/gemm/problem.rs:SHAPES,LAYOUTS',M+'eval/cpu_reference.rs:seed_inputs'],
  'Benchmark prepare generates the requested logical row-major shape then swaps only its last two strides for column layout. A2x3 allocation has strides[3,1], changed to[1,3] while keeping shape2x3; its maximum addressed extent becomes8 elements from a6-element allocation. A3x2 view produces aliasing between[2,0] and[0,1]. Both lhs/rhs arms are affected; square/row controls pass. The benchmark catalogue contains rectangular shapes with all four layout combinations. Correctness seeding uses shared builder.layout instead, so successful correctness is evidence for a different physical layout than this timed preparation.',
  'Use the shared layout-aware TestInput builder in benchmark preparation as correctness does, or construct a correctly transposed base shape and swap both shape/strides. Time the same admitted problem whose output correctness is established.',
  'Five CPU cases compile exact prepare with explicit client/device/tensor-builder metadata doubles, shared exact stride computation and a cached builder-default contract check. No actual TestInput execution, GPU access, benchmark timing or rendered measurement receipt. This code is benchmarks-feature gated.',probe_names[1]),
 ('Q446','Matmul CPU reference indexes every operand using output rank and panics on valid rank broadcast',
  [M+'eval/cpu_reference.rs:matmul_cpu_reference',M+'definition/base.rs:MatmulProblem'],
  'matmul_cpu_reference allocates lhs_index/rhs_index at output rank and fills batch axes from the left, then passes them to HostData.get_f64. A rank2 lhs[1,2] and rank3 rhs[2,2,1] producing[2,1,1] cause exact cached HostData.strided_index to index lhs strides[2] and panic. Expected dot products are23 and33. Same-rank ordinary and singleton-broadcast controls return those values correctly. Problem constructors support different ranks, but the claimed CPU ground truth has not migrated with that shape contract.',
  'Construct operand indices in each operands own rank and align batch axes from the right. Keep folded-output and broadcast interpretation explicit, and use a reference independent of the kernel lowering while covering the full admitted shape contract.',
  'Three CPU cases compile exact matmul_cpu_reference/num_batches plus exact cached HostData get_f64/strided_index and StridedLayout.compute_strides. Shape/container/payload/progress/problem types doubled. No actual TestInput, GPU, strategy selection or full crate build; cpu-reference feature only.',probe_names[1])]
for ident,title,sources,evidence_text,repair,scope,probe in new:
 notes['findings'].append({'id':ident,'title':title,'source':';'.join(sources),'evidence':evidence_text,'status':'documented, not repaired; scoped CPU witness where named, otherwise source/primary-contract finding','scope':scope,'repair_triage':{'disposition':'documented in read-only full-source audit','reason':repair},'verification':NAME,'diagnostics':[] if probe is None else [probe]})
rows=[
 ('Fresh compiler construction rules out the suspected retained cross-kernel state leak','WgpuServer::pipeline constructs compiler(backend, options) anew for every uncached compilation.',
  [W+'compute/server.rs',W+'compiler/wgsl/compiler.rs'],
  'This closes A1360 for the actual server path: accumulated flags/arrays do not leak from one compiled kernel into the next there. Public manual compiler reuse remains an API-lifetime question, not a demonstrated product bug. In-memory pipeline key includes kernel execution mode; cached pipeline path returns before fresh validation. Author-supplied kernel identity and persisted properties hash still own cache consistency.'),
 ('Queued resource ownership outlives bindings at native-handle granularity','prepare_bindings converts managed bindings into WgpuResource native buffer clones before enqueue.',
  [W+'compute/server.rs',W+'compute/schedule.rs',W+'compute/storage.rs'],
  'Code comment admits managed lifetime is not tied to the resource. A cloned native Buffer keeps allocation alive but does not retain a managed sub-slice reservation. Scheduler dependency/order authority must prevent region reuse before encoded use completes. get_resource explicitly returns ManagedResource with its binding, whereas scheduled tasks carry raw WgpuResource. No allocator reuse race was run; keep region lifetime distinct from native handle liveness.'),
 ('Per-stream memory and queue share different ownership domains','Each WgpuStream creates its own main/uniform/staging managers but clones a common Device/Queue.',
  [W+'compute/mem_manager.rs',W+'compute/schedule.rs',W+'compute/stream.rs'],
  'Main allocations follow configured policy; staging/uniforms force ExclusivePages and Auto. Main-only memory_usage/cleanup omit those other pools. Uniform handles are retained until flush submission then cleared. This is a scoped usage projection, not total device accounting. Scheduler uses explicit binding streams for launch dependencies; writes schedule on requested stream with empty dependency list after resource lookup from binding stream, a cross-stream followup.'),
 ('Mapped readback retains a binding and unmaps on controller drop','WgpuAllocController holds view, native buffer and ManagedMemoryBinding.',
  [W+'compute/controller.rs',W+'compute/stream.rs'],
  'Drop releases the view before unmap and binding release. read_resources copies aligned native regions, maps buffers then creates lazy pinned Bytes controllers; mapping/channel failures panic despite Result return type. memory_mut casts an immutable mapped view to mutable memory under caller contract; compatibility with map-read/write/alias rules needs external API proof. No unsafe-memory mutation or device mapping failure witness was run.'),
 ('Polling tokens do not independently own the polling worker lifetime','WgpuPoll uses Arc count to decide activity and Drop sends cancellation without joining.',
  [W+'compute/poll.rs',W+'compute/stream.rs',W+'compute/timings.rs'],
  'Read/profile futures retain activity Arc tokens, not WgpuPoll itself. After WgpuPoll drops the threshold>2 no longer represents the same owner count, so an outstanding lone token may coexist with cancellation. Future/server lifetime and external registry retention must be traced before claiming a real hang. Native poll failures warn; browser relies on browser event loop and no worker. No live thread or device poll test ran.'),
 ('Queue-write admission is not counted as encoded task admission','pending_write_count and tasks_count govern different submission triggers.',
  [W+'compute/stream.rs',W+'compute/schedule.rs',W+'compute/server.rs'],
  'Q440 proves success without submitting a write-only tail. Sixty-four writes explicitly submit an empty encoder; regular flush only sees encoded tasks, and scheduling flush ignores errors by policy. sync watches the previous submission after that flush. Keep flush/error-drain, submission and completion distinct. CPU aligned queue model only; primary wgpu contract inspected.'),
 ('Profile measurement and no-work fallback have different validity','QueryProfiler returns device ProfileTicks(now,now) when no query set was registered.',
  [W+'compute/timings.rs',W+'compute/stream.rs'],
  'This manufactures measured zero for no work, while the selected0.11 runtime explicitly represents missing measurements and refuses NotMeasured. The retained profiler0.10 is not currently paired with that runtime, so do not invent a deployed mixed-version path. Query-set refcount cleanup, timestamp epoch and saturation are separate from real duration correctness. profile-tracy get_cur_timestamp still dereferences Result from get_mapped_range despite other30.0 adaptations; actual feature build remains unrun.'),
 ('Imported device execution capabilities must come from the device owner','WgpuSetup permits existing Device/Queue import but create_server reads adapter-only feature and compute limit data.',
  [W+'runtime.rs',W+'backend/base.rs',W+'backend/wgsl.rs',W+'device.rs'],
  'Q442 records the unfinished capability ownership migration at the shared-device seam. Default constructors request almost all adapter features/limits, concealing the mismatch; external import can enable a narrower contract. init_device is a real shared-device route, so the architecture does not inherently require a second device. WgpuSetup single-device-per-adapter note is broader than local wgpu current native enforcement, though browser restriction is documented.'),
 ('Enumerated physical devices and typed device ordinals diverge','enumerate_all_devices numbers a mixed list but select_from_adapter_list filters by type first.',
  [W+'runtime.rs',W+'device.rs'],
  'Q441 gives exact mixed-adapter misroute/out-of-range evidence. DeviceId is registry authority, while per-type ordinal is a user-facing selector; they cannot silently share inconsistent index spaces. Other adapters are fallback candidates at the same ordinal; request_adapter_with_preference .expect text claims falling back to first adapter but actually panics. Environment override is concrete std parsing, warning name differs from the actual variable.'),
 ('Existing-device unique-ID claim exceeds registry width','AtomicU32 counter and Existing(u32) ultimately narrow to DeviceId u16.',
  [W+'runtime.rs',W+'device.rs'],
  'Q443 proves encoding collision at65536, before the U32 overflow guard. init_device ignores ComputeClient::init return, so registry collision outcome is external, not proved successful wrong-device registration. Separate from current runtime A1321, this retained source shows the same width seam explicitly. No devices allocated in the CPU probe.'),
 ('Retained WGPU source and test surface are now completely read','All33 inventoried files313218chars complete, including macro-generated test declarations.',
  [W+'lib.rs',W+'graphics.rs',W+'element.rs',W+'runtime.rs'],
  'Root selector remains retired0.10; current root0.11 registry implementation was not substituted for this source denominator. Exported core/std test macros name extensive dtype/kernel tests, but their source declarations are not execution receipts and external macro-expanded suites are not claimed read/run. AutoGraphicsApi selects browser/Metal/Vulkan by target; the test-only environment path is separate from runtime device override.'),
 ('Selected matmul architectures share vocabulary and branch at public Strategy','tiled and multi-level are separately feature gated, sharing definition/routine/strategy/tune_key.',
  [M+'lib.rs',M+'definition/mod.rs',M+'launch.rs','support/patches/cubek-matmul/Cargo.toml','support/patches/cubek-matmul/Cargo.toml.orig'],
  'At least one architecture is required. The standalone patch workspace reuses local runtime and self selectors; root also selects this local0.3 crate. Normalized manifest and original manifest differ in dev self-dependency and workspace context, so neither is a substitute for checking active feature resolution. Definitions read so far centralize problem/precision/error vocabulary; algorithm selection and launches remain unread.'),
 ('Batch broadcasting and reference indexing have not migrated to one shape contract','MatmulProblem supports rank-specific extents and NumPy-style batch broadcasting; reference still assumes output rank everywhere.',
  [M+'definition/base.rs',M+'eval/cpu_reference.rs'],
  'Q444 zero/singleton extent bug is in shared helper, not duplicate implementations. Q446 is divergence between admitted problem shapes and CPU ground truth. from_shapes_and_strides now reads each operand rank separately, but relies on layout validation for rank/stride admission and does not directly compare contracted extents/output volume here. Those validations need full launch tracing; no extra malformed-shape defect asserted.'),
 ('Accumulator presence is explicit launch ownership metadata','AccumulatorOperand distinguishes absent product from product-plus-accumulator and defaults absent in both constructors.',
  [M+'definition/base.rs',M+'definition/error.rs'],
  'Comment correctly explains extra shared-memory stage cannot be inferred from ordinary shape. Callers must propagate presence into selection; budget/blueprint/launch callers are still unread. SharedMemoryTooBig typed availability error makes refusal explicit rather than success. This is promising explicit ownership, with migration completeness pending, not proof all routines account for it.'),
 ('Dynamic element selection and deprecated type-level precision are separate mappings','MatmulElems and MatmulPrecision both name global/stage/register types.',
  [M+'definition/elems.rs',M+'definition/precision.rs',M+'launch.rs'],
  'Dynamic from_globals widens f16/bf16 accumulators toF32; deprecated precision uses platform-dependent half accumulators on macOS and mapsF64 stage/registers toF32. This is material precision divergence, not interchangeable aliases. from_single_dtype forces all equal; launch_ref documents inner stage/register mutation. Trace selected routines and test caller claims before treating declared global dtype as actual compute precision.'),
 ('Performance cost is an estimate rather than a measured operation identity','MatmulCost counts2K-1 arithmetic operations and all batch operand reads/writes.',
  [M+'definition/cost.rs',M+'eval/benchmarks/gemm/mod.rs'],
  'Broadcast reuse, quantized payload/scales, padding and accumulator traffic are not represented in this minimal estimate. compute_peak_ops_per_s tests peak>0 but not finiteness, so positive infinity could pass; concrete throughput probe controls are elsewhere. Original cost tests fully read, not run. This cost projection feeds category work; it must not be confused with actual issued/read traffic or roofline model validation.'),
 ('Vector-size admission shares external tensor layout filtering','AvailableVectorSizes starts from client IO optimized sizes and filters using tensor_vector_size_parallel.',
  [M+'definition/vectorization.rs'],
  'TMA forces input vector sizes1 while output remains optimized. Rank-derived target dimensions assume rank at least2 for column layouts and at least1 for output. pick_max returns typed NoValidVectorization when lists empty. External helper and downstream routine constraints own the remaining layout agreement; no extra backend vector-width claim was executed.'),
 ('Benchmark and correctness data preparation are siloed implementations','GemmBench.prepare swaps metadata strides; correctness seed_inputs uses layout-aware shared builder.',
  [M+'eval/benchmarks/gemm/benchmark.rs',M+'eval/benchmarks/gemm/correctness.rs',M+'eval/cpu_reference.rs',M+'eval/benchmarks/gemm/problem.rs'],
  'Q445 exposes the practical cost of these duplicate preparations: rectangular column cases measure different/invalid physical data from the corresponding correctness case. Benchmark rhs binding uses lhs dtype, currently hidden because catalogue precisions use matching operand types. Correctness reference requires two seeds without checking length. No GPU benchmark was run.'),
 ('Catalogue labels and correctness hooks do not themselves prove receipts','GEMM/CPU-GEMM categories expose independent benchmark, work and correctness callbacks.',
  [M+'eval/benchmarks/gemm/mod.rs',M+'eval/benchmarks/gemm/problem.rs',M+'eval/benchmarks/gemm/strategy.rs',M+'eval/benchmarks/gemm_cpu/mod.rs',M+'eval/benchmarks/mod.rs',M+'eval/mod.rs'],
  'The generic catalogue concatenates strategies from whichever architectures compile. CPU category restricts names and row/row shapes by string filters, avoiding Q445 column metadata cases there. It delegates work and correctness to generic GEMM. Optional hooks are capability declarations, not evidence that each timed problem was checked. Very large historical batches remain choices in the benchmark harness, with no allocation/timing receipt here.'),
 ('CPU ground truth shares host layout/index utilities but not arbitrary-rank correctness','matmul_cpu_reference calls cached HostData.get_f64 and shared StridedLayout output strides.',
  [M+'eval/cpu_reference.rs',M+'definition/base.rs'],
  'Q446 proves valid rank mismatch panics through exact dependency stride indexing. CPU reference accumulates inF64 then converts to declared output dtype; epsilon helper takes maximum epsilon across all global/stage/register elements times a safety factor. produce_with captures launch outcome and only reads executed output, a real separation from compile error. Its independent arithmetic still needs the same full admitted shape contract; no actual reference validation campaign ran.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1362):
 assert all((ROOT/p).exists() for p in implementations)
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'retained backend complete; selected matmul definitions/evaluation complete; launches/routines pending','verification':NAME})
notes['ruled_out'].append({'area':'Retained WGPU mutable compiler state','detail':'A1360 followup ruled out for concrete WgpuServer::pipeline: it creates a fresh compiler for each uncached kernel. Manual public reuse remains separate.','verification':NAME})
notes['review_followups'].extend([
 {'area':'Retained backend completion','detail':'Q440-Q443 cover write-only submission, mixed-adapter ordinals, imported-device capability owner and existing-ID width. Current root does not select retained0.10. Query-profiler no-work zero, profile-tracy30.0 Result adaptation, native poll-token owner, mapped-read mutability, scheduler write dependencies and raw sub-slice lifetimes remain source-qualified followups.','verification':NAME},
 {'area':'Selected matmul shape and evidence migration','detail':'Q444 empty batch broadcast, Q445 benchmark layout split, Q446 CPU-reference rank contract. Finish routines/strategy/args and launch paths, explicit accumulator shared-memory accounting, precision mapping and vectorization admission. Initial args.rs517chars only; do not claim its remaining19448chars read.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT));assert stats['complete_files']==3317 and stats['read_chars']==37404386 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'Retained WGPU completion plus selected matmul definition/evaluation source','files':35,'chars_newly_read':181107,'verification':NAME,'basis_commit':old['basis_commit'],'qualification':'15 retained backend files109178chars complete; all33/313218 retained source complete.20 matmul files71412chars complete and args.rs517chars read.20 CPU cases pass; one additional source/primary-contract imported-device finding. No actual GPU/driver/benchmark or full dependency build.'})
next_read={'path':M+'multi_level/args.rs','start':517,'reason':'Complete selected matmul args, algorithms, routines, launches, tuning and tests, then retained reduce and remaining support/workflows.'}
pending='No audit-owned process running.3317/3738 source files complete;421 remain. Retained WGPU33/33 complete; matmul20/317 complete plus517chars of args.20 scoped CPU cases pass. All3738 live source membership/hashes unchanged. No production edit, generated owned binary, isolation, actual GPU/driver/benchmark/reference run.'
notes['current_area']='Retained WGPU complete; selected matmul definition/evaluation source complete; args517chars read';notes['next_area']=next_read['reason'];notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False});notes['checkpoint']['new_receipts'].extend([NAME,*probe_names])
addition=f'''

### Retained WGPU completion and selected matmul definition/evaluation review, 2026-10-17

Read another 181,107 characters: 35 complete files and the first 517 characters of matmul multi_level/args.rs. All 33 retained WGPU files (313,218 characters) are now read. Twenty of 317 selected matmul files are complete. Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters; 421 files remain. All current source membership/hashes remain unchanged.

Q440-Q443 concern the retired, root-unselected WGPU0.10 copy: write-only flush succeeds without submitting its queued tail; mixed-adapter enumeration assigns global ordinals that type-filtered selection misinterprets; imported setups advertise adapter capabilities beyond the enabled Device; Existing(u32) IDs narrow to16 bits long before their overflow guard. Seven exact-body CPU cases qualify queue/enumeration/encoding behavior. The imported-device issue is a source/primary-API-contract finding only. No actual GPU, driver enumeration, imported low-feature device or real sync callback was exercised. The concrete server constructs a fresh compiler for each uncached kernel, ruling out A1360's suspected state leak on that path.

Q444-Q446 concern the selected matmul0.3 source and its feature-gated evidence harness. Shared broadcast_batches turns zero with singleton into one, unlike installed NumPy2.4.3. Benchmark preparation swaps strides without changing rectangular logical shapes, causing out-of-allocation views or aliasing; correctness preparation uses a separate layout-aware builder. The CPU reference uses output rank to index every operand and panics on valid rank-mismatched broadcast inputs. Thirteen exact-body CPU cases include ordinary/rank/layout controls and cached shared host-index/stride functions. No actual TestInput, kernel launch, GPU access, benchmark timing or full crate build occurred.

A1362-A1381 record compiler lifetime, native handle versus managed region ownership, per-stream pools/common queue, readback controller cleanup, polling-token lifetime, queue submission versus completion, no-work timing, imported-device authority, device ID/index spaces, selected matmul architecture vocabulary, accumulator presence, precision mappings, cost projections, vector admission and the split benchmark/reference preparation. Unfinished migration details stay alongside individual bugs rather than being replaced by file-splitting recommendations.

Twenty scoped CPU cases pass. Four exact owned executable/PDB outputs were verified and removed. Prior report bytes, source/shared files and Git index are preserved; retained receipts remain below100MB. No production edit, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [retained completion probe](../../../support/receipts/authority-success-audit/{probe_names[0]}) and [matmul definition probe](../../../support/receipts/authority-success-audit/{probe_names[1]}). Continue multi_level/args.rs at character517. Full-source review remains active and incomplete.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={**old,'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':OLD,'sha256':sha(HERE/OLD)},'coverage':stats,'source_sha256':{p:sha(ROOT/p) for p in paths},'preserved_production_and_test_hashes':guards,'read_through':{p:c[p]['through'] for p in paths},'new_findings':[v[0] for v in new],'authority_entries':ids,'next_read':next_read,'pending_verification':pending,'external_context_sha256':external,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{p:sha(HERE/p) for p in dict.fromkeys(evidence)},'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only.35 complete files plus517chars next;181107chars newly read.20 scoped CPU cases and one source/primary-contract finding. Four owned exe/PDB outputs removed. Retained WGPU source complete; selected matmul algorithms/launches/tests remain. No actual GPU/driver/benchmark/reference run or production edit; all3738 live source membership/hashes unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**record['source_sha256'],**old['auxiliary_read_sha256'],**old['runtime_input_sha256']}.items())
assert all(sha(ROOT/p)==h for p,h in old['shared_working_file_sha256'].items() if p!=old['document']['path'])
assert all(sha(Path(p))==h for p,h in external.items())
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
