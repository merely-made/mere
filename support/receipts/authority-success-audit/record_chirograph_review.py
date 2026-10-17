"""Record Chirograph source coverage and evidence; no production edits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
notes,authority=load('review_notes.json'),load('authority_map.json')
prior=load('client_review_receipt.json')
probe=load('chirograph_review_probe_receipt.json')
preserved=prior['preserved_production_and_test_hashes']
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert probe['test_exit']==0 and probe['tests']==5 and probe['generated_native_output_removed']
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert load('pending_read.json')==[]
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
paths=[p for p in inventory if p.startswith('crates/chirograph/')]
assert len(paths)==5
assert all(sha(ROOT/p)==inventory[p]['sha256']==coverage[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in paths)
scene_paths=[p for p in inventory if p.startswith('crates/cambium/scenes/sceno/')]
scene_complete=all(sha(ROOT/p)==inventory[p]['sha256']==coverage.get(p,{}).get('sha256') and coverage[p]['through']==inventory[p]['chars'] for p in scene_paths)
if scene_complete:
    assert len(scene_paths)==9
    sceno_log=(HERE/'sceno_unit_test_output.jsonl').read_text(encoding='utf-8-sig',errors='replace')
    assert 'test result: ok. 37 passed; 0 failed;' in sceno_log
unitlog=(HERE/'chirograph_unit_test_output.jsonl').read_text(encoding='utf-8-sig',errors='replace')
assert 'test result: ok. 52 passed; 0 failed;' in unitlog
assert any(json.loads(s).get('reason')=='build-finished' and json.loads(s).get('success') for s in unitlog.splitlines() if s.startswith('{'))
findings=[
 {'id':'Q067','status':'public capture/live-client admission mismatch reproduced; not repaired',
  'title':'Validated captures retain presentation metadata the live client rejects',
  'source':'crates/chirograph/src/capture.rs:147','related_sources':['crates/chirograph/src/capture.rs:160','crates/chirograph/src/capture.rs:204','crates/graphshell/graphshell-client/src/lib.rs:validate_presentation'],
  'evidence':'CaptureWire validation checks the scene, version and optional score/authority generation, but not the presentation sidecar. With valid scene tables, absent-item bindings, duplicate bindings and codec/capability conflicts each pass V1 from_projection, validation, encoding, decoding and resource-availability checks, then V2 lifting/encoding/decoding/availability checks. Current ClientState rejects each corresponding ProjectionSnapshot. A valid presentation control is accepted by both.',
  'scope':'Current public native APIs on generated fixtures. Targeted current repository search found capture APIs only in Chirograph and client tests, not a production capture consumer; external consumer adoption is not established. No persisted user artifact or renderer crash executed. Resource availability callback is a caller assertion, not a byte verifier; this diagnostic supplies the actually present resource hash.',
  'verification':'chirograph_review_receipt.json','diagnostic':'chirograph_review_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal excludes production edits','reason':'Move carrier-independent presentation invariants to a shared validator and use it for captures and live snapshots. Keep session/cache policy checks outside immutable capture identity where intended. Exercise both versions with malformed sidecars rather than only valid-table round trips.'}},
 {'id':'Q068','status':'public frame/native codec divergence reproduced; not repaired',
  'title':'Frame line assembly silently rewrites invalid UTF-8 that native readers reject',
  'source':'crates/chirograph/src/line.rs:73','related_sources':['crates/chirograph/src/line.rs:91','crates/graphshell/graphshell-host/src/line.rs:LineReader::next','ports/graphshell/src/webrtc_browser.rs:360'],
  'evidence':'LineAssembler uses String::from_utf8_lossy for complete lines and unterminated tails. A CarrierOutput descriptor containing an invalid byte in its label is rewritten with U+FFFD and successfully decoded as a typed protocol response. Public native LineReader returns InvalidData for those bytes. A valid multibyte UTF-8 value split between pushes is reconstructed correctly and agrees with the native reader. Targeted BrowserSession source passes frame bytes directly through this assembler.',
  'scope':'Current public Chirograph assembler/decode and native graphshell-host LineReader over Tokio duplex. Browser path is source-traced, not run in wasm or a real channel. Demonstrates divergent malformed-byte admission and altered data, not a content-hash bypass, authority forgery or valid UTF-8 fragmentation defect.',
  'verification':'chirograph_review_receipt.json','diagnostic':'chirograph_review_probe_receipt.json',
  'repair_triage':{'disposition':'documented; active goal excludes production edits','reason':'Make complete-line and partial-tail UTF-8 decoding fallible, with the same rejection policy as native readers. Thread the error through browser frame callers and test both invalid-byte locations and split valid codepoints. API/error propagation needs a coordinated pass.'}}
]
entries=[
 {'id':'A064','domain':'Portable capture versus live projection presentation admission','claim':'Captures validate before persistence/disclosure and carry the presentation sidecar beside the same scene tables.',
  'implementations':['CaptureWire genuinely shares V1/V2 scene/version/resource logic','Live ClientState privately checks binding identity and codec/capability invariants','Capture validation does not call those presentation checks','Resource availability callback confirms supplied hash membership only'],
  'assessment':'Q067 is a second, weaker presentation admission path around real shared table validation. The optional score/authority generation agreement is not source authenticity verification.','state':'three malformed presentation classes accepted in both capture versions and refused live','verification':'chirograph_review_receipt.json'},
 {'id':'A065','domain':'Shared NDJSON spelling versus byte-to-string admission','claim':'Chirograph line is the codec every end shares.',
  'implementations':['encode/decode functions genuinely shared across carriers','Native Lines reader performs strict UTF-8 conversion before decode','Frame LineAssembler replaces invalid bytes before the same decode','BrowserSession source uses LineAssembler for both complete lines and EOF tails'],
  'assessment':'Q068 shows that sharing text JSON decode does not unify the preceding byte admission rule. Valid fragmented UTF-8 is supported; malformed bytes receive divergent treatment.','state':'typed frame acceptance versus native InvalidData reproduced','verification':'chirograph_review_receipt.json'},
 {'id':'A066','domain':'Chunk generator bounds, assembly integrity and caller request ownership','claim':'Sequential chunks are verified before the assembled resource is returned.',
  'implementations':['from_slice clamps generated chunks and never produces non-final empty progress','ResourceAssembly checks offsets, per-frame digest and final whole digest','Native app private wrapper separately checks requested session/hash, stable bounded total, decoded size and progress','Wrapper has explicit eight-fault source tests'],
  'assessment':'Public assembly alone accepts a scope/resource switch with a shared prefix and a valid final hash, and accepts empty non-final chunks. This is an API boundary qualification, not a current app-client bypass: its wrapper explicitly rejects these cases. Sharing the crypto helper does not make request ownership/bounds universal.','state':'raw assembly boundary reproduced; app wrapper protections source-confirmed, not rerun here','verification':'chirograph_review_receipt.json'},
 {'id':'A067','domain':'Projection acknowledgment versus durable persistence ownership','claim':'ProjectionAck is described as the last revision a client has durably applied.',
  'implementations':['ClientState acknowledgement reads the current in-memory scene','Live applies and resume recovery need no ProjectionStore','MemoryOnly is the default cache policy','Explicit persist_session has a separate injected store acknowledgment'],
  'assessment':'The acknowledgment is real client application progress but is not a durable-store receipt. The protocol prose should name this distinction. No restart-loss bug, recovery persistence requirement or falsely successful real store operation is inferred from the word alone.','state':'source-confirmed terminology overclaim; consumer durability contract not inferred','verification':'chirograph_review_receipt.json'}
]
def upsert(items,added,key):
    for x in added:
        at=next((i for i,held in enumerate(items) if held.get(key)==x[key]),None)
        if at is None: items.append(x)
        else: items[at]=x
upsert(notes['findings'],findings,'id')
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'SessionOpen promises negotiation while native loop echoes the requested version','source':'crates/chirograph/src/lib.rs:SessionOpen','note':'Native session_loop builds SessionOpened with open.version and descriptor without an observed version/capability compatibility gate. Chirograph unit test named an_open_negotiates_capabilities checks only serialization/no principal. Trace endpoint selection, product consumers and version conventions before claiming a whole-protocol incompatibility bypass.'},
 {'title':'Snapshot and diff protocol-version admission differ in the live client','source':'crates/graphshell/graphshell-client/src/lib.rs:apply_snapshot','note':'Client apply_diff rejects an unsupported major; apply_snapshot validates scene/presentation but does not compare ProtocolVersion. Chirograph DTO decoding itself accepts the numeric version. Qualify version negotiation/caller checks before promoting future-version acceptance to a current product compatibility failure.'},
 {'title':'Portable captures omit live cache policy including memory-only constraints','source':'crates/chirograph/src/capture.rs:ProjectionCaptureV1::from_projection','note':'Capture intentionally omits session/cache policy and from_projection assumes an already-authorized snapshot. It can structurally capture an EditableText presentation. Current targeted capture call-site search found tests only; do not infer a real unauthorized export. A future capture producer must explicitly own retention/export and resource disclosure decisions.'},
 {'title':'Single coordinated selection silently chooses the lexically first producer','source':'crates/chirograph/src/lib.rs:CoordinatedSelection::targets_for','note':'Single permits multiple map clauses through set or serde, then values().next chooses lexical producer order; role is not used. This may be an explicit one-view precondition, not a bug. Inspect actual coordinated-selection consumers and design ruling before imposing a different resolution policy.'},
 {'title':'LineAssembler buffer has no admitted byte ceiling and rescans partial prefixes','source':'crates/chirograph/src/line.rs:LineAssembler','note':'push is unbounded and next_line searches from buffer start each call; repeated fragments with no newline can repeatedly scan the accumulated prefix. Native stream adapters also lack a generic assembly ceiling. Track caller/frame limits before a remote memory/CPU exhaustion claim; sharing a text codec does not provide a total message budget.'}
],'title')
upsert(notes['source_qualifications'],[
 {'area':'Chirograph complete source','note':'All five source/manifests fully read. CaptureWire shares versioned byte/address/resource plumbing; V2 strict shapes and score-generation checks are real; lifting V1 deliberately changes bytes/address. Titulus is an in-crate module again, not another current crate. Content references verify declared length and BLAKE3 then SHA-256. Action form composition shares exact choice/schema validation with client drafts.'},
 {'area':'Protocol DTOs and source authority','note':'Selections, opaque live-view references, clipping artifacts and consent receipts are data, not authenticated principals or source truth. SessionOpen deliberately carries no identity; endpoint policy remains responsible for typed intent validation and consent. Do not call a constructible confirmed boolean an authorization bypass without a composing endpoint trace.'},
 {'area':'Chirograph package execution','note':'All 52 existing native unit tests passed. Five standalone public-API diagnostics then passed with valid controls and explicit chunk-wrapper qualification. No doctest, wasm, actual browser/DTLS, durable-user-artifact or external-consumer gate was run.'}
],'area')
if scene_complete:
    upsert(notes['source_qualifications'],[
      {'area':'Sceno complete source','note':'All nine source/manifests fully read; 37 native unit tests passed. Fold validates live unique members, stand-in/root membership and disjoint active folds. FoldEffect.is_shown explicitly combines own visibility and fold hiding. Members retain host order. Score reading refuses newer versions and upgrades pre-v5 Anchored to Seeded before stamping the reader version; this is an implemented migration, not a TODO.'},
      {'area':'Scene primitives and source boundaries','note':'SourceRef remains an opaque adapter/id; source/instance identity is structural. LinkKey is explicitly a derived source-pair view token, not a minted graph pair entity. Size2 non-negativity is explicitly convention, not enforced. Dense Scene.to_world returns None on dangling/cyclic chains; Scene::new installs world while derived Default does not. Do not treat raw DTO construction or primitive arithmetic as whole-snapshot validation.'}
    ],'area')
    upsert(notes['candidates'],[
      {'title':'Score arrangement round-trip test omits the three new subdivision families','source':'crates/cambium/scenes/sceno/src/score.rs:every_arrangement_round_trips','note':'The test says every family but its manual vector omits Partition, Setscape and Mosaic from the current Arrangement enum. Subdivision types have no local tests. The compiler may still enforce solver exhaustiveness elsewhere; no serialization or solver defect is established merely from this missing coverage.'},
      {'title':'Canonical routed LinkKey ordering is only applied by its constructor','source':'crates/cambium/scenes/sceno/src/link.rs:LinkKey','note':'new sorts endpoints, but public fields and derived Deserialize can admit first > second; stable_id serializes fields as held. Trace Scenotime relation validation and actual importing/grouping consumers before alleging duplicate rendered pair identity. The constructor and its ambiguous-delimiter/stratum tests are real protections for callers that use it.'},
      {'title':'Capture re-encoding can migrate a nested score before computing a fresh content address','source':'crates/cambium/scenes/sceno/src/score.rs:Score::read','note':'A pre-v5 nested score reads as version 5 with old Anchored converted to Seeded; capture content_address re-encodes its current typed value. V1 capture lifting explicitly documents a new address, while nested score normalization inside V2 is not similarly discussed. No consumer mismatch reproduced; distinguish raw transfer-byte hash from normalized typed artifact identity before proposing a policy.'}
    ],'title')
    notes['candidates']=[dict(x,note=x['note']+' Subsequent full Sceno reading establishes that overlapping/nested active folds are forbidden and Fold.members is explicitly ordered; do not promote an invalid-overlap conflict as a valid-input bug.') if x.get('title')=='Frozen fold disclosure uses table order and can resurrect own-hidden members' and 'Subsequent full Sceno' not in x['note'] else x for x in notes['candidates']]
if 'crates/chirograph' not in notes['completed_areas']: notes['completed_areas'].append('crates/chirograph')
if scene_complete and 'crates/cambium/scenes/sceno' not in notes['completed_areas']: notes['completed_areas'].append('crates/cambium/scenes/sceno')
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
next_prefix='crates/cambium/scenes/scenotime/' if scene_complete else 'crates/cambium/scenes/sceno/'
next_path=next(p for p in sorted(inventory) if p.startswith(next_prefix) and (coverage.get(p,{}).get('sha256')!=inventory[p]['sha256'] or coverage.get(p,{}).get('through',0)<inventory[p]['chars']))
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Chirograph and shared Graphshell source fully read; '+('Sceno complete, sparse scene validation/diffs next' if scene_complete else 'scene data/validation ownership next')
notes['next_area']=next_prefix+', then remaining Mere source'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':['chirograph_review_receipt.json'],'diagnostic_outcomes':'52 Chirograph unit tests passed; five bounded public-API diagnostics passed while confirming two findings and qualifying chunk bounds ownership.',
 'next_read':next_read,'workspace':'Reused C:/t/cargo-targets/mere. Standalone probe executable/PDB removed. No isolated Cargo home, target or worktree.'}
if scene_complete: notes['checkpoint']['diagnostic_outcomes']+=' All 37 Sceno unit tests passed after complete source reading.'
notes['audit_continuation']={'previous_turn':'progress: completed client source, Q062-Q066 and eight bounded diagnostics','classification_of_previous_turn':'progress','production_edits_this_turn':False,
 'completed_areas_this_pass':['crates/chirograph/']+(['crates/cambium/scenes/sceno/'] if scene_complete else []),'coverage_checkpoint':stats,'next_read':next_read,'user_instruction':'Continue full source audit; preserve concurrent work and do not change production code.',
 'coverage_note':'Chirograph source complete; whole repository incomplete. Diagnostics confirm gaps and boundaries, not fixes or remote acceptance.'}
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'chirograph_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
doc='design_docs/mere_docs/technical_architecture/2026-10-15_transport_agnosticism_note.md'
evidence=['chirograph_unit_test_output.jsonl','chirograph_client_probe_build_output.jsonl','chirograph_native_host_build_output.jsonl','chirograph_review_probe.rs','run_chirograph_review_probe.py','chirograph_review_probe_build_output.txt','chirograph_review_probe_output.txt','chirograph_review_probe_receipt.json']
if scene_complete: evidence.append('sceno_unit_test_output.jsonl')
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'previous_turn_classification':'progress',
 'kind':'Complete Chirograph source review, package tests and bounded public-API diagnostics; no repairs','findings':[x['id'] for x in findings],'authority_entries':[x['id'] for x in entries],
 'complete_areas':['crates/chirograph/']+(['crates/cambium/scenes/sceno/'] if scene_complete else []),'source_files':len(paths),'source_chars':sum(inventory[p]['chars'] for p in paths),'coverage':stats,'next_read':next_read,'production_edits_this_pass':False,
 'preserved_production_and_test_hashes':preserved,'source_sha256':{p:sha(ROOT/p) for p in paths},'governing_document_sha256':{doc:sha(ROOT/doc)},
 'unit_tests':{'exit':0,'passed':52,'failed':0,'command':'cargo test -p chirograph --lib --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build --message-format=json'},
 'native_builds':{'client':0,'host_native':0},'diagnostic_tests':{'passed':5,'failed':0},
 'coverage_limits':['No whole-source completion claim.','No browser, network/DTLS, wasm, installed resident or durable-user-capture gate.','Targeted port excerpts/searches do not count as full source coverage.','App-client private chunk wrapper source read; its existing tests not rerun this pass.'],
 'generated_output_removed':probe['generated_native_output_removed'],'retained_isolation':[],'diff_check':{'exit':diff.returncode,'log':'chirograph_review_diff_check.txt'},'evidence_sha256':{p:sha(HERE/p) for p in evidence}}
if scene_complete:
    receipt['sceno']={'source_files':len(scene_paths),'source_chars':sum(inventory[p]['chars'] for p in scene_paths),'source_sha256':{p:sha(ROOT/p) for p in scene_paths},'unit_tests':{'exit':0,'passed':37,'failed':0}}
save('chirograph_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authorities':len(authority['entries']),'next_read':next_read,'preserved_hashes':len(preserved),'diff_check':diff.returncode}))
