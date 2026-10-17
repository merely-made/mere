"""Read-only signal review checkpoint, preserving the previous Pictograph pass."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))

def save(n,v):
    (HERE / n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')

def upsert(items,values,key):
    for value in values:
        at = next((i for i,item in enumerate(items) if item.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old = load('pictograph_review_receipt.json')
probe = load('pictograph_signal_probe_receipt.json')
assert probe['tests']==2 and probe['test_exit']==0 and probe['generated_output_removed']
assert 'test result: ok. 2 passed; 0 failed;' in (HERE / 'pictograph_signal_probe_output.log').read_text()
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT / p)==h for p,h in preserved.items())
assert all(sha(ROOT / p)==h for p,h in probe['source_sha256'].items())
notes,authority = load('review_notes.json'),load('authority_map.json')
findings = [
 {'id':'Q102','status':'documented from fully read source and native observation; not repaired',
  'title':'An empty affinity result restarts settling on every unchanged frame',
  'source':'crates/canvas/pictograph/src/canvas/strategy.rs:sync_affinity_force; canvas/tests/affinity.rs:empty_content_affinity_is_authoritative_but_inert; canvas/frame.rs',
  'evidence':'ContentOnly plus Some(empty) installs no force, so affinity_force_installed remains false. The unchanged-input fast path requires that boolean true; every frame therefore sends set_affinity_force(None) and settle(180). Native paused Canvas keeps exactly180 ticks after each of400 unchanged CPU frames and advances its clock to6.666667 seconds. A disabled parallel control stays idle; disabling affinity lets the remaining budget drain. The existing inert test asserts force count0 only and never checks runtime work or repeated settle demand.',
  'scope':'Actual public Canvas configuration and CPU frame loop, generated edgeless graph, bounded400-frame controls. Positions may be masked by pause; no headed body movement, measured power/latency, GPU or offloaded actor behavior claimed. Q097 proved a one-shot size pause bypass; this separately proves repeated empty-result scheduling and the inert-test evidence gap.',
  'verification':'pictograph_signal_review_receipt.json','diagnostic':'pictograph_signal_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal forbids production edits','size':'contained state/gate repair',
     'reason':'Track a processed empty result independently of whether a force exists; only actual contributing-input changes should reinstall. Route ordinary settle through the pause gate. A legitimate empty result is successful computation, not an uninitialized installation.'}},
 {'id':'Q103','status':'documented from fully read source and native observation; not repaired',
  'title':'Edgeless spectral producer supplies uniterated coordinates instead of empty structure',
  'source':'crates/canvas/pictograph/src/signals/producers.rs:spectral_coords; smallest_laplacian_eigenvectors; signals/registry.rs:spectral',
  'evidence':'The producer promises empty coordinates for an edgeless graph so EmbeddingFallback can handle absent structure. When c=2*max_degree is0, the eigenvector loop skips iteration but still pushes its normalized cosine start. Native three-node edgeless graph returns three finite nonzero coordinates identical at0 and200 iterations. Registry returns/caches those same values once. Empty and single-node controls return empty; a linked Resource fixture returns coordinates and shared projected degree importance works.',
  'scope':'Native public signal producer and registry, using valid generated graphs. Main arrangement dispatch reads coords.spectral in inspected source; the actual SpectralAdapter, headed fallback pixels and arbitrary symmetric graphs were not executed here. Do not generalize to incorrect eigenvectors on every connected graph.',
  'verification':'pictograph_signal_review_receipt.json','diagnostic':'pictograph_signal_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal forbids production edits','size':'contained degenerate-input repair',
     'reason':'Handle zero total structure before publishing starting vectors, honoring the documented empty result so the existing arrangement fallback owns placement. Preserve intentional iteration-count behavior for structured inputs instead of inventing fallback positions in the producer.'}},
]
upsert(notes['findings'],findings,'id')

def entry(number,domain,claim,implementations,assessment):
    return {'id':f'A{number:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'all signal source read; partial Pictograph package; two native observations; no production repair',
        'verification':'pictograph_signal_review_receipt.json'}

entries = [
 entry(117,'Empty computed state versus installed effect','Empty authoritative affinity is inert and installs once per real change.',
    ['Registry correctly caches an empty affinity result','Canvas uses force existence as its processed-input gate','Empty result leaves installed=false','Frame consequently resets settle demand every time','The inert test checks only zero force count'],
    'Q102 separates successful empty computation from absent effect. Central producer reuse does not guarantee correct consumer lifecycle. Repeated ordinary settle also bypasses the pause gate from Q097.'),
 entry(118,'Spectral facts versus arrangement fallback','Missing structure returns no coordinate, allowing EmbeddingFallback to decide placement.',
    ['Edgeless spectral branch skips matrix iteration','It still publishes normalized initial vectors','Registry caches them as valid coords.spectral','SpectralAdapter declares its own existing fallback for missing coordinates'],
    'Q103 is false supplied fact rather than missing registry sharing. Fix producer admission and retain fallback ownership in the arrangement; do not add another independent edgeless layout.'),
 entry(119,'Actual channel computation and cache ownership','One registry serves one graph and computes each fact when its key changes.',
    ['Slot caches the current revision/parameter combination','Structure, visits and URL grouping have separate keys','Physics facts use caller-supplied PhysicsViewKey','Off-thread community results share the same held slot','Graph swap explicitly requires resetting the registry'],
    'This centralization is implemented. One-graph ownership and truthful PhysicsViewKey are caller preconditions, not runtime identity checks. offer_community accepts a tagged value; Canvas performs stale-result admission. Do not label a caller violating those documented preconditions an executed host bug.'),
 entry(120,'Shared topology algorithms and incomplete consumers','The TopologyView seam lets the same structural algorithms serve kernel and generic Chartulary graphs.',
    ['Kernel TopologyView reads projected Surface/Resource buckets','Chartulary implementation uses generic graph adjacency','Degree/betweenness/Louvain/articulation/Jaccard all consume this seam','Signal tests cover alias lift, mixed stores and historical parity','Face size/height still call the narrower Graph method'],
    'Shared algorithm reuse is real. New native Resource degree control confirms importance sees the edge that Q091 face encoding ignores. importance.rs still claims exact agreement with size-by-degree; this wording is stale after partial consumer migration. Physics mass and arrangement weight remain intentionally distinct.'),
 entry(121,'Test evidence and canonical record preservation','Arrangement goldens and binding receipts qualify exact reuse and persistence.',
    ['Goldens pin historical Surface placements and compare live Resource placements separately','The one-ULP negative control proves the hash can detect changes','Binding serializes typed readback and typed given specs for byte comparison','Role tests include moved seeded versus fixed/returning controls','Source counts and force-count-only cases have narrower evidence'],
    'The read tests are not all self-referential; several use literal goldens and effect controls. Typed canonical serialization equality is not raw input JSON whitespace/order preservation. These package tests were read, not freshly run, so they are not this pass acceptance receipts.'),
 entry(122,'Face derivation and renderer lowering','Address-derived IconVG faces use stable versioned arithmetic and caller palettes; Vello errors remain explicit.',
    ['Derivation uses fixed FNV/SplitMix integer choices and exact small coordinates','Two LOD arms and palette slots are encoded through Emblem','Literal digest fixtures pin mapping','VelloSink retains a gradient lowering error until into_scene','decode propagates Emblem and lowering errors before returning Graphic'],
    'Actual shared rendering type and refusal propagation were read. No evidence of a second face authority or silently accepted gradient error in this module. Vello/GPU feature and cross-target deterministic execution were not exercised in this CPU-only pass.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['source_qualifications'],[
 {'area':'Pictograph signal review completion','note':'All11 source/test files in src/signals.rs and src/signals/ are now fully read:120164 chars. Three earlier Canvas test files and two more Canvas test files, plus lib.rs and vello.rs, were also read this pass. Eighteen additional files,247196 chars; package114/163 complete,1656824 chars. Canvas/test-directory and integration-test remainder still pending.'},
 {'area':'Signal pass native verification','note':'Two bounded native CPU observation tests pass against prior Cargo-built libraries whose build-log, artifact and previously read source hashes were checked unchanged. Empty affinity repeats180-tick scheduling through400 frames; spectral edgeless output is nonzero and iteration-independent. Controls are included; no whole package suite, new Cargo build, GPU feature, headed UI, store or network executed.'},
 {'area':'Signal and binding evidence narrowing','note':'Resource/Surface arrangement parity, cache invalidation, aliases, fixed goldens, role effects and explicit binding refusal are present in read tests. Test source is evidence of coverage design, not a fresh passing result. The prior spec byte-preservation candidate is narrowed to typed canonical reserialization; raw wire JSON preservation is not demonstrated or claimed by these tests.'},
],'area')
upsert(notes['ruled_out'],[
 {'title':'Signal registry is not merely a duplicate of Cartography placement','source':'crates/canvas/pictograph/src/signals/registry.rs; producers.rs; physics.rs',
  'note':'Registry centralizes source facts and exports narrow signal contracts; Scenomise/Cartography own placement. Graph-free physics facts serve Canvas, board and group subgraphs. The seam is implemented reuse, with documented one-graph/reset and revision-key preconditions. No independent durable graph authority introduced.'},
],'title')

inventory,coverage = load('source_inventory.json')['files'],load('read_coverage.json')
complete = sorted(p for p,v in inventory.items() if p.startswith('crates/canvas/pictograph/') and coverage.get(p,{}).get('sha256')==v['sha256'] and coverage[p]['through']==v['chars'])
added = [p for p in complete if p not in old['source_sha256']]
signals = [p for p in complete if p.startswith('crates/canvas/pictograph/src/signals')]
assert len(complete)==114 and len(added)==18 and len(signals)==11
assert sum(inventory[p]['chars'] for p in added)==247196
assert all(sha(ROOT / p)==inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python',str(HERE / 'read_coverage.py'),'status'],cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p,v in sorted(inventory.items()) if p.startswith('crates/canvas/pictograph/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read = {'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
area = 'crates/canvas/pictograph/src/signals'
if area not in notes['completed_areas']: notes['completed_areas'].append(area)
notes['current_area']='Pictograph signals complete; Canvas tests and integration tests remain'
notes['next_area']='crates/canvas/pictograph'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
    'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['pictograph_signal_review_receipt.json'],'next_read':next_read,
    'diagnostic_outcomes':'Q102/Q103 reproduced by two bounded CPU observations. All11 signal files read; no production changes or whole-package suite claim.',
    'workspace':'Existing stable C:/t/cargo-targets/mere reused; owned exe/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation']={'previous_turn':'progress: Pictograph96 files and Q091-Q101',
    'classification_of_previous_turn':'progress','production_edits_this_turn':False,
    'coverage_checkpoint':stats,'next_read':next_read,
    'user_instruction':'Continue full-source audit; preserve concurrent work; do not change production code.',
    'coverage_note':'Eighteen additional files and247196 chars. Mere706/3352 and Pictograph114/163 fully read; objective remains active.'}
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes)
save('authority_map.json',authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only Pictograph signal review, 2026-10-16

All11 signal source/test files are fully read, alongside the Canvas test root,
affinity/goldens/roles/binding tests, face generator and Vello bridge. This adds
18 files and247,196 characters. Mere coverage is706/3352; Pictograph114/163.
The remaining Canvas and integration tests are still pending, beginning with
bounded_scenes.rs. The full-source audit remains active; production is unchanged.

Q102 proves that an empty authoritative affinity is not inert at runtime.
sync_affinity_force treats force existence as the processed-input flag. An empty
result has no force, so every unchanged frame repeats set(None) and settle(180).
A native paused Canvas keeps exactly180 ticks through400 CPU frames and advances
its field clock to6.666667 seconds. A disabled control stays idle; disabling
affinity allows the remaining budget to drain. The existing inert test checks
only zero force count, which does not cover scheduling. This extends Q097's
one-shot pause bypass with a distinct repeated empty-result lifecycle defect.
The contained repair is a processed-input state independent of force presence,
plus the existing pause gate; the active goal leaves production unchanged.

Q103 proves that three edgeless nodes disclose nonzero spectral coordinates
identical at0 and200 iterations. The c=0 branch skips matrix iteration but
publishes its normalized starting vectors, contradicting the documented empty
map that lets EmbeddingFallback handle absent structure. The registry correctly
caches the incorrect supplied fact once. Empty/single-node controls return
empty; a linked Resource fixture supplies coordinates and correct shared degree
importance. No actual SpectralAdapter fallback pixels or general connected-graph
eigenvector failure was executed. Handle zero structure before publishing facts,
then retain placement fallback ownership in the existing arrangement layer.

Registry centralization is real: structure/visit/URL-grouping and parameters
key separate source facts, and graph-free physics producers serve Canvas, board
and grouped subgraphs. One registry serves one graph; callers must reset on a
swap and provide truthful physics-view revisions. These are explicit caller
preconditions, not hidden runtime identity checks. No graph-identity misuse
was promoted into a product bug. The generic TopologyView and mixed-store/alias
parity tests are implemented sharing. importance.rs still claims exact agreement
with size-by-degree, which Q091's narrower Surface consumer disproves.

The reviewed tests include actual literal placement/digest goldens, one-ULP
hash controls, Resource/Surface parity, refusal cases and seeded/pinned/returning
effect controls. They were read, not freshly run. Binding's byte comparison is
between canonical serializations of typed specs; it does not prove raw input
JSON whitespace/order preservation. The versioned face generator uses shared
Emblem encoding and palette references. VelloSink retains lowering errors and
decode propagates them before returning a Graphic; no silent gradient-success
defect was found in that read module. Vello/GPU execution remains unqualified.

Two native CPU observations pass against the existing Cargo-built libraries,
with unchanged source, build-log and artifact hashes verified. Passing means
the defects were reproduced, not repaired. No package suite, new Cargo build,
headed UI, GPU, network or store was exercised. The owned executable/PDB were
removed; no isolated output/home/worktree remains.

Exact reading coverage, hashes and observations are in the
[signal audit receipt](../../../support/receipts/authority-success-audit/pictograph_signal_review_receipt.json).
'''
destination=ROOT / doc
before=destination.read_bytes()
assert b'### Read-only Pictograph signal review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE / 'pictograph_signal_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence=['pictograph_signal_probe.rs','run_pictograph_signal_probe.py','pictograph_signal_probe_build.log',
    'pictograph_signal_probe_output.log','pictograph_signal_probe_receipt.json','pictograph_signal_review_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Read-only complete signal review; partial Pictograph and Mere source review',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_pictograph_files':complete,
    'added_complete_files':added,'complete_signal_files':signals,'next_read':next_read,
    'source_sha256':{p:sha(ROOT / p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'diagnostic_tests':2,'diagnostic_exit':0,'package_tests':'read only; not run',
    'reused_library_build':{'path':'pictograph_library_build.jsonl','sha256':probe['library_build_log_sha256']},
    'generated_output_removed':probe['generated_output_removed'],'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE / p) for p in evidence},
    'limits':['Full-source objective incomplete; no production repair.',
        'No whole-package suite, headed UI, GPU/Vello feature, network or store execution.',
        'Typed canonical serialization equality is not raw JSON byte identity.',
        'Prior Q073 client byte-identity failure remains open and broader previous suites were not rerun.']}
assert all(sha(ROOT / p)==h for p,h in preserved.items())
save('pictograph_signal_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'added_files':len(added),'findings':len(notes['findings']),
    'authority_entries':len(authority['entries']),'next_read':next_read}))
