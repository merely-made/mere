"""Append-only checkpoint: complete retained Burn CubeCL source review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,subprocess,sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='retained_burn_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('support_qualifier_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256']);runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256'];external=dict(old['external_context_sha256'])
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(Path(p))==h for p,h in external.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
i=load('source_inventory.json')['files'];c=load('read_coverage.json')
paths=[p for p in i if p.startswith('support/patches/burn-cubecl/')]
assert len(paths)==119 and sum(i[p]['chars'] for p in paths)==695238
assert all(sha(ROOT/p)==i[p]['sha256']==c[p]['sha256'] and c[p]['through']==i[p]['chars'] for p in paths)
assert load('pending_read.json')==[];sources={p:sha(ROOT/p) for p in paths}
probes=['retained_burn_layout_probe_receipt.json','retained_burn_zero_matmul_probe_receipt.json']
evidence=['record_retained_burn_review.py']
for name,passed in zip(probes,[13,5]):
 probe=load(name);assert probe['exit']==0 and probe['passed']==passed
 assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
 assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
 assert all(not Path(out['path']).exists() for out in probe['removed_outputs'])
 evidence.extend([name,*probe['evidence_sha256']])
selection=load(probes[0])['root_selection_input_sha256'];assert all(sha(ROOT/p)==h for p,h in selection.items())
aux.update(selection)
# Inventory validation is separate from reading credit. Include explicit source files.
spec=importlib.util.spec_from_file_location('audit_source_membership',HERE/'read_coverage.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
explicit={'design_docs/mere_docs/receipts/natal-cap-20261017/mere-intake-narrow-final.diff','ports/graphshell/web/extension/manifest.chromium.json','ports/graphshell/web/extension/manifest.firefox.json'}
live=set()
for p in subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=ROOT).decode().split('\0'):
 if not p or p.startswith('support/receipts/authority-success-audit/'):continue
 f=ROOT/p
 if f.is_file() and (module.is_source(f) or p in explicit):live.add(p)
assert live==set(i)
assert all(sha(ROOT/p)==v['sha256'] for p,v in i.items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q423' and authority['entries'][-1]['id']=='A1281'
new=[
 ('Q424','Retained pointwise convolution flattening ignores gaps between batches',
  'support/patches/burn-cubecl/src/kernel/conv/im2col.rs:reshape_input,is_spatial_contiguous',
  'The actual is_spatial_contiguous predicate accepts shape[2,1,2,1], strides[4,2,1,1]. Enumerating the original view yields offsets[0,1,4,5]; the reshape_input flattening metadata yields[0,1,2,3]. The predicate checks channel/spatial contiguity without verifying batch stride, yet the reshape merges batch and spatial axes. Forward pointwise, data-gradient and weight-gradient paths use it. True contiguous and spatial/channel gap controls distinguish the missing batch condition.',
  'Require the merged batch axis to be contiguous too, or materialize the view through the shared layout/reshape authority. Add aligned sliced-batch tensor tests before any reactivation.',
  'Four CPU tests use the unchanged layout predicate and independently enumerated source/flattened addresses. No device-aligned real slice construction, convolution values, CubeCL lowering or GPU execution. The tiny view is an arithmetic counterexample; actual slice code can materialize unaligned byte offsets. Root selects registry burn-cubecl0.22.0, not this retained0.22.0-pre.4 copy.',probes[0]),
 ('Q425','Retained direct transpose-convolution range excludes a valid unit-kernel origin',
  'support/patches/burn-cubecl/src/kernel/conv/conv_transpose2d/transpose_direct.rs:input range arithmetic',
  'The exact stride/dilation/padding integer range block yields empty rows(0,0) and columns(0,0) for a1x1 kernel, stride3, dilation1, padding0, output origin0 and2x2 input. Input(0,0)*weight(0,0) should contribute at output(0,0). The kernel-size*dilation-stride term produces the wrong lower bound for this valid shape. Unit-stride unit/three-kernel controls include the origin.',
  'Derive inverse bounds from the transposed-convolution equation and use a shared tested bounds helper rather than separately maintained2D/3D arithmetic. Check dilation, padding, stride wider than kernel and boundaries before reactivation.',
  'Three CPU cases compile the exact extracted2D range block with scalar args/input/clamp doubles. No shader compilation or convolution value run.3D source has related arithmetic but different unsigned operations; this executed result is not generalized to3D. Retained/root-unselected copy only.',probes[0]),
 ('Q426','Retained zero-inner-dimension matmul returns nonempty output without zeroing',
  'support/patches/burn-cubecl/src/kernel/matmul/tune/base.rs:matmul_autotune; kernel/matmul/utils.rs:init_matmul_output; ops/numeric.rs:empty_device_dtype',
  'matmul_autotune treats any zero input dimension as proof output is zero-sized. For[2,0]@[0,3], output has six elements and each empty dot product is0. The exact initialization/early-return prefix returns a supplied six-element17-filled output unchanged without dispatch. With out=None, the prefix returns its allocator output without initialization; source init_matmul_output calls empty_device_dtype, which calls client.empty_tensor rather than fill. M0/N0 controls truly have empty outputs; nonzero operands reach the dispatch marker.',
  'Distinguish empty output dimensions from empty reduction K. Explicitly zero the nonempty output for K0, including supplied output, through the normal fill authority. Keep shape validation and empty identities shared between tuning and non-tuning paths.',
  'Five CPU tests compile the exact early-return prefix with metadata/tensor/allocator doubles. Supplied-output behavior is concrete; allocator sentinel shows absent initialization in this prefix, not observed uninitialized device memory. No backend/GPU execution or registry-backend failure. Retained/root-unselected copy only.',probes[1])]
for ident,title,source,evidence_text,repair,scope,probe in new:
 notes['findings'].append({'id':ident,'title':title,'source':source,'evidence':evidence_text,'status':'reproduced retained-source CPU arithmetic/control flow; documented, not repaired','scope':scope,'repair_triage':{'disposition':'documented retired-copy reactivation hazard in read-only audit','reason':repair},'verification':NAME,'diagnostics':[probe]})
P='support/patches/burn-cubecl/'
rows=[
 ('Retained Burn backend is archaeology, not root dependency authority','Root Cargo selection separates archived source from shipping dependency selection.',
  ['Cargo.toml','Cargo.lock',P+'Cargo.toml',P+'Cargo.toml.orig'],
  'All119 retained source/manifests read. Root retires burn-cubecl path selector under ruling410 and locks registry0.22.0; retained manifest is0.22.0-pre.4 with a standalone workspace. cubecl-wgpu and cubek-reduce are also registry-selected at root; burn-remote/cubecl-runtime/cubek-matmul remain local. Independent Distillery probes still select cubek-reduce, so root retirement cannot establish every consumer retired. Root claims were checked against lock entries. No archived defect in this slice is labelled a current registry/runtime failure.'),
 ('Runtime erasure retains multiple placement identities','Tensor carries client, device, allocation handle and metadata; graph replay checks device.',
  [P+'src/backend.rs',P+'src/tensor/base.rs',P+'src/ops/base.rs'],
  'CubeGraph replay rejects wrong device. Public tensor constructors accept client/device separately; later same-device comparison checks device, while same_view checks full service identity. Placement coherence remains a caller invariant to trace, not a reproduced wrong-device operation. Sync barriers/device faults are explicitly distinct from per-buffer failed launches; profile NotMeasured becomes None and tokens close the opening stream. Checked pool alignment/caps delegate configuration resolution.'),
 ('Fusion registry is shared but callback lifetime still sits under one lock','Built-in and user providers share a single global registry sealed at startup.',
  [P+'src/fusion/registry.rs',P+'src/fusion.rs'],
  'Six original type-erased core tests pass in the CPU extraction. Duplicate/sealed/lookup semantics are explicit. Provider fusers/restore callbacks execute while the global mutex is held, leaving reentrant callback deadlock as a followup, not an executed failure. Provider name override versus serialized Operation::NAME alignment is not enforced here. Fallback wrapper delegates FallbackOp.execute and its failure propagation rather than creating another executor.'),
 ('Attention admission protects unsupported options before flash dispatch','Ordinary ModuleOps routes bias/softcap/custom scale through the shared fallback.',
  [P+'src/ops/module.rs',P+'src/kernel/attention.rs'],
  'Low-level flash helper ignores bias, but ordinary module admission intercepts it, so this audit does not report ordinary bias loss. InvalidConfig can fallback while other errors surface. Shape anchoring/tuning and fallback score materialization have separate performance/memory implications. Causal/options tune-key omission needs cache/strategy evidence before a correctness claim.'),
 ('Aliasing and layout use shared primitives with explicit identity checks','Same-view detection binds allocation offsets/stream/service; tensor metadata carries storage tiling.',
  [P+'src/kernel/binary.rs',P+'src/kernel/memory_order.rs',P+'src/tensor/base.rs',P+'src/ops/base.rs'],
  'Exact same-view binary operands use one binding with fresh output; in-place broadcast requires nonoverlap and mutability. Memory-order votes prefer full-output operands, normalize singleton strides and restore logical order; qparams bypass. Reshape delegates shared reshape_action; tiled tensors are refused by row arguments until untile. Original layout/alias/tensor tests are source-read, not run. Zero-shape subtraction in is_nonoverlapping and tiled logical/physical rank interactions remain followups.'),
 ('Convolution has shared kernels but repeated tuning/admission logic','Forward/data/weight gradients retain separate strategy grids and alignment helpers.',
  [P+'src/kernel/conv/forward.rs',P+'src/kernel/conv/data_backward.rs',P+'src/kernel/conv/weight_backward.rs',P+'src/kernel/conv/im2col.rs'],
  'stride_align/pow2_factor logic appears across three paths; grouped/depthwise/bias restrictions and asymmetric-padding preparation are explicit. Test cfg forces batches_per_run1, so unit builds do not exercise production chunk selection. Q424 identifies a shared flattening authority gap. Existing performance comments are historical, not measured in this audit.'),
 ('Transpose-convolution copies inverse bounds across dimensional variants','Direct2D and3D kernels own separate input-range formulas.',
  [P+'src/kernel/conv/conv_transpose2d/transpose_direct.rs',P+'src/kernel/conv/conv_transpose3d.rs'],
  'Q425 qualifies only the extracted2D signed arithmetic. Related3D arithmetic subtracts in unsigned form before casting; backend usize lowering/wrapping must be established before extrapolation. No GPU convolution test executed. Shared inverse-bounds authority would reduce dimensional drift.'),
 ('Numerical capability declarations need backend witnesses','Deform gradients, CTC, grid sampling and FFT each have explicit restrictions and local helpers.',
  [P+'src/kernel/conv/deform_conv2d.rs',P+'src/kernel/ctc.rs',P+'src/kernel/grid_sample.rs',P+'src/kernel/fft.rs',P+'src/ops/module.rs'],
  'Source families fully read; some paths split into modules. Deform atomic Intrinsic/CAS dtype/capability dispatch is explicit; compare_exchange_weak old-value retry semantics need DSL contract evidence. CTC alpha/alpha-beta duplicate forward recurrence but share helpers, finite sentinel/threshold limits are documented, and shared-memory caps are not all-device proof. has_ctc_loss_backward=true means implementation exists, not all edge cases tested. Grid bilinear-only restriction is explicit; last grid component read+1 versus actual stride and empty extents remain followups. FFT pads to power of two; arbitrary-n semantics and explicit rfft n0 handling need contract review.'),
 ('Boolean representation comment and bit operation require an invariant','bool cast masks the low bit while its comment describes any nonzero representation.',
  [P+'src/kernel/cast.rs',P+'src/ops/bool_tensor.rs'],
  'Even nonzero integer patterns would mask to0, but ordinary BoolStore producers may constrain representation to0/1. This is a source invariant/documentation followup, not a demonstrated valid tensor failure. Bool storage uses U8/U32, native bool unsupported. Flex32/F32 early-return metadata identity also needs real caller contract before promotion.'),
 ('Index reductions mix serial, atomic and per-update ownership','Scatter/select/slice share wrappers but different update variants own distinct race behavior.',
  [P+'src/kernel/index/',P+'src/kernel/mask/',P+'src/ops/int_tensor.rs',P+'src/ops/tensor.rs'],
  'Ordinary scatter serializes each row; additive atomic capability/rounding nondeterminism is documented and disabled on CPU. scatter_nd per-update non-atomic read/modify/write makes duplicate-index reductions a followup requiring public duplicate-index semantics. Gather/select unchecked values and select_assign checked write need exact backend contract. Aligned slices keep metadata; unaligned or stepped slices materialize. Mask auto chooses mutability/broadcast while explicit strategy requires caller admission.'),
 ('Empty reductions have explicit identities while empty matmul conflates axes','Reduce owns empty identities; matmul autotune short-circuit bypasses the reduction identity.',
  [P+'src/kernel/reduce/',P+'src/kernel/matmul/tune/base.rs',P+'src/kernel/matmul/utils.rs',P+'src/ops/numeric.rs'],
  'Reduce assigns Sum/Any0, Prod/All1, floatMeanNaN and returns errors for identity-less extrema/args. Q426 proves K0 matmul returns nonempty output untouched. Fused values/indices share reduction grid with separate timing; accumulator length is in the cache key for TopK shared-memory differences. Dense axis-fold admission restores logical metadata.'),
 ('Autotune configuration/cache identity remains distributed','Matmul/reduce/sum/attention families construct their own keys, groups and bounds.',
  [P+'src/kernel/matmul/tune/',P+'src/kernel/reduce/',P+'src/tune_key.rs'],
  'Reduce OnceLock autotune_level contrasts fresh config reads for bounds; initial Full configuration can avoid registering bounds later needed after config change, pending actual config lifetime contract. Whole-sum candidates share sum_one_shot name across parameters; name cache identity needs TunableSet contract before a collision finding. Matmul tile capability correctly applies the same promoted register dtypes as selection. Separate local candidate grids are not by themselves proof of split durable authority.'),
 ('Quantization allocation regions remain one lifetime through offsets','Values/scales/global metadata are separately described but parameters reconstruct handles from the value allocation.',
  [P+'src/ops/qtensor.rs',P+'src/tensor/quantization.rs',P+'src/tensor/base.rs',P+'src/ops/base.rs'],
  'Two-level global scale is asserted F32; byte splitting and scale-region subtraction are checked. QFloat scheme packing/native restrictions and unsupported reshape/reblock paths fail explicitly. Whole-allocation transfer keeps scales beyond value handle bounds and adjusts end offsets for destination alignment. Same-runtime wgpu and cross-runtime fallback host-stage where peer transport is unavailable. Existing two-device ignored and dual-runtime tests fully read but not run. Shared-allocation guarantee of create/empty_tensors remains an underlying runtime contract to verify.'),
 ('Transaction batching assumes one placement and at least one binding','tr_execute chooses first tensor client and batches all handles through it.',
  [P+'src/ops/transaction.rs',P+'src/ops/distributed.rs'],
  'An empty primitive unwraps None and mixed-device handles are not checked in this implementation; higher-level transaction admission must be traced before declaring a reachable bug. Read failures map to ExecutionError; no success on those errors. Distributed all_reduce makes mutable contiguous storage or copies through zeros+add, then delegates runtime collective completion to sync_collective; queueing and completion remain different lifetimes.'),
 ('Template cache comment claims text identity not visible in wrapper ID','SourceKernel emits precompiled source while id delegates to caller-provided KernelSource.id.',
  [P+'src/template/base.rs',P+'src/template/source.rs'],
  'define comment says template text rides in kernel name to invalidate edited templates, but definition holds root/empty body and id simply forwards caller ID; source is separate. Runtime compilation cache may add source identity, so record a false-local-claim/cache followup pending active runtime read, not a demonstrated stale GPU kernel. Language is declared and mismatch described as launch error. Template substitution is nonrecursive and metadata serialization downcasts shapes/strides toU32 without bounds checks.'),
 ('Cumulative operations are explicitly quadratic fallback kernels','Float/int cumulative calls share one generic kernel with operation families.',
  [P+'src/ops/numeric.rs',P+'src/ops/tensor.rs',P+'src/ops/int_tensor.rs'],
  'O(n^2) reads and intended small-shape scope are documented. Max/min implement NaN propagation through float-only IR checks, rather than pretending generic Numeric supports float IsNan. This is a clearly bounded fallback, not evidence of a shared optimized scan. No cumulative GPU test was run.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1282):
 ident=f'A{number}';ids.append(ident);authority['entries'].append({'id':ident,'domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,'state':'retained supplier source fully read; root selection and execution scopes explicit','verification':NAME})
notes['review_followups'].extend([
 {'area':'Retained backend reactivation hazards','detail':'Q424 batch-gap flattening; Q425 direct2D transpose bounds; Q426 K0 matmul identity. Retired copy is not root-selected. Do not patch archive as if that repairs registry backend. Verify upstream selected code only if tracing an active consumer requires it.','verification':NAME},
 {'area':'Supplier ownership/lifetime followups','detail':'Trace runtime cache source identity, callback-under-lock reentrancy, service/device placement coherence, combined quantized allocation guarantees, runtime config freeze, duplicate-index reductions, transaction admission and backend-specific atomic/layout contracts. These source suspicions lack full runtime/GPU witnesses and remain separate from reproduced CPU findings.','verification':NAME}])
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==3135 and stats['read_chars']==35935608 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'support/patches/burn-cubecl/','files':119,'chars':695238,'verification':NAME,'basis_commit':head,'qualification':'All retained source/manifests read.18 CPU cases:6 original registry core tests,4 predicate/address cases,3 exact2D range cases,5 exact early-return cases with explicit doubles. No full backend/GPU build or current registry failure claim. All3738 live source membership/hashes reconfirmed.'})
next_read={'path':'support/patches/burn-remote/Cargo.toml','start':0,'reason':'Continue selected Burn remote patch, then runtime/WGPU/Cubek supplier source and remaining support/workflow source; preserve full denominator.'}
pending='No audit-owned process running.18 CPU cases pass with stated extraction/double limits.3135/3738 source files read;603 remain. All3738 live source hashes/membership unchanged. No production edit, generated binary, isolated target/home/worktree or native/GPU action.'
notes['current_area']='Retained Burn CubeCL complete; selected Burn remote next';notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','classification_of_previous_turn':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,*probes])
addition=f'''

### Retained Burn backend source review, 2026-10-17

All119 files in support/patches/burn-cubecl are fully read (695,238 characters). Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters;603 source files remain. Full live source membership and all3,738 source hashes remain unchanged. This inventory verification awards no unread-source credit.

Q424: pointwise convolution flattening accepts a batch gap that changes addresses from[0,1,4,5] to[0,1,2,3]. Q425: the direct2D transpose-convolution integer bounds omit a valid origin contribution for a unit kernel with stride3. Q426: zero-inner-dimension matmul returns a nonempty supplied output untouched rather than zeroing its empty dot products.18 CPU cases pass:6 original registry tests,4 layout predicate/address cases,3 extracted range cases and5 extracted early-return cases. The latter use explicit scalar/tensor/allocator doubles. No CubeCL lowering, GPU execution, full backend build or actual uninitialized-device read occurred.

These are retained-copy reactivation hazards. Root selects registry burn-cubecl0.22.0, while this standalone archived copy is0.22.0-pre.4 and its path selector is retired. Current registry failures are not established by these tests. Root also selects registry cubecl-wgpu/cubek-reduce, but independent Distillery probes still select the retained reduce patch. A1282-A1297 track this consumer distinction plus placement identities, callback/lock lifetime, layout admission, repeated convolution bounds/tuning, quantized allocation offsets, reduction identities, transaction placement and source-cache identity. The template wrapper comment claims template text enters its cache identity; that is not visible in its forwarded ID and remains a runtime-cache followup. Shared reshape/fallback/error propagation and explicit unsupported cases are recorded alongside risks, without converting source-read tests into executed receipts.

All prior report bytes, protected source/shared work and Git index are preserved. Four exact owned executable/PDB outputs were verified and removed across the two probes. No audit process, isolated target/home/worktree or GPU/native run remains. Evidence stays below100MB. Local/upstream remain {head}/{upstream}. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [layout/range probe](../../../support/receipts/authority-success-audit/{probes[0]}) and [matmul probe](../../../support/receipts/authority-success-audit/{probes[1]}). Continue selected burn-remote source.
'''
doc.write_bytes(prefix+addition.encode())
for name,value in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/name).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,'prior_receipt':{'path':'support_qualifier_review_receipt.json','sha256':sha(HERE/'support_qualifier_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'external_context_sha256':external,
 'new_findings':[v[0] for v in new],'authority_entries':ids,'full_repository_source_read':False,'next_read':next_read,'pending_verification':pending,'audit_authored_production_edits_this_pass':False,
 'all_inventory_live_membership_and_hashes_verified':{'files':len(i),'added':[],'removed':[],'changed':[]},
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in dict.fromkeys(evidence)},'git_index_sha256':sha(index),'retained_isolation':[],'shared_generated_cache':old['shared_generated_cache'],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'qualification':'Progress only. All119 retained Burn files695238chars read.18 CPU cases with stated extraction/double limits. Retained copy is root-unselected; no registry/GPU failure claim. Four owned exe/PDB outputs removed. Full source inventory unchanged.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':record['new_findings'],'authority_entries':ids,'budget_bytes':budget,'next_read':next_read},indent=2))
