"""Record fully read facade sources and bounded native evidence; no product edits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old=load('modulus_manifest_review_receipt.json'); probe=load('mere_facade_probe_receipt.json')
assert probe['test_exit']==0 and probe['tests']==3 and probe['generated_output_removed']
assert probe['library_build_exit']==0 and probe['probe_build_exit']==0
suite=(HERE/'mere_graph_suite.log').read_text(encoding='utf-16' if (HERE/'mere_graph_suite.log').read_bytes().startswith(b'\xff\xfe') else 'utf-8')
assert 'test result: ok. 45 passed; 0 failed' in suite
assert not (HERE/'mere-sidecar-fixture').exists()
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/mere/'))
assert len(complete)==18 and sum(inventory[p]['chars'] for p in complete)==215322
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['library_sha256'].values())
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==943 and stats['read_chars']==13251270
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
findings=[
    {'id':'Q123','title':'URL-structure outline silently loses distinct graph nodes sharing host and path',
     'source':'crates/mere/src/glossary.rs:parse_host_path, build_trie, outline_rows, outline_djot; gloss.rs:build_outline_snapshot',
     'status':'reproduced through current public graph-only APIs; documented, not repaired',
     'evidence':'Each pair of valid addresses differing only by query, scheme, or fragment creates two graph nodes but yields only one linked outline row and one Djot link. Distinct-path control retains both. The trie drops scheme/query/fragment and stores only one here value per host/path, overwriting the previous entry.',
     'scope':'Native outline and Djot output executed. Canvas snapshot source re-resolves emitted URLs rather than retaining stable member IDs, but no canvas selection or product UI executed. No assumption about which member wins iteration order.',
     'repair_triage':{'disposition':'documented; current active goal forbids production edits','size':'contained outline leaf preservation; broader stable selection migration may exceed one-hour diversion','reason':'Separate hierarchy grouping from member identity. Retain every distinct member at a shared path and carry stable IDs through consumers; URL structure must not become a second identity authority.'}},
    {'id':'Q124','title':'Equal endpoint titles interleave distinct relation bundles into repeated groups',
     'source':'crates/mere/src/roster.rs:build_link_rows',
     'status':'reproduced through current public graph-only row projection; documented, not repaired',
     'evidence':'Two distinct endpoint pairs with equal source/target titles and Cites/Quotes assertions sort A-Cites,B-Cites,A-Quotes,B-Quotes, producing four starts_bundle markers for two bundles. Distinct-title control produces two. Sorting considers relation kind before stable endpoint IDs, then grouping compares adjacent endpoint IDs.',
     'scope':'Native row DTO output, with graph-minted stable IDs. No rendered roster UI executed; assertions and editability are retained, so the defect is grouping rather than deletion.',
     'repair_triage':{'disposition':'documented; current active goal forbids production edits','size':'contained stable-ID bundle ordering correction','reason':'Group by stable endpoint pair before ordering assertions within a bundle. Keep same-kind assertion IDs and source-owned editability separate.'}},
    {'id':'Q125','title':'Corrupt subgraph sidecars silently become fresh indexes that can replace saved definitions',
     'source':'crates/mere/src/subgraph.rs:SessionSubgraphs::load and save',
     'status':'reproduced public sidecar load/save with valid-file control; documented, not repaired',
     'evidence':'A valid two-definition index reloads both. Truncating its sidecar makes load return one default session with no error. Load itself leaves corrupt bytes intact; saving that returned index replaces them, and the next load contains only the default. Missing, unreadable and corrupt inputs share the same fallback in source; only truncation was exercised.',
     'scope':'Native temporary fixture under the audit evidence folder, removed after probe. Explicit subsequent save executed; no automatic host save cycle or crash injection claimed. The source documents its forgiving posture, but the API exposes no distinction for recovery.',
     'repair_triage':{'disposition':'documented; current active goal forbids production edits','size':'checked load outcome plus host recovery policy; full adoption may exceed one-hour diversion','reason':'Distinguish first-run absence from damaged/unreadable state, preserve recoverable bytes, and require hosts to handle the checked result before replacement. Do not invent a parallel subgraph store or claim crash-safe multi-file persistence.'}},
]
for f in findings:
    f['verification']='mere_facade_review_receipt.json'; f['diagnostic']='mere_facade_probe_receipt.json'
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,'assessment':assessment,
        'state':'All18 facade sources read;45 graph-only library tests and3 native observations pass',
        'verification':'mere_facade_review_receipt.json'}
entries=[
    entry(202,'Facade composition versus blanket boundary prose','The facade does not expose render or session-host concerns.',
        ['lib reexports the actual kernel, Forme, linked-data, Pictograph canvas and workbench owners',
         'graph is the portable domain feature; canvas/workbench are optional capabilities included in defaults'],
        'Real owner reuse rather than copied implementations. Introductory blanket exclusion is broader than the feature-scoped composed API; feature documentation supplies the narrower distinction. A root facade name alone does not prove portability of every feature closure.'),
    entry(203,'Outline grouping versus identity','Glossary and Gloss project navigation from graph truth.',
        ['Glossary manually splits URL shape and stores one leaf per host/path',
         'Gloss enriches rows by URL lookup and emits URL-based selection intents',
         'QueryView carries Resource and Surface IDs separately'],
        'Q123 proves a lossy alternate identity on outline/Djot output. URL grouping can remain presentation, but cannot own member identity. QueryView is a stronger existing seam to follow, not proof the outline already shares it.'),
    entry(204,'Roster labels versus stable grouping','Link rows preserve exact assertions and endpoint bundles.',
        ['Input/output retains endpoint IDs, assertion IDs and caller editability',
         'Ordering prioritizes labels then relation kind; bundle starts compare adjacent IDs'],
        'Q124 proves display labels disrupt grouping despite retained IDs. RelationSelector is explicitly only a filter; exact assertion ID remains mutation identity. Do not replace that distinction with family/kind-based edit authority.'),
    entry(205,'Subgraph derivation and persistence','Kernel owns graph walks; local roster is a revision-gated cache.',
        ['Component/ego delegate kernel; unsupported shape kinds explicitly fall back to seed',
         'Checked query creation/reconcile preserve refusal diagnostics and prior roster',
         'Shape compatibility APIs erase errors; observations expose them separately',
         'Sidecar save uses direct write and load folds errors into fresh default'],
        'No second graph truth store in derivation, but Q125 is a real recovery/success ambiguity. Pure caches skip revision on deserialize and invalidate edits. Coverage refreshes separately from graph revision. Swapping graph instances requires explicit cache clearing as documented; no cross-instance proof claimed.'),
    entry(206,'Saved queries, appearance projection and freeze','Saved definitions and frozen membership are distinct.',
        ['Resource facet stores checked QueryViewSpec with exact expected-value admission',
         'SessionSubgraphs shares only specs and derives on receiver',
         'Freeze re-evaluates and resolves shape Surface IDs versus query Resource IDs before delegating kernel'],
        'Source implements the split and preserves cross-stratum IDs and exact source assertions. QueryView save need not pass later appearance/relation budgets; tests explicitly allow that. Member/presentation budgets do not claim evaluator time limits. Query-specific tests were read, not run in this graph-only gate.'),
    entry(207,'Clip insertion success and durable authority','Host owns receiving session, body formatting and image persistence.',
        ['Clip uses graph apply helpers and source-owned ClippedFrom relation',
         'Helper discards operation returns and succeeds on node existence'],
        'Shared operations are actual reuse. Source establishes only node existence as final success condition; no valid-input lower-level refusal or partial clip was reproduced. Do not promote this candidate to a proven bug. Clip and durable host journal were not executed.'),
    entry(208,'Strength editing authority','StrengthDraft is an inert candidate over an exact coupling/owner baseline.',
        ['Finite parsing, bounded local undo/redo, exact coupling plus owner equality',
         'Candidate returns typed commit intent; graph_id validation and durable write belong to caller'],
        'Not an independent durable coupling authority. Caller graph identity precondition is explicit; candidate not accepting graph_id is not itself an admitted cross-graph bug. Draft tests were read but canvas/workbench not executed.'),
    entry(209,'Routing rules and host markers','Mere layers app-owned routes over Inker engine policy.',
        ['Internal schemes and JSON-LD marker append rules to actual Inker policy',
         'JSON-LD marker is documented headless host ingest, not registered render engine'],
        'Single engine policy reuse is real, while disjointness/order claims need Inker matcher qualification. An internal address can also carry a content type, so prose alone cannot establish disjoint requests. No workbench route execution or external Inker source review in this checkpoint.'),
    entry(210,'Trail and classifier presentation ownership','Neutral domain vocabulary consumes caller-owned live state.',
        ['Trail input is explicit, shortened rows are inert, recovery retains node ID',
         'Classifier deduplicates induced undirected edges and returns shape labels; kernel gathers relations'],
        'These are bounded pure projections, not separate persistence authorities. Structural classification and linked derivation support differ explicitly: a displayed Loop/Corridor rank does not prove a full linked shape walk. Forme owns shared shape/spec types and is next for source review.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Clip insertion checks node existence rather than all requested postconditions',
     'source':'crates/mere/src/clip.rs:insert_knot_clip',
     'note':'Multiple apply return values are ignored; no valid-input refusal reproduced, so remains candidate. Shared graph authority is reused and durable journal belongs to host. Broader transactional clip admission needs owner tracing, not a speculative patch.'},
    {'title':'Subgraph sidecar recovery and stable outline selection need consumer migration',
     'source':'crates/mere/src/subgraph.rs; glossary.rs; gloss.rs',
     'note':'Q125 and Q123 proven at public API seam. Host recovery/save and stable-ID navigation adoption may exceed one-hour diversion. Record checked outcomes, preservation policy and shared IDs before changing all consumers. Direct sidecar save is not crash-atomic; no crash/power-cut injection performed.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/forme/forme/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='crates/mere facade18/18 fully read; graph-only tests45/45 and three native defect observations'
notes['next_area']='crates/forme/forme'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['mere_facade_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'45 graph-only library tests and three fresh public API native observations pass.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'18 more full files and215322 chars. Full Mere objective active.'}
upsert(notes['source_qualifications'],[{'area':'Mere facade complete source review',
    'note':'All18 inventoried facade files read,215322 chars. Current graph-only Cargo library gate45 tests passes. Three native observations prove outline loss, bundle grouping and corrupt sidecar replacement, with controls. No query/clip/canvas/workbench/default feature, product UI or GPU execution. Probe binaries and sidecar fixture removed; stable shared target only.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only Mere facade source review, 2026-10-16

All18 inventoried crates/mere source/manifest files are read, adding215,322
characters. Full-source coverage is943/3352 files and13,251,270/36,854,820
characters. The complete repository objective remains active. Resume with
crates/forme/forme/Cargo.toml. Production/concurrent sources remain unchanged.

Q123: outline grouping becomes a lossy identity authority. Two graph nodes
differing by scheme, query or fragment yield only one linked outline row and
one Djot link. Distinct paths retain two. The hand-written URL splitter drops
those distinctions and one trie here slot overwrites the prior member. Gloss
then re-resolves URLs instead of carrying stable IDs, though Canvas selection
was not executed. Preserve every member independently of hierarchy grouping;
full consumer selection migration may exceed the one-hour diversion limit.

Q124: two endpoint pairs with equal titles, each with Cites and Quotes, become
four bundle starts. Labels then relation kind sort before stable endpoint IDs,
interleaving A-Cites,B-Cites,A-Quotes,B-Quotes. Distinct-title control has two
starts. This is a contained grouping correction; exact assertion IDs and
source-owned editability already survive and should remain independent.

Q125: a valid two-definition subgraph sidecar reloads both, but a truncated
copy silently loads as one fresh default. Load leaves corrupt bytes intact;
an explicit subsequent save replaces them, losing recoverable definitions.
The tested path is public load/save, not an automatic host save or crash.
Separate missing first-run state from corrupt/unreadable state and expose a
checked recovery outcome. Host recovery/adoption and crash-safe multi-file
policy are broader work and are documented rather than delaying the audit.

Authority entries A202-A210 distinguish real reuse from competing meanings.
The facade reexports actual owners; its blanket exclusion of render/host
concerns is broader than its optional/default canvas/workbench composition.
QueryView preserves Resource and Surface identity, exact source assertions,
and explicit projection refusals. Saving its spec intentionally need not pass
later appearance/relation budgets. Those budgets do not bound SPARQL runtime.
Subgraph component/ego walks delegate kernel, while other structural kinds
explicitly fall back to the seed. A classifier chip is not proof that a linked
walk for that shape is implemented. Sharing carries specs; freeze re-evaluates,
keeps strata distinct, and delegates frozen Resource ownership to kernel.

StrengthDraft stays a local candidate, with caller graph identity and journal
ownership explicit. Clip similarly reuses apply helpers but ignores their
returns and checks only node existence. No valid-input partial clip was
reproduced, so that remains a candidate. Trail is an explicit-input projection.
Routing app markers reuse Inker policy; matcher/order claims need that owner
read before promotion. No second implementation is inferred from a reexport
or UI label alone.

Fresh Cargo graph-only library gate passes45 tests. Three generated native
public-API observations pass with controls. Query/clip/canvas/workbench sources
and their feature-gated tests are read, not executed by this gate. No default
feature, renderer, GPU, hosted product or UI receipt is claimed. Owned probe
executable/PDB and sidecar fixture are removed; no isolated target, Cargo home
or worktree created. Hash preservation and evidence are in the
[Mere facade review receipt](../../../support/receipts/authority-success-audit/mere_facade_review_receipt.json).
'''
assert b'### Read-only Mere facade source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'mere_facade_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['mere_facade_probe.rs','run_mere_facade_probe.py','mere_graph_library_build.jsonl',
    'mere_graph_library_build_stderr.log','mere_facade_probe_build.log','mere_facade_probe_output.log',
    'mere_facade_probe_receipt.json','mere_graph_suite.log','mere_facade_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Mere facade source reading; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'added_complete_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'graph_only_library_tests':45,'graph_only_library_exit':0,
    'diagnostic_tests':3,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full source objective incomplete; no production fixes.',
        'Graph-only library tests and CPU/filesystem observations only; no default/canvas/clip/query/workbench or GPU/UI gate.',
        'Corrupt sidecar replacement requires explicit subsequent save; no automatic host or crash simulation.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('mere_facade_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
