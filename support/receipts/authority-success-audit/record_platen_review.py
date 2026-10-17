"""Record complete Platen/shared Workbench reading and bounded successful gates."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None:items.append(value)
        else:items[at]=value
def log(name):
    raw=(HERE/name).read_bytes();return raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
old=load('forme_review_receipt.json');probe=load('platen_probe_receipt.json')
assert probe['test_exit']==0 and probe['tests']==9 and probe['generated_output_removed']
assert '50 passed; 0 failed' in log('platen_library_suite.log')
assert '22 passed; 0 failed' in log('shared_workbench_suite.log')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith(('crates/platen/platen/','crates/cambium/workbench/')))
assert len(complete)==14 and sum(inventory[p]['chars'] for p in complete)==175764
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==989 and stats['read_chars']==13765477
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(n,title,source,evidence,scope,size,repair):
    return {'id':f'Q{n:03}','title':title,'source':source,
        'status':'reproduced against current Cargo-built public library; documented, not repaired',
        'evidence':evidence,'scope':scope,'verification':'platen_review_receipt.json','diagnostic':'platen_probe_receipt.json',
        'repair_triage':{'disposition':'documented; active goal forbids production edits','size':size,'reason':repair}}
findings=[
    finding(132,'Fraction sanitizers retain negative shares and overflow finite positive shares into zero',
        'crates/platen/platen/src/projection_geometry.rs:TreeGeometry::sanitize; workbench/tree.rs:renormalize and set_fractions_at',
        'sanitize([-1,2]) leaves[-1,2] despite detecting invalidity. Both sanitize and live set_weights([f32::MAX,f32::MAX]) produce[0,0] because their f32 sum overflows. Live canonical_roundtrips then becomes false and debug to_persisted_json panics before returning its advertised Result. Ordinary[3,1] becomes[0.75,0.25] and serializes.',
        'Native no-default-feature Platen public data/API paths, caught debug assertion. No release panic claim: release skips debug assertion. Graphshell load validates finite positive shares/total; its live edit builds and validates a candidate, so those host paths reject the invalid result rather than proving a UI crash.',
        'shared stable normalization/admission; contained algorithm repair',
        'Use one finite positive normalization policy across geometry and live state, with overflow-safe accumulation/scaling and refusal or repair before publication. Keep valid fractions bit-exact as promised. Do not use a debug assertion as the only fallible persistence boundary.'),
    finding(133,'Opening into an unknown target removes the already-open member before reporting failure',
        'crates/platen/platen/src/workbench.rs:TileLayout::open_in_slot_of',
        'Starting with stack[1,2], active2, open_in_slot_of(2,99) returns false and leaves only1. It detaches2 before checking target99. move_to_slot_of with the same unknown target preserves both; a valid target control succeeds. The documented caller fallback can reopen the member but cannot recover its prior grouping/active state automatically.',
        'Native ordinary public TileLayout API, with guarded move and valid-target controls. No current external production caller of open_in_slot_of was found by repository symbol search; the generic API failure is proven, current product trigger not.',
        'contained target admission before detach',
        'Resolve/check the target before mutating, or stage a candidate and publish only on success. Preserve full prior structure/active selection on refusal rather than reconstructing it with a fallback open.'),
    finding(134,'Tree projection ignores a stacking edge when a later tile connects two existing slots',
        'crates/platen/platen/src/tree_projection.rs:project_tree and stacked_slot',
        'Root members inserted1,2,3 with StackedWith edges1-3 and2-3 project into two slots;2 and3 are not together even though their explicit stacking edge exists. All three tiles survive. Insertion-aligned chain1-2,2-3 projects one slot. The greedy first-known-slot lookup never merges previously placed components.',
        'Native public Arrangement/project_tree with ordinary authored relations and chain control. No Group/nested projection support inferred; those remain explicitly deferred. No rendered host or grouping editor execution.',
        'contained equivalence-component grouping with stable order',
        'Resolve stacking groups independent of visit/edge order before emitting slots, preserving member occurrence identities and first-authored order. Do not merge separate appearances solely by graph member UUID.'),
    finding(135,'Geometry sidecar overrides semantic membership and tab grouping without a disagreement outcome',
        'crates/platen/platen/src/workbench/bridge.rs:TileLayout::from_arrangement, canonical_roundtrips',
        'Same arrangement with1 StackedWith2 yields one two-tab cell without geometry. Geometry containing only1 silently omits2; geometry splitting1/2 yields two cells despite the stacking relation. That split layout still returns canonical_roundtrips=true because it derives a new matching arrangement from itself rather than comparing with the supplied authority. Reconciliation only removes geometry members absent from the arrangement.',
        'Native source/sidecar mismatch at public constructor. Graphshell validate enforces exact membership-set equality and therefore rejects the missing-member case before its load path; it does not compare stacking-edge semantics. No current product archive or live editor mismatch executed.',
        'authority reconciliation design and consumer migration may exceed one-hour diversion',
        'Specify and enforce which source owns membership/grouping versus geometry refinement. Reject/report mismatches or reconcile from the authored arrangement with explicit coverage. A self-roundtrip receipt only proves reconstruction of its own output, not agreement with an independent authority.'),
    finding(136,'Closing an inactive earlier tab changes the active member in both tiling reducers',
        'crates/platen/platen/src/workbench/tree.rs:Stack::remove; crates/cambium/workbench/lib.rs:TileTree::remove_tile',
        'For stack[1,2,3] with active2, closing inactive1 leaves index1 active, now selecting3, in both independent implementations. A last-active3 control remains3 through their clamp path. Both removal functions clamp only out-of-range indices instead of preserving the selected tile identity.',
        'Two native public reducers exercised. Graphshell event Closed calls layout.close_tile, and missing-access masking/load also closes members, source-traced only; no hosted event or renderer execution.',
        'contained selected-identity preservation in each owner, then shared algorithm review',
        'Preserve the active tile by identity when removing another tab; choose documented neighbor only when the active tile itself closes. Align both reducer implementations or share the neutral operation with appropriate ID mapping.'),
    finding(137,'Loaded duplicate geometry passes the canonical check and produces duplicate accessibility IDs',
        'crates/platen/platen/src/workbench/bridge.rs:from_arrangement and canonical_roundtrips; accessibility.rs:project_tile_layout',
        'Arrangement containing1 plus geometry stack[1,1] loads as two tabs, passes canonical_roundtrips, and project_tile_layout emits duplicate AccessKit NodeIds because tab paths use the repeated member. One close removes only one copy. Normal open_stack([1,1]) deduplicates to one.',
        'Native geometry admission and AccessKit DTO construction only, no AccessKit consumer/adapter panic executed. Graphshell host validate rejects duplicate access IDs before loading this state, so that existing boundary limits product impact.',
        'contained loaded-layout uniqueness admission; occurrence-aware generalization broader',
        'Enforce the same one-member-per-cell policy on import as ordinary mutators, with explicit refusal/diagnostics. If repeats are meant to be real appearances, introduce occurrence IDs consistently across geometry, editing and a11y rather than reusing member IDs.'),
    finding(138,'Self-edge drag deletes a tile while the shared Workbench reports Applied',
        'crates/cambium/workbench/lib.rs:TileTree::drag and Workbench::apply',
        'From stack[1,2], dragging1 to its own Right edge returns WorkbenchOutcome::Applied and leaves only2. Target validation succeeds before removing1, then split_at_tile cannot find the removed target; its false return is ignored and drag returns true. Unknown-target control is Unchanged with both tiles; other-tile edge control succeeds preserving both.',
        'Native shared public Workbench wrapper and reducer. No platform event producer or current host execution. Platen self-split APIs guard this case and Graphshell uses split_out for own-cell edges, so this is a separate shared reducer defect rather than a demonstrated Graphshell deletion.',
        'contained self-target policy and atomic move correction',
        'Resolve an alternate target within the original stack for an intended split-out, or refuse self-edge drops without mutation. Publish Applied only if insertion succeeded and tile custody is retained. Keep the explicit outside tear-out Effect boundary.'),
    finding(139,'Floating geometry promises nonfinite sanitization but host-area values can panic or leak NaN',
        'crates/cambium/workbench/float.rs:FloatingTile::resolve and constrained_extent',
        'Finite400x400 area resolves normally. A NaN host x origin panics in f32::clamp; a NaN host width returns NaN x. Nonfinite relative x/width are sanitized as the existing policy promises. The host area bypasses the finite helper even though resolve documents nonfinite inputs reading as zero.',
        'Native CPU geometry with explicit nonfinite host input and finite/relative-input controls. Does not prove ordinary host viewport is nonfinite or any GPU/renderer failure. No finite-range behavior regression claimed.',
        'contained area precondition/finite admission clarification',
        'Apply the documented finite policy to the full area before bounds/clamp arithmetic, or expose/refuse an invalid area explicitly and document the precondition. Keep host size/layout policy outside this neutral contract.'),
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'Platen10/10 and shared Workbench4/4 read;50+22 original tests and9 native probes',
        'verification':'platen_review_receipt.json'}
entries=[
    entry(225,'Arrangement authority versus layout cache','Forme owns semantic membership/grouping; geometry only refines presentation.',
        ['Forme Arrangement is geometry-free', 'Platen Pane is mutable working tree',
         'Bridge rebuilds only from geometry when supplied and intersects with referenced member set'],
        'Q135 proves competing membership/grouping meanings and limits the single-authority claim. It is not enough to say Pane is a cache while it derives the next canonical arrangement from itself. Graphshell adds membership checks; independent group agreement remains unqualified.'),
    entry(226,'Roundtrip qualification versus independent truth','Canonical roundtrips proves the cache is derivable.',
        ['to_arrangement emits both the semantic source and full geometry from one Pane',
         'canonical_roundtrips compares only reconstructed root with that Pane',
         'Random gesture test checks600 seeds against the same paired encoder/decoder'],
        'Useful reconstruction evidence, freshly passing, not proof that geometry agrees with an independently authored arrangement or that every mutator preserves intent. Q132/Q135/Q137 show what that receipt misses. Projection mode is intentionally reset to Tree, not part of its root equality.'),
    entry(227,'Fraction normalization authorities','Fractions remain usable and canonical after load/edit.',
        ['TreeGeometry sanitize detects invalidity but divides negative shares by total',
         'Platen Pane renormalize independently repeats f32-sum division',
         'Shared Workbench normalize is a third copy; raw event setter leaves share policy host-owned',
         'Graphshell has separate finite/positive/total validation'],
        'Q132 proves numeric faults in Platen loaded/live normalizers. Share policy differs across layers rather than one common admission. Do not infer a host UI panic from a library debug assert; Graphshell validates a staged candidate. Stable normalization plus explicit owner policy is a broader unification seam.'),
    entry(228,'Stack relations versus incremental grouping','StackedWith joins tiles into tab stacks.',
        ['project_tree places root appearances in MemberOf insertion order',
         'stacked_slot chooses the first previously emitted neighbor slot',
         'No merge when a late appearance connects existing slots'],
        'Q134 violates an explicit stacking edge; source order becomes a hidden grouping authority. Unsupported Groups/Portal/Supernode are explicitly outside v1 and are not separately called bugs.'),
    entry(229,'Failure and tile custody','A placement refusal must leave an existing tile recoverable.',
        ['Platen open_in_slot_of detaches before target admission',
         'Other Platen moves guard both held members',
         'Shared TileTree drag validates target but self-target vanishes after removal'],
        'Q133 and Q138 prove two distinct incomplete admission paths. A precheck alone is not sufficient if the target is the moved object. Typed Applied/Unchanged also needs actual postconditions. Outside tear-out remains an honest host effect that retains custody.'),
    entry(230,'Tab selection across independent reducers','A close removes a tab, not an unrelated active selection.',
        ['Platen Stack remove stores active index',
         'Shared Workbench remove_tile separately stores/clamps active index'],
        'Q136 proves the same active-identity bug in both production libraries. Shared vocabulary does not make mutation logic shared. Product may correctly choose one authority yet inherit the copied defect.'),
    entry(231,'Occurrence identities versus member-keyed geometry','Forme supports repeated member occurrences and unbound tool tiles.',
        ['TilePlan retains ArrangementNodeId and optional GraphMemberId',
         'TileLayout stores GraphMemberId only, deduplicates normal opens',
         'build_arrangement collapses occurrences by member map',
         'flat_pane filters unbound tool tiles while tile_tree_from_plan can resolve them'],
        'These are different supported profiles, not a universal lossless Arrangement conversion. Q137 proves loaded state can bypass the restricted profile and duplicate a11y identity. Broader mirror/tool support requires occurrence-aware design and should not delay the audit.'),
    entry(232,'Accessibility owner and namespace','Platen owns its layout projection; Uxtree supplies the common AccessKit carrier.',
        ['Platen accessibility::project_tile_layout returns UxTree',
         'Slot-index and member paths under fixed tile-layout root',
         'Titles, bounds and actions explicitly host-owned'],
        'This supplies the actual Platen-domain projector that is absent from Uxtree itself, clarifying A221. A generic Group and active description are current behavior, not a full interactive accessible tab contract. Q137 proves duplicate IDs on duplicate input; multi-layout namespace/slot-reindex stability remains source-only scope.'),
    entry(233,'Document layout delegation','Platen composes opaque content rather than laying it out itself.',
        ['document-layout feature wraps actual document_canvas::layout_document',
         'Host supplies viewport and typography',
         'No local paragraph/font/layout algorithm copy'],
        'Concrete delegation is real reuse. Four document_scene tests were read but excluded by no-default-feature gate. Stale canvas_scene reference in prose does not establish a second active renderer. Document-canvas is next for full source review.'),
    entry(234,'Shared Workbench authority scope','Mere projects onto a neutral presentation contract, not a second graph writer.',
        ['TileTree includes host-usable raw reducer',
         'Workbench wrapper exposes typed outside tear-out effect',
         'Workspace holds tiled/float stations and accepts neutral commands',
         'Platen keeps separate GraphMemberId Pane algorithms'],
        'No graph/session/source relation authority is added by the neutral contract. There are two live reducer implementations with divergent semantics and copied bugs; neutral algorithm sharing merits separate design review. Current Graphshell uses Platen and guarded product mapping, not the raw TileTree reducer.'),
    entry(235,'Float station invariant versus restoration','Every tile lives at exactly one station.',
        ['Normal float/dock move custody only on successful destination insertion',
         'External insert refuses known IDs',
         'Workspace::new accepts a caller tree and tiled_mut explicitly transfers invariant responsibility',
         'Serde feature derives shape without general station validation'],
        'Fresh22-test gate covers nominal station travel and roundtrip. Mutable restoration/public raw data remain caller-owned admission; do not claim malicious/duplicate serde state is rejected automatically. Raise normalizes z, but float/insert increment top_z without immediate normalization; overflow under malformed restored z is only a source candidate.'),
    entry(236,'Float geometry admission','Nonfinite inputs read as zero before constraints.',
        ['Relative coordinates and min/max use finite helper',
         'Host area origin/extent go directly into coordinate/clamp arithmetic'],
        'Q139 narrows the false promise: host area remains unguarded. Native finite and bad-relative controls show the implemented policy for relative data is real; no normal finite-host defect inferred.'),
    entry(237,'Transient presentation versus canonical tree','Rails/drawers do not rewrite saved organization.',
        ['WorkbenchPresentation has stack anchors, optional drawer rectangles and override fractions',
         'apply changes controlled open state and focus-return request without a tree argument',
         'Invalid path/length/nonfinite/zero-only fractions fall back to canonical shares'],
        'This is actual separate transient authority, with fresh tests. A finite positive set can still overflow a downstream f32 sum; this method returns shares without normalization, so renderer consumption needs review rather than a new unexecuted numeric finding.'),
    entry(238,'Product-local guards and common contracts','A shared library receipt alone defines product admission.',
        ['Graphshell validates graph ID, bounds, exact membership and duplicate-free geometry',
         'Divider edit checks input then validates candidate before publication',
         'Closed event and missing-access masking invoke shared Platen close',
         'Own-edge drop maps to split_out rather than raw shared drag'],
        'Consumer source tracing limits Q132/Q135/Q137/Q138 product claims and connects Q136 to actual call sites. This is a guard survey, not full reading credit for forme_workspace or a hosted execution receipt. Do not discard working product guards when moving validation into shared owners.'),
    entry(239,'Documentation and current source owner','Workbench came from Genet and remains geometry-free.',
        ['Root workspace resolves workbench to Mere crates/cambium/workbench',
         'Manifest comment says unchanged Genet1.86 MSRV beside actual1.97.1',
         'Platen calls ratio-bearing split state geometry-free while separating pixel layout'],
        'Genet ancestry is provenance, current source owner is Mere. Ratios/nesting/active tabs are explicitly geometry sidecar state; geometry-free here means no pixels, not absence of layout authority. Use the precise distinction when claiming one arrangement authority.'),
]
upsert(authority['entries'],entries,'id')
old_entry=next(x for x in authority['entries'] if x['id']=='A221')
old_entry['followup']={'verification':'platen_review_receipt.json','note':'Platen accessibility.rs owns project_tile_layout returning shared UxTree. Uxtree itself has no Platen-specific function; this is a legitimate domain-owned projector, not evidence the Platen projection is absent.'}
upsert(notes['candidates'],[
    {'title':'Platen cache/semantic reconciliation and occurrence support need an explicit owner rule',
     'source':'crates/platen/platen/src/workbench/bridge.rs; tree_projection.rs; accessibility.rs',
     'note':'Q135/Q137 prove divergent semantic membership/grouping and duplicate geometry. TilePlan carries occurrence IDs/tools; TileLayout only member IDs. Broader migration and cache reconciliation can exceed the one-hour diversion limit. Keep finite/uniqueness/member guards already implemented in Graphshell.'},
    {'title':'Float restored stacking order and transient share consumers need admission qualification',
     'source':'crates/cambium/workbench/float.rs:top_z, float, insert_floating; presentation.rs:fractions',
     'note':'Serde can restore high z and ordinary insert/float increment top_z; no extreme-z probe. Transient finite shares do not validate finite sum; no renderer numeric probe. Source-only follow-up, not a demonstrated normal host failure.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/inker/document-canvas/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Platen10/10 and shared Workbench4/4 source review'
notes['next_area']='crates/inker/document-canvas'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['platen_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'Current Platen no-default-feature library build and50 tests pass; Workbench serde gate22 tests+0 doctests passes;9 generated native observations pass.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'14 additional complete files175764 chars; consumer excerpts not counted as full file reading.'}
upsert(notes['source_qualifications'],[{'area':'Complete Platen and shared Workbench source review',
    'note':'All14 inventoried files175764 chars read. Current Platen no-default-feature Cargo build/library suite50 tests pass. Workbench serde Cargo suite22 unit+0 doctests passes. Nine generated public native observations pass. Default document-layout4 tests, rendered product, GPU/AccessKit consumers and OS UI not run. Consumer guard excerpts source-traced, no full-file credit. Owned probe executable/PDB removed; stable shared target only.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only Platen and shared Workbench source review, 2026-10-16

All10 Platen and4 shared Workbench files are read,175,764 more characters.
Coverage is989/3352 files and13,765,477/36,854,820 characters. The full-source
goal remains active. Resume at crates/inker/document-canvas/Cargo.toml.
Production/concurrent source remains unchanged. Authority entries A225-A239
track the shared vocabulary, separate reducers and actual admission owners.

Q132: sanitization detects[-1,2] as invalid but leaves it negative after division
by total1. Both geometry sanitize and live set_weights turn finite positive
[f32::MAX,f32::MAX] into[0,0] because the f32 sum overflows. A live layout then
fails canonical_roundtrips and debug to_persisted_json panics before its Result.
Ordinary[3,1] becomes[0.75,0.25] and serializes. No release panic claimed.
Share normalization is independently repeated; use a stable finite policy
without nudging already-valid serialized fractions.

Q133: open_in_slot_of(held2,missing99) returns false but removes2 from its
existing stack first. The guarded move API preserves it on the same refusal;
valid-target control succeeds. Check/stage before detaching. No external
production caller of this specific open helper was found in repository search.

Q134: members inserted1,2,3 with StackedWith1-3 and2-3 produce two slots,
leaving explicit2-3 ungrouped. A chain aligned with insertion order produces
one slot. First-known-slot placement never merges earlier groups. Resolve
stack components independently of order while retaining occurrence identity.

Q135: geometry is a competing semantic authority. The same arrangement
1 StackedWith2 yields one cell without geometry, silently omits2 with a stale
one-member sidecar, and yields two cells with split geometry. The latter still
passes canonical_roundtrips: it re-derives both canonical inputs from its own
output, so it proves self-reconstruction, not agreement with the supplied
arrangement. Existing reconciliation removes extraneous members only. Explicit
authority/reconciliation and occurrence support may exceed the one-hour
diversion limit and are documented instead of redesigning during the audit.

Q136: closing inactive1 from[1,2,3] with active2 selects3 in BOTH Platen and
shared Workbench. Independent removal implementations clamp the old index
instead of preserving selected identity. The last-active control remains
correct through clamping. Graphshell Closed and missing-member masking use
Platen close, source-traced only; no hosted event execution was performed.

Q137: arrangement member1 plus geometry[1,1] admits two tabs, passes the
canonical check, emits duplicate AccessKit IDs, and leaves one copy after close.
Ordinary open_stack deduplicates the same input. Loaded-profile admission must
match mutation policy; real repeated appearances need occurrence IDs throughout.
No AccessKit consumer or adapter was executed on the malformed DTO.

Q138: shared Workbench drag1 onto its OWN Right edge returns Applied but
deletes1 from[1,2]. The target existed before removal, then split_at_tile cannot
find it; that failure is ignored. Unknown-target control leaves both and returns
Unchanged; other-tile edge control preserves both and succeeds. Preserve custody
and report insertion failure, or implement a properly anchored split-out.

Q139: FloatingTile::resolve says nonfinite inputs read as zero, but NaN host x
panics in clamp and NaN area width returns NaN x. Bad relative coordinates are
sanitized and finite-area control is correct. Apply or state the area admission
rule. This does not prove ordinary host viewports are invalid.

Current Graphshell has useful guards: load checks exact member set, duplicate
accesses and finite positive shares/total; edit validates a candidate before
publishing. This blocks Q132 invalid shares and Q137 duplicates in those paths,
and the missing-member Q135 case. It does not compare stacking-edge semantics.
Own-cell edge drop maps to split_out, protecting that host from Q138. These
are source-traced guard excerpts, not full-file reading credit or UI receipts.
Do not remove working guards when consolidating validation. The public-library
defects remain real without overstating their current product reach.

There are independent active reducers in Platen Pane and neutral Workbench
TileTree, plus the previously read parked GraphTree. Neutral typed vocabulary
is shared, mutation algorithms are not. The same selection bug survives in two
copies. Workbench's outside-drop Effect correctly keeps custody pending a host
decision, and Workspace station moves guard successful insertion. Public raw
restoration/serde/tiled_mut remain caller-owned admission. Transient rails and
drawers genuinely keep presentation separate from canonical organization.

Platen owns project_tile_layout returning Uxtree's AccessKit carrier; A221 is
qualified accordingly. Fixed root/slot-index namespaces, unbound tool tiles,
member-keyed geometry and distinct occurrence IDs need explicit profile claims,
not a universal lossless-Arrangement claim. Document scene construction delegates
to the actual document-canvas owner with caller viewport/style; no copied text
layout implementation appeared here. That owner is next for source review.

Fresh Platen no-default-feature Cargo library build and50 tests pass. Shared
Workbench serde Cargo gate passes22 unit tests and0 doctests. Nine native public
probes pass with controls, including two expected caught panics. Platen's default
document-layout feature and four wrapper tests were read but not run. No rendered
product, AccessKit consumer, GPU, native adapter or headed UI executed. Probe
executable/PDB removed; no isolated target, Cargo home or worktree created.
Evidence and source preservation are recorded in the
[Platen/Workbench review receipt](../../../support/receipts/authority-success-audit/platen_review_receipt.json).
'''
assert b'### Read-only Platen and shared Workbench source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'platen_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['platen_probe.rs','run_platen_probe.py','platen_probe_receipt.json','platen_probe_build.log',
    'platen_probe_output.log','platen_library_build.jsonl','platen_library_build_stderr.log',
    'platen_library_suite.log','shared_workbench_suite.log','platen_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Platen/shared Workbench reading; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[f['id'] for f in findings],
    'authority_entries':[e['id'] for e in entries],'coverage':stats,'added_complete_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'platen_no_default_library_tests':50,'platen_test_exit':0,
    'workbench_serde_unit_tests':22,'workbench_doctests':0,'workbench_test_exit':0,
    'diagnostic_tests':9,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
    'source_traced_consumers':{'ports/graphshell/src/forme_workspace.rs':{
        'sha256':sha(ROOT/'ports/graphshell/src/forme_workspace.rs'),
        'read_scope':'constructor, layout/open/masking, event, validate/load/save and geometry validation excerpts',
        'counts_as_complete_file_coverage':False}},
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere source objective incomplete; no production repairs.',
        'Platen document-layout default feature/wrapper tests not executed.',
        'No hosted product, renderer, native adapter, AccessKit consumer, GPU or UI execution.',
        'Product guard tracing is excerpt-level and does not count as full-file coverage.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('platen_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
