"""Save complete Forme subtree coverage and current-source diagnostic qualifications."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,re
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None:items.append(value)
        else:items[at]=value
old=load('mere_facade_review_receipt.json');probe=load('forme_probe_receipt.json')
assert probe['test_exit']==0 and probe['tests']==6 and probe['generated_output_removed']
assert all(r['exit']==0 for r in probe['runs'].values())
assert '120 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out' in (HERE/'forme_direct_unit_output.log').read_text()
assert '9 passed; 0 failed' in (HERE/'curation_direct_unit_output.log').read_text()
def read_log(name):
    raw=(HERE/name).read_bytes();return raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
assert 'os error 32' in read_log('forme_library_suite.log')
assert 'os error 32' in read_log('forme_library_build_stderr.log')
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/forme/'))
assert len(complete)==32 and sum(inventory[p]['chars'] for p in complete)==338443
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
assert all(sha(ROOT/p)==v for p,v in probe['source_sha256'].items())
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==975 and stats['read_chars']==13589713
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(number,title,source,evidence,scope,size,repair):
    return {'id':f'Q{number:03}','title':title,'source':source,
        'status':'reproduced against current directly compiled Forme source; documented, not repaired',
        'evidence':evidence,'scope':scope,'verification':'forme_review_receipt.json','diagnostic':'forme_probe_receipt.json',
        'repair_triage':{'disposition':'documented; active goal forbids production edits','size':size,'reason':repair}}
parked='Parked but compiled/public GraphTree API; repository search found no production caller, only comments outside the package. Public API defect, not demonstrated current product UI impact. '
findings=[
    finding(126,'Full parity reports success with topology, active focus, expansion and order unobserved',
        'crates/forme/forme/src/parity.rs:compare and ParityReport::full_match',
        'A tree with parent1/child2, active1 and expanded1 reports is_clean/full_match true when external membership/visible set match but children/expanded/order are empty and active=None. Explicit all-root topology or active2 controls detect the respective mismatches. Skip policy is deliberate in source/tests, but unobserved axes become true match fields and full_match claims full structural and behavioral parity.',
        parked+'Native report executed with nonempty graph and observed-mismatch controls. No claim skipped axes actually match; no adapter or migration host executed.',
        'contained diagnostic vocabulary correction; consumer gate adoption may be broader',
        'Carry checked/matched/unknown per axis or explicit capability flags and distinguish allowed migration divergence from exact membership equality. Never promote absent observations into a full-parity receipt.'),
    finding(127,'Duplicate child IDs in reorder corrupt topology while reporting structural success',
        'crates/forme/forme/src/topology.rs:reorder_children; tree/actions.rs:apply_reorder',
        'Valid tree parent1 with children[2,3] accepts NavAction::Reorder new_order[2,2], returns structure_changed=true and stores[2,2,3]. Its own assert_invariants panics on duplicate children. A unique reorder[3,2] restores invariants. Valid membership filtering does not deduplicate requested order.',
        parked+'Native ordinary public action, invariant failure caught as expected, unique-order control passes. No hostile serialization or private field edit used.',
        'contained reorder admission/deduplication correction',
        'Validate or deduplicate new_order before publication, retain omitted actual children once, and report no-op/refusal truthfully. Existing property generator never emits Reorder, so its broad any-sequence claim does not cover this action.'),
    finding(128,'Failed Forme saves stamp live timestamps and contaminate later successful creation time',
        'crates/forme/forme/src/store.rs:save_forme',
        'With session path blocked by an ordinary file, save_forme returns Err but changes a fresh document created/updated timestamps from0 to100. First successful save at200 then persists created100 and updated200. Timestamp mutation precedes directory creation, serialization and rename. Ordinary later roundtrip works.',
        'Native store feature compiled from current source, deterministic directory-admission failure, followed by successful save/load. Owned fixture removed. Does not claim a successful save lost its JSON or prove automatic host success.',
        'contained candidate timestamp staging',
        'Stamp a candidate and publish the caller document only after the file replacement succeeds, or restore the prior stamps on every error. Preserve clock policy and do not inflate tmp+rename into a multi-file/crash-durability guarantee.'),
    finding(129,'Forme lookup accepts document identity different from requested file identity',
        'crates/forme/forme/src/store.rs:load_forme and load_all_formes',
        'Two valid saved documents A/B initially load by their IDs. Copying valid A JSON over B path makes load_forme(B) return Some(documentA) without error. Renaming the duplicate to not-a-uuid.json also admits two A documents from load_all_formes, despite its uuid-only filename comment. Only JSON syntax is checked.',
        'Native valid-file mismatch and list duplication with ordinary matching-ID control; no host write-back, malicious remote input or graph-binding bypass executed. Caller still owns expected graph binding.',
        'contained filename/document-ID admission, with explicit list diagnostics policy',
        'Check the decoded ID against requested/path ID. Enumerated files need valid filenames and an explicit duplicate/refusal outcome. Keep graph binding policy host-owned; do not create a second catalog to conceal mismatches.'),
    finding(130,'Cold ancestors suppress active descendant pane geometry while accessibility still publishes it',
        'crates/forme/forme/src/tree/layout.rs:layout_split_panes and visible_children_of; ux.rs:emit_split_container',
        'Cold root1 with Active child2 returns active2 and a tab for2 but no pane rectangles in SplitPanes. emit_ux_tree nevertheless emits Pane2. Making parent1 Warm restores child geometry. Layout filters visible roots/immediate children before descent; accessibility independently includes all visible members in insertion order.',
        parked+'Native geometry and UX descriptors executed with lifecycle control. This is divergence between two projections, not proof of screen-reader/platform rendering failure.',
        'shared visible-frontier projection; policy/adoption may exceed one-hour diversion',
        'Resolve visible descendants through non-present ancestors and let geometry/accessibility consume the same admitted pane set. Preserve topology and host lifecycle policy; do not reactivate cold ancestors merely to hide the divergence.'),
    finding(131,'Nested split handles use a representative leaf edge instead of the sibling subtree edge',
        'crates/forme/forme/src/tree/layout.rs:extract_layout_results and first_leaf_member',
        'For a horizontal subtree of members1/2 beside root3 in1000x600, actual subtree/root3 boundary is x500 but the outer handle is x375. It averages first leaf1 right edge x250 with root3 left edge x500. Vertical-subtree control puts the handle at the correct edge. The inner pane rectangles abut correctly.',
        parked+'Native finite geometry and direction control. No interactive drag or rendered host exercised; geometry defect is independent of current product adoption.',
        'contained container-bound geometry correction; nested resize intent mapping needs follow-up',
        'Compute outer handles from direct sibling container layouts, retaining representative members only as labels/intent references. Qualify whether drag acts on a container or only its first leaf before promising correct nested resizing.'),
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'Complete Forme subtree32 files read; bounded direct native tests; no Cargo package gate',
        'verification':'forme_review_receipt.json'}
entries=[
    entry(211,'Canonical arrangement and parked navigation','Arrangement core owns workbench facts; rich graph navigation is parked.',
        ['Geometry-free Arrangement/FormeDocument are canonical modules',
         'GraphTree, TreeTopology, lenses, reconciliation, pressure, legacy UX remain compiled/reexported',
         'Repository symbol search finds product Arrangement/FormeDocument consumers, but only comments for GraphTree/TreeTopology outside Forme'],
        'Parking is a warning, not code removal. Public legacy authority remains available, although current product adoption was not found. Removing it needs API/consumer review, not a broad unqualified deletion. Tests in this old lane do not qualify the canonical Platen workbench.'),
    entry(212,'Subgraph specification versus roster implementation','Curation owns shared specs and hosts derive member truth.',
        ['Forme reexports actual Curation SubgraphKind/SubgraphSpec',
         'Old GraphTree reconciliation computes roster deltas and choices over member tags',
         'Mere SessionSubgraphs repeats set-difference logic over anchors and auto-applies checked graph derivation'],
        'Spec extraction is real shared ownership. Delta implementations remain separate with different stores/policies, so a single spec vocabulary does not imply one reconcile authority. Old fork helpers change the current binding; SaveAsNewFork does not mint a new subgraph despite its variant description. No new functional consumer failure inferred.'),
    entry(213,'Member identity and appearance identity','One member may have several arrangement occurrences.',
        ['ArrangementNodeId is distinct from GraphMemberId',
         'Identity preservation reuses member IDs only for unique old/new occurrences',
         'Repeated appearances keep supplied IDs; rebound nodes are guarded'],
        'Source and original passing tests preserve these distinctions. UUID aliases still rely on host stratum resolution. Identity(members) builds a disposable materialization; it is not a durable copied roster. Root/schema validation is not automatically established by serde roundtrip.'),
    entry(214,'Store identity and publication','Store owns JSON shape/I/O; host supplies directory and time.',
        ['save_forme stamps caller then writes tmp/rename',
         'load_forme checks syntax but not requested ID',
         'load_all_formes skips malformed/unreadable files and only checks extension'],
        'Q128/Q129 establish concrete contradictions at failure and identity seams. Single-file rename is not fsync or multi-writer serialization. List skip policy is explicit, but lacks evidence of complete session recovery; returned success cannot certify every file loaded.'),
    entry(215,'Parity outcomes versus missing coverage','Full match means full structural and behavioral parity.',
        ['Cold omissions tolerated without membership mismatch count',
         'Empty topology/expansion/order and absent active focus skip comparisons',
         'Boolean full_match has no coverage/unknown state'],
        'Q126 demonstrates positive output with unobserved axes. Tests intentionally pin skip behavior and can all pass; their names do not expand measurement scope. Shared policy and actual comparison coverage should remain distinct.'),
    entry(216,'Mutation invariants and property-test scope','Any navigation action sequence preserves topology invariants.',
        ['Normal attach/reparent reject missing IDs, duplicates and cycles',
         'Reorder only filters membership, admitting duplicate IDs',
         'Property generator omits Reorder while header says any sequence',
         'An attach_and_query helper in topology tests lacks #[test]'],
        'Q127 is an ungenerated public action that violates invariants. Direct120-test pass is useful but excludes four store tests and integration property tests. No universal invariant proof or petgraph external-identity uniqueness admission inferred.'),
    entry(217,'Lifecycle, topology, pane set and accessibility','Visible panes and semantic panes are projections of the same tree.',
        ['Geometry descends only through immediately visible members',
         'Legacy UX emits all visible members independently by insertion order',
         'SetLifecycle changes state without activation/dismissal intents; explicit Activate/Dismiss have intents'],
        'Q130 proves the divergent pane set. Lifecycle notification is explicitly host-supplied, so absence of an activation intent on SetLifecycle is not by itself a bug. Topology rows, tab order and pane order are different definitions and need explicit semantics rather than a generic visibility label.'),
    entry(218,'Split geometry versus resize identity','Split boundaries describe adjacent sibling pane containers.',
        ['Taffy computes correct leaf rectangles',
         'Boundary uses first leaf of each subtree as geometry',
         'Split ratio lives on member leaves rather than container identities'],
        'Q131 proves wrong outer handle geometry; actual nested drag semantics remain unexecuted. Platen owns the current workbench projection and is next to read; do not promote legacy geometry tests as its receipt.'),
    entry(219,'Pressure policy and actual resource authority','Policy recommends lifecycle demotion, host applies it.',
        ['Host supplies opaque origin and recency',
         'Per-origin/global passes demote only Warm nonexempt members',
         'Active/exempt members can leave budgets exceeded intentionally'],
        'Implemented as advisory data, not resource deallocation. Current original tests pass. Source comment about least-recent origins is broader than actual global per-member recency sort. No actual process memory, GPU context release, or bounded host runtime receipt.'),
    entry(220,'Curation fold wire admission and source authority','Folds are view-local curation, not source graph groups.',
        ['V1 decode migrates to V2 summary and validates sorted unique members',
         'Constructors canonicalize and validate stand-in/root/label',
         'Host owns scope/member availability and active overlap checks'],
        'Nine fresh original Curation tests pass. Shared types and validation are actual reuse. Public fields and direct V1 migrate allow callers to manufacture unchecked values, so a type alone is not an admission proof. No overlap/source-scope policy is silently invented here.'),
    entry(221,'Two accessibility vocabularies','Portable accessibility is shared across hosts.',
        ['Old Forme ux emits private generic descriptors over GraphTree with Debug-based IDs',
         'uxtree projects Inker blocks to actual AccessKit nodes with hashed domain paths',
         'uxtree graft module joins independent TreeId namespaces; stitch merges one namespace'],
        'These are distinct implementations and identity schemes. The older lane is parked and no product adoption found, so coexistence does not establish two live platform authorities. Current shared uxtree has no Platen-specific projection despite its introductory description; consumers may construct UxTree directly.'),
    entry(222,'Graft admission and consumer safety','Composition checks the consumer rules before an adapter can panic.',
        ['Checks host graft/guest correspondence, duplicate tree IDs, graft children and nonempty guests',
         'as_subtree guesses missing root from a parentless node',
         'No general per-tree topology/focus/root validation',
         'Original E1a tests model joins, focus, bounds, actions and retirement across three consumer versions'],
        'Strong concrete modeled join coverage in source, but blanket panic-prevention prose is broader than checks. Malformed/cyclic guest trees and nested grafts remain source-only candidates, not reproduced defects. Those original uxtree tests were read, not freshly executed; OS adapters and actual hosted sessions are outside their stated scope.'),
    entry(223,'Document projection and stable source identity','One structural tree serves accessibility, automation and inspector.',
        ['Paths include exact document address and original block indices',
         'Presented wrappers unwrap while retaining IDs; folds use current Navigation and retain surviving indices',
         'Inline in-page links have no URL and their target is not retained in AccessKit node',
         'Link/button roles carry values but do not add Click actions here; host owns event synthesis'],
        'Projection identity is deterministic for the same authored shape, not invariant under arbitrary insertion or duplicate appearances. Host action mapping for InPage/Submit needs consumer qualification. No platform inability or navigation bug claimed from values/roles alone. Old gpui integration prose and html_root_url0.0.1 are stale beside0.0.2/current stack.'),
    entry(224,'Stitch versus graft ownership','Independent sessions keep independent node namespaces.',
        ['stitch concatenates UxTree nodes into one namespace without duplicate admission',
         'graft keeps guest NodeIds under separate TreeIds with host routing table',
         'Grafts owns publication order/live focus metadata, not durable session truth'],
        'Both seams exist with different caller preconditions. Duplicate document appearances stitched with identical paths can collide in source; no current product collision reproduced. The E1a raw-ID control demonstrates why independent sessions use grafts, not proof every stitch consumer has migrated. Stable hashing uses DefaultHasher, whose cross-toolchain stability is not established by same-run tests.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Uxtree composition panic-prevention scope exceeds per-frame graft checks',
     'source':'crates/forme/uxtree/src/graft.rs:Composition::new, as_subtree',
     'note':'General guest root/focus/tree topology and nested graft validation absent. No malformed guest consumer execution this pass; do not promote source suspicion to native finding. Qualify caller-owned valid-tree preconditions versus promised safety before a potentially broad shared validator redesign.'},
    {'title':'Uxtree structural identity and host action admission need consumer tracing',
     'source':'crates/forme/uxtree/src/lib.rs:stitch, attach_link_children, project_menu',
     'note':'Stitch concatenates duplicate path IDs; InPage target is not carried, and links/submit button carry values without Click action. Hosts own event synthesis and may retain separate maps. No current host/OS fault reproduced. Trace action owners and multi-appearance consumers before proposing an identity/action migration beyond the one-hour diversion limit.'},
    {'title':'Arrangement structural admission is separate from serde shape',
     'source':'crates/forme/forme/src/arrangement.rs and store.rs',
     'note':'Insert can create another WorkbenchRoot; attach/relate accept arbitrary IDs and cycles, while serde derives admit raw shape. Caller/schema preconditions and Platen validation need reading. Do not label raw builder flexibility a proven product bug without that owner seam.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/platen/platen/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Entire crates/forme subtree32/32 read: Forme25, Curation3, Uxtree4'
notes['next_area']='crates/platen/platen'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['forme_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'Six native Forme observations,120 original unit tests excluding four store tests,9 original Curation tests pass. Direct rustc source compilation; Cargo attempts failed on shared target lock.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'32 more complete files,338443 chars. Uxtree source-only; original property tests read, not executed.'}
upsert(notes['source_qualifications'],[{'area':'Complete Forme/Curation/Uxtree source subtree',
    'note':'32 files338443 chars fully read. Fresh direct rustc builds of unchanged Forme source with store/petgraph,120 original non-store unit tests,6 generated native observations and9 original Curation unit tests pass against exact hashed prior Cargo deps. Cargo test/build attempts both fail before compilation on Windows shared target lock. Four original store tests, property integrations, all Uxtree tests/consumers, native OS/headed/GPU not run. Owned direct libraries/executables/PDBs and fixtures removed; no isolated cache.'}],'area')
assert len({f['id'] for f in notes['findings']})==len(notes['findings'])
assert len({f['id'] for f in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only complete Forme subtree review, 2026-10-16

All32 inventoried files under crates/forme are read: Forme25, Curation3,
Uxtree4. This adds338,443 characters; repository coverage is975/3352 files
and13,589,713/36,854,820 characters. The full-source goal remains active.
Resume at crates/platen/platen/Cargo.toml. Production and concurrent work
remain unchanged. Authority entries A211-A224 accompany individual findings.

Q126: full parity can mean unobserved. Nonempty parent/child tree with active
and expanded root reports clean/full_match when external topology, expansion,
order and active focus are absent. Explicit topology/active mismatch controls
are detected. Skipping unsupported axes is intentional; reporting those as
matched and claiming full structural/behavioral parity loses coverage truth.
Separate unknown/checked/matched and allowed migration divergence.

Q127: valid parent1 children[2,3] accepts reorder[2,2], reports structural
change and stores[2,2,3]. Its invariant checker panics; a unique-order control
passes. Membership filtering is not deduplication. The property generator
never emits Reorder, despite the any-sequence introduction; those tests were
read but not run. Deduplicate or refuse before mutation and retain omitted
actual children once. This is a contained mutation admission correction.

Q128: failed store save stamps the live document before I/O. A file blocking
the session directory makes save return Err but changes timestamps0 to100.
First successful save at200 persists created100. Stage timestamps with the
candidate and publish them only after replacement succeeds. Native ordinary
failure and later roundtrip are executed; no automatic host save is claimed.

Q129: load_forme(B) accepts valid A JSON in B's filename and returns A. The
matching-ID control succeeds first. load_all_formes also accepts arbitrary
not-a-uuid.json and returns duplicate A records, despite its uuid-only comment.
Check file/document identity and report list refusals explicitly. Expected
graph binding remains caller-owned; no remote or authorization bypass tested.

Q130: Cold root1 with Active child2 yields active2 and a tab/UX Pane for2 but
no pane rectangle. Warm-parent control restores geometry. Geometry prunes
nonvisible ancestors before descent, while legacy accessibility independently
emits every visible member. Share an admitted visible frontier without forcing
ancestors active. Current host adoption and platform accessibility effects are
unproven; policy/consumer migration may exceed the one-hour diversion limit.

Q131: a horizontal member1/2 subtree beside root3 has true boundary x500 in
a1000px area, but outer split handle x375. First leaf1 ends at250; the code
averages that with root3's500 rather than using the direct container boundary.
Vertical-subtree control is correct. Use container geometry; nested drag intent
mapping remains a separate, unexecuted consumer question.

The four GraphTree/parity/legacy-layout findings affect parked but compiled
public APIs. Repository search found no production GraphTree/TreeTopology call
outside Forme, only comments. These are public defects, not demonstrated
current product failures. Forme's canonical Arrangement/FormeDocument do have
product consumers. Parking is a warning, not removal; deprecation needs an
API/consumer review rather than sweeping deletion. Arrangement occurrences and
graph members have distinct IDs; unique-member identity preservation is real.
Curation really owns SubgraphSpec/Kind through reexports. Old GraphTree delta
and newer Mere SessionSubgraphs delta implementations remain separate, with
different storage and automatic/proposal policies. One spec does not establish
one reconciliation implementation. Pressure remains advisory; no resource
release is proved by recommending Cold.

Uxtree's AccessKit projection is separate from parked Forme UxNodeDescriptors.
It carries original block paths through presentation wrappers and folds, while
graft retains independent TreeId namespaces and orders focus updates. Source
contains substantive modeled tests for three consumer versions, including raw-ID
and missing-graft controls. Those were read, not rerun here; they explicitly
leave actual sessions and native adapters to other receipts. Composition's
blanket panic-prevention prose exceeds its graft-pair checks: general guest
topology/root/focus checks are absent. That remains a source-only candidate.
Stitch path collisions, InPage/Submit action mapping and missing Click action
advertisement also require host tracing before any functional claim or broad
migration. Stable same-run hash IDs do not prove cross-toolchain hash stability.
No speculative production repair was made.

Direct rustc builds of current unchanged Forme source with store/petgraph pass
six native observations and120 original unit tests; four original store tests
are deliberately filtered. Current Curation source passes its nine original
unit tests. Cargo package test/build attempts failed before compilation on the
shared Windows target lock, so no Cargo gate is claimed. Dependencies are exact
hashed artifacts from the prior successful graph Cargo build; a cached kernel
fixtures library is imported by original test sources. Integration/property and
Uxtree tests, OS/headed/product/GPU paths were not executed. All owned direct
libraries/executables/PDBs and fixtures are removed; no target/home/worktree
isolation created. Preservation and exact commands appear in the
[Forme subtree review receipt](../../../support/receipts/authority-success-audit/forme_review_receipt.json).
'''
assert b'### Read-only complete Forme subtree review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'forme_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['forme_probe.rs','run_forme_probe.py','forme_probe_receipt.json','forme_direct_library_build.log',
    'forme_probe_build.log','forme_probe_output.log','forme_direct_unit_build.log','forme_direct_unit_output.log',
    'curation_direct_unit_build.log','curation_direct_unit_output.log','forme_library_suite.log',
    'forme_library_build.jsonl','forme_library_build_stderr.log','forme_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Forme subtree source review; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[f['id'] for f in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'added_complete_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'diagnostic_tests':6,'diagnostic_exit':0,'direct_forme_unit_tests':120,
    'direct_forme_store_tests_skipped':4,'direct_curation_unit_tests':9,'cargo_gate':'failed before compile: os error32 shared target lock',
    'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere objective incomplete; no production repairs.',
        'Direct source rustc gates with exact prior Cargo dependencies, not successful Cargo package gate.',
        'Uxtree source only; no property integration, OS, actual host sessions, GPU or headed runtime execution.',
        'Parked GraphTree current product adoption not found in repository symbol search.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('forme_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
