"""Read-only Canvas test review and bounded presentation diagnostics."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib
import json
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n,v): (HERE / n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old=load('pictograph_signal_review_receipt.json')
probe=load('pictograph_height_probe_receipt.json')
assert probe['test_exit']==0 and probe['tests']==3 and probe['generated_output_removed']
assert 'test result: ok. 3 passed; 0 failed;' in (HERE / 'pictograph_height_probe_output.log').read_text()
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT / p)==h for p,h in preserved.items())
assert all(sha(ROOT / p)==h for p,h in probe['source_sha256'].items())
notes,authority=load('review_notes.json'),load('authority_map.json')
findings=[
 {'id':'Q104','status':'documented from fully read implementation/test source and native diagnostics; not repaired',
  'title':'Degree lift raises the node body and picking but leaves its face on the ground',
  'source':'crates/canvas/pictograph/src/canvas/frame.rs:gnode updates; face_layer; view.rs:face_rect_at; input.rs:node picking; tests/face_on_body.rs',
  'evidence':'A native hub with three Surface UserGrouped edges has48px degree height. CPU Livery body readback moves24/48/96px at zoom0.5/1/2; node_face_rect stays on the ground anchor at every zoom. Height-off controls align. At zoom1 clicking the generated face center selects nothing while clicking the raised body center selects the node. The face painter uses the same unlifted face_rect_at helper; body placement and picking independently subtract the lift. Existing face/body tests use one edgeless node with no lift, so their alignment assertion does not cover this feature combination.',
  'scope':'Two bounded native CPU Canvas frame, body hit-readback, face rectangle and public pointer observations. Generated Surface topology intentionally avoids Resource degree omission Q091. The actual CPU paint construction shares the inspected helper; raster pixels, headed window, host DOM mode, favicon decode and other representations were not executed.',
  'verification':'pictograph_geometry_review_receipt.json','diagnostic':'pictograph_height_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal forbids production edits','size':'contained arithmetic repair; broader shared display-pose ownership documented',
     'reason':'Apply the resolved visual lift to face placement as well as the body. A broader single display-pose reader for body, texture/vector face, accessibility and picking needs separate qualification rather than adding another independent coordinate adjustment.'}},
 {'id':'Q105','status':'documented from fully read implementation/test source and native diagnostic; not repaired',
  'title':'Suggestion body shrinks while its derived face retains full-size geometry',
  'source':'crates/canvas/pictograph/src/canvas/frame.rs:gnode updates; face_layer; view.rs:face_rect_at; grammar_view.rs:grammar_face_scale; tests/grammar_view.rs',
  'evidence':'With graph grammar enabled and a member admitted as a suggestion, CPU body hit-readback width is16.2px but its derived face rectangle remains25.92px, wider than its body. With grammar off, the same fixture restores the intended0.72 inset. Body width uses node_size*grammar_face_scale, while face_rect_at and the derived face decoder height use node_size alone. The existing grammar test asserts suggestion CSS width but never compares its face geometry; the face/body alignment test uses grammar off.',
  'scope':'One bounded native CPU Canvas frame/geometry observation with suggestion admission and grammar-off control. Height was disabled to isolate this from Q104. Actual raster pixels, GPU, headed UI, host DOM presentation and favicon image decode were not executed. This demonstrates derived face overflow, not all suggestion interaction semantics.',
  'verification':'pictograph_geometry_review_receipt.json','diagnostic':'pictograph_height_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal forbids production edits','size':'contained footprint repair; coordinate with Q104',
     'reason':'Give the face geometry and decode/LOD input the same resolved display size the body uses. Keep intentional minimum pointer targets distinct from visual size; do not fix overflow by changing graph truth or physical collider policy.'}},
]
upsert(notes['findings'],findings,'id')
def entry(number,domain,claim,implementations,assessment):
    return {'id':f'A{number:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'partial Canvas test source review; three native CPU observations; no production repair',
        'verification':'pictograph_geometry_review_receipt.json'}
entries=[
 entry(123,'Visual node center across layers','The vector/favicon face sits on the same upright node body at every zoom.',
    ['GNODE frame placement subtracts height*zoom','Input picking independently compensates height','face_rect_at projects the ground anchor without lift','Derived and favicon paths call that helper','Alignment test uses a single degree-zero node'],
    'Q104 proves two display-coordinate implementations diverge under degree lift. The shared face painter/readback helper is real but does not make it shared with body placement. Native pointer miss follows from the split; headed pixels remain unverified.'),
 entry(124,'Resolved display footprint versus raw node size','Faces are inset within the resolved body; suggestions become small marks.',
    ['Body width/height multiply grammar_face_scale','Suggestion scale makes a36px node16.2px','Face rectangle and decode height omit that scale','Grammar test checks body CSS width only','Face/body test keeps grammar disabled'],
    'Q105 exposes separate size authorities for one visual object. Centralizing the face helper alone did not centralize the resolved size. Visual footprint, physical collider size and intentional hit slop should remain explicit separate policies.'),
 entry(125,'Density receipts, probes and stop semantics','Density quality bars are checked over specified generated starts at ruled thresholds.',
    ['density.rs has a quick default sample check and ignored release assertions','density_admits has print-only candidate probes and a separate assertion receipt','Tolerance/quantization and gen-200 floor changes are explicit','density_probe/wander directly install alternate laws and print traces','Some helpers reach a tick cap without an explicit terminal result'],
    'These sources do not justify calling the whole ignored corpus default-suite coverage. Direct experimental injection is labelled probe work rather than a second production law owner. A printed at-stop/cap reading needs its terminal condition disclosed; no failed ruled quality bar is asserted from this source-only pass.'),
 entry(126,'Fold/source time and intentional live-view separation','Folding and source scrubbing do not rewrite live graph facts.',
    ['Fold tests compare persisted facts after normalizing wall-clock snapshot timestamp','Fold/expand preserve member placement and undo records','SourceTimeCanvas retains the live Canvas while presenting a disposable history snapshot','Arrangement loop checks source cursor, strategy and a shared member slot','Full view copying is a different refresh-live path'],
    'Read tests actually exercise graph-preservation comparisons, not merely counts. The arrangement loop does not establish full history-presentation fidelity, all hide/fold/dynamics state, or refresh_live behavior. Existing reconstruction candidate remains qualified, not a proven graph-truth rewrite.'),
 entry(127,'Gloss lens membership versus main Canvas scope','Gloss owns an independent layout, selection scope and cache.',
    ['Gloss tests set explicit independent positions','Selection-scoped geometry excludes induced outside edges','Scope contributes to its own cache key','Importance data are shared across main and gloss consumers','Main-scope/fold membership is not asserted by these gloss tests'],
    'An independent lens is intentional. Do not label a differently scoped Gloss a duplicate graph authority or a leak solely because it can show the whole graph. Main overlay/minimap scope candidate still needs its actual product contract.'),
 entry(128,'Meaning sharing and physics evidence boundaries','One Meaning snapshot and one cluster computation feed several consumers.',
    ['Tests compare the same partition across channels and Columns','Actual Louvain run counter has a second-registry positive control','Content edits rerun while structure-only edits keep Meaning','Sparse and dense lexical pairs are compared bit-for-bit','Sliced completion is bounded and compared to whole execution','GPU/model and timing receipts are separately gated'],
    'The reviewed code includes substantial non-self-referential evidence design. These package tests were read only. Grouped helpers return at rest or6000 frames but line output always says rest; that is a qualified diagnostic-label problem, not proof a valid product run falsely claimed convergence.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Grouped receipt cap labelled as rest','source':'crates/canvas/pictograph/src/canvas/tests/grouped.rs:rest_on; line',
  'note':'rest_on exits at settle_count>0 OR6000 frames and stores only frame count, while line prints rest afterN frames. No run at the cap was freshly executed. Preserve an explicit ended-by-rest/cap value before treating printed diagnostics as convergence evidence. Separation assertions themselves still compare actual resulting positions.'},
 {'title':'Accessibility geometry under display lift and suggestion scale','source':'crates/canvas/pictograph/src/canvas/reader.rs:item_rect; view.rs; frame.rs',
  'note':'The described-item rectangle uses projected ground position and raw node_size; Q104/Q105 prove display center/scale divergence elsewhere. Accessibility bounds were not directly exercised here. A shared resolved display-pose seam should cover it after native qualification, without conflating it with durable node position or physical collider geometry.'},
],'title')
upsert(notes['source_qualifications'],[
 {'area':'Canvas test source coverage through Meaning fixture','note':'Twenty additional Canvas test files fully read,248295 chars, from bounded_scenes.rs through meaning_topics.rs. Includes Density probes/receipts, raw-term/catalog declarations, fold/source-time, framing, Gloss, grammar, grouped roles, home-at-budget-end, drag, elapsed stepping and Meaning controls. Package suites and ignored receipts were not executed.'},
 {'area':'Canvas visual geometry native evidence','note':'Three generated native CPU observations pass for Q104/Q105, using unchanged hash-verified Cargo artifacts. Body location comes from CPU Livery hit readback; face geometry is the same helper the painter calls. Zoom, height-off and grammar-off controls included. Pointer test executes public input APIs. No raster pixels, headed UI, GPU, host DOM mode or favicon decode.'},
 {'area':'Test-only execution paths are not production admission evidence','note':'Density helpers and finite-budget Canvas probe deliberately call private Physics setters/step paths. The source explicitly labels the finite-budget Canvas branch as not reached by a public path. Their presence alone does not establish a production pause bypass beyond the separately executed Q097/Q102 observations. AtRest home-return work remains intentional.'},
],'area')
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p,v in inventory.items() if p.startswith('crates/canvas/pictograph/') and coverage.get(p,{}).get('sha256')==v['sha256'] and coverage[p]['through']==v['chars'])
added=[p for p in complete if p not in old['source_sha256']]
assert len(complete)==134 and len(added)==20 and sum(inventory[p]['chars'] for p in added)==248295
assert all(sha(ROOT / p)==inventory[p]['sha256'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE / 'read_coverage.py'),'status'],cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/canvas/pictograph/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Pictograph Canvas tests read through meaning_topics; remaining tests pending'
notes['next_area']='crates/canvas/pictograph'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
    'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['pictograph_geometry_review_receipt.json'],'next_read':next_read,
    'diagnostic_outcomes':'Q104/Q105 reproduced by three bounded native CPU geometry/input observations;20 further test files read. No production repair or whole-package suite claim.',
    'workspace':'Stable C:/t/cargo-targets/mere reused; owned probe exe/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation']={'previous_turn':'progress: signal completion and Q102/Q103','classification_of_previous_turn':'progress',
    'production_edits_this_turn':False,'coverage_checkpoint':stats,'next_read':next_read,
    'user_instruction':'Continue full-source audit; preserve concurrent work; do not change production code.',
    'coverage_note':'Twenty additional files and248295 chars. Mere726/3352; Pictograph134/163. Objective remains active.'}
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes)
save('authority_map.json',authority)
doc='design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition='''

### Read-only Canvas geometry and test review, 2026-10-16

Twenty more Canvas test files are fully read, from bounded_scenes through the
Meaning topic fixture:248,295 characters. Mere coverage is726/3352 files and
Pictograph134/163; the full-source goal remains active. Resume at mosaic.rs.
Production is unchanged and no whole-package or ignored release suite was run.

Q104 and Q105 expose separate pose/size calculations for the same displayed node.
With three Surface UserGrouped neighbors, degree height is48px. The CPU body
hit-readback rises24/48/96px at zoom0.5/1/2, but the derived face rectangle stays
on its ground anchor. Height-off controls align. A public pointer click at the
face center misses, while the raised body center selects the node. Body and
picking subtract visual lift; the shared face_rect_at painter/readback helper
does not. The existing face/body test uses a single edgeless unlifted node.
Resource degree omission Q091 was deliberately excluded from this fixture.

With height disabled, graph grammar makes an admitted suggestion body16.2px
wide while its derived face remains25.92px, wider than the body. Turning the
grammar off restores the intended0.72 inset. Body sizing multiplies the grammar
scale; face geometry and decoder/LOD height read the raw node size. The grammar
test checks CSS body width, and the alignment test keeps grammar disabled, so
neither covers their combination. Three native CPU observations reproduce both
defects with zoom/height/grammar controls. Actual raster pixels, headed UI,
GPU, host DOM mode and favicon decoding were not exercised.

The contained corrections belong in resolved visual center/size admission.
A broader shared display-pose seam covering CSS bodies, vector/image faces,
accessibility and input should be documented separately so the audit does not
become a coordinate-system rewrite. Physical collider size and deliberate
minimum hit targets are separate policies. reader.rs currently computes its
rectangle from the ground anchor/raw size too, but accessibility bounds were
not directly probed in this pass and remain a qualified follow-up.

The read test source contains meaningful controls as well as weaker evidence.
Density default checks differ from ignored release bars; its convergence/wander
and candidate-overlay probes are labelled print-only. Explicit thresholds and
rounding rules are recorded, not silently inferred from test names. Experimental
helpers directly install alternate laws, and the finite-budget Canvas home test
explicitly states that no current public path reaches its injected case. This
is not another production scheduling authority. AtRest home return remains an
intentional exception to ordinary settle-trigger admission.

Fold tests compare persisted graph facts after normalizing only the snapshot
wall-clock stamp. Source scrubbing retains a separate live Canvas and checks
cursor/strategy/shared-member placement across arrangements; that does not prove
full refresh-live presentation copying. Gloss has an explicitly independent
lens/selection scope and cache. Meaning tests compare partitions across consumers,
instrument a second-registry run as a control, compare sparse/dense lexical pairs,
and bound sliced completion against whole execution. These tests were read only,
not freshly passed. GPU/model and timing receipts remain separately gated.

One qualified test-output issue remains: grouped rest_on can hit6000 frames,
but its printed line always says rest afterN frames. No capped case was executed;
the separation assertions still compare actual positions. Record the terminal
reason before treating that label as convergence evidence. This is separate
from the two reproduced product geometry defects.

The native probe reused unchanged source/artifact/build-log hashes and its owned
executable/PDB were removed. No isolated target, Cargo home or worktree remains.
Exact evidence and preservation hashes are in the
[geometry audit receipt](../../../support/receipts/authority-success-audit/pictograph_geometry_review_receipt.json).
'''
destination=ROOT / doc
before=destination.read_bytes()
assert b'### Read-only Canvas geometry and test review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE / 'pictograph_geometry_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence=['pictograph_height_probe.rs','run_pictograph_height_probe.py','pictograph_height_probe_build.log',
    'pictograph_height_probe_output.log','pictograph_height_probe_receipt.json','pictograph_geometry_review_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Read-only partial Canvas test and visual geometry source review; Mere incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_pictograph_files':complete,
    'added_complete_files':added,'next_read':next_read,'source_sha256':{p:sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes':preserved,'diagnostic_tests':3,'diagnostic_exit':0,
    'package_tests':'source read only; not run','reused_library_build':{'path':'pictograph_library_build.jsonl','sha256':probe['library_build_log_sha256']},
    'generated_output_removed':probe['generated_output_removed'],'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE / p) for p in evidence},
    'limits':['Full-source objective incomplete; no production repair.',
        'Body geometry is CPU Livery hit readback, not raster-pixel measurement.',
        'No package/release suite, headed UI, GPU, host DOM mode, favicon decode, network or store executed.',
        'Historical view fidelity, scoped overlays and grouped capped-rest label remain qualified follow-ups.',
        'Prior Q073 client byte-identity failure remains open; broader previous gates were not rerun.']}
assert all(sha(ROOT / p)==h for p,h in preserved.items())
save('pictograph_geometry_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'added_files':len(added),'findings':len(notes['findings']),
    'authority_entries':len(authority['entries']),'next_read':next_read}))
