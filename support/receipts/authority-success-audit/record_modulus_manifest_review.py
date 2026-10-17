"""Record complete Modulus/root-build-source review with honest test boundaries."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,v): (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old=load('conatus_review_receipt.json'); probe=load('modulus_probe_receipt.json')
gate=load('manifest_helper_gate_receipt.json'); declarations=load('root_manifest_declaration_check.json')
assert probe['test_exit']==0 and probe['tests']==2 and probe['generated_output_removed']
assert probe['suite_exit']==0 and probe['suite_passed_counts']==[18,8,0]
assert gate['export_tests']['exit']==0 and gate['export_tests']['passed']==4
assert gate['stale_alias']['missing_from_lock'] and not gate['stale_alias']['alias_executed']
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in gate['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['library_sha256'].values())
notes,authority=load('review_notes.json'),load('authority_map.json')
findings=[
    {'id':'Q120','title':'Public brick material lookup aliases distant world cells through a narrowing key cast',
     'status':'reproduced through public Modulus CPU APIs; documented, not repaired',
     'source':'crates/conatus/modulus/src/lib.rs:BrickMap::material_at; trace.rs:brick_material_at',
     'evidence':'For a map containing only key[0,0,0] filled with material7, material_at([524288,0,0]) and material_at([-524288,0,0]) both return7. Nearby outside x8 correctly returns0, and actual public trace at either distant coordinate is Clear. Public material_at divides world i32 coordinates by8 then casts to i16 without checking range, wrapping +/-65536 to key0. The pointer-volume reader used by traversal instead bounds local coordinates and treats outside volume as air.',
     'scope':'One generated native test against a freshly built Modulus library, with selected/near-outside and actual trace controls. No GPU or product scene executed. The trace itself is bounded and does not inherit this distant-key alias; the defect is the separate public material lookup.',
     'diagnostic':'modulus_probe_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained lookup unification/admission repair',
        'reason':'Check key range before narrowing or route public lookup through the existing bounded pointer-volume reader. Keep negative coordinates, retained high slots and material byte order controls. This directly removes the divergent lookup authority rather than adding a third copy.'}},
    {'id':'Q121','title':'The exact DDA parallel-threshold endpoint is classified differently by clipping and stepping',
     'status':'reproduced through public Modulus CPU traversal; WGSL matching source inspected only',
     'source':'crates/conatus/modulus/src/trace.rs:brick_ray_box and brick_crossing; brick_dda.wgsl matching predicates',
     'evidence':'A finite ray starting at(0.5,4.5,4.5), with solid voxels from x4, returns Clear for direction(1e-6,0,0). The next representable f32 direction above1e-6 and the unit-direction control both hit voxel[4,4,4]. Clipping treats only abs(d)<1e-6 as parallel, while crossings move only for d>1e-6 or d<-1e-6. Exact endpoints thus clip as moving but produce sentinel crossings. The CPU public contract explicitly permits nonunit directions. The WGSL contains the same predicate mismatch; it was not executed.',
     'scope':'One generated native test with finite public inputs and neighbouring/unit controls. Bounded numeric endpoint finding, not a product-camera or general GPU parity failure. Below-threshold behaviour remains the explicitly approximate raw traversal policy; no global threshold redesign is inferred.',
     'diagnostic':'modulus_probe_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained boundary-predicate correction',
        'reason':'Choose one consistent endpoint classification across CPU clip/crossing, WGSL and exact test reference, then test both signed endpoints and adjacent floats. Do not silently retune the overall parallel threshold or replace the finite-precision traversal policy.'}},
    {'id':'Q122','title':'The recommended test-web example alias applies its workaround to a retired package',
     'status':'source-confirmed example configuration defect; no alias/build execution; documented, not repaired',
     'source':'.cargo/config.toml.example:alias.test-web; crates/canvas/pictograph/Cargo.toml:package.name; Cargo.lock',
     'evidence':'The example instructs copying it to config.local.toml and describes disabling incremental for the canvas package to avoid a historical Windows link failure. Its alias sets profile.dev.package.mere-canvas.incremental=false. Current canvas package is pictograph and Cargo.lock contains no mere-canvas. The intended package therefore receives no override. Active tracked .cargo/config.toml contains only build-dir; this alias is not currently active there.',
     'scope':'Parsed example, actual package manifest and current lock names. The Windows failure and alias were not rerun, so current necessity/effectiveness of the historical workaround is not established. This is a stale target in a recommended configuration, not a claim the current web build fails.',
     'diagnostic':'manifest_helper_gate_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained example target correction, followed by separate workaround retirement check',
        'reason':'Target the actual package if retaining the workaround, then qualify whether current toolchain still needs it. Updating the package name alone does not prove the historical linker defect still exists or the workaround can be removed.'}},
]
for value in findings: value['verification']='modulus_manifest_review_receipt.json'
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'Modulus12/12 and12 root build/config/helper sources read;26 Modulus+4 exporter tests and2 observations',
        'verification':'modulus_manifest_review_receipt.json'}
entries=[
    entry(189,'Material lookup authorities','Picks and shader-facing material reads refer to the same bounded atlas.',
        ['Public material_at uses key lookup with unchecked i32-to-i16 narrowing',
         'Private brick_material_at uses bounded pointer coordinates',
         'WGSL follows pointer/atlas texture lookup', 'Public trace uses the private CPU reader'],
        'Q120 proves the two production CPU readers disagree outside the key domain. The bounded trace control narrows impact; do not claim every pick aliases. Existing in-range/shrink tests can pass while this divergence survives.'),
    entry(190,'Parallel classification across traversal stages','Clip and DDA take the same ray rules.',
        ['Clip parallel test uses abs(d)<threshold', 'Crossing movement uses strict signed >/<threshold',
         'CPU and WGSL manually repeat both predicates',
         'Exact reference converts f32 ray values to f64 but compares to its own f64 literal'],
        'Q121 demonstrates the exact endpoint gap on CPU. Matching copies preserve the same defect rather than proving correctness. The f64 reference is a useful rounding instrument but its threshold literal is not necessarily the exact promoted f32 threshold; edge policy should be shared explicitly.'),
    entry(191,'CPU traversal references versus executed WGSL','The CPU mirror is described as the shader walk, operation for operation.',
        ['Default tests call BrickMap.trace, never a GPU dispatch',
         'Exact f64 walk measures rounding under the same authored offset/tie/budget policy',
         'Frozen accumulated walk is an explicit test-only positive control',
         'Shared shader test checks source strings and absence of product-policy words'],
        'Fresh26-test gate qualifies CPU and source constraints, not actual WGSL execution or cross-backend FMA behaviour. The test-only historical implementation is a reference, not an extra production authority. Source parity still needs maintenance because Rust and WGSL are authored separately.'),
    entry(192,'Traversal completion and receipt success','A spent traversal budget must not certify clear air.',
        ['CPU BrickTrace has distinct Exhausted and Clear',
         'WGSL BrickHit found=false represents both miss and exhaustion',
         'Default test asserts exhaustion explicitly and sample quality bars reject faults',
         'Ignored large receipt prints tallies but contains no failure assertions'],
        'The CPU status distinction is honest and freshly exercised. A successful ignored receipt exit would not itself prove full-frame/spread quality; its printed tallies must be inspected. It was not run. Default sampled tests do assert meaningful bars and retain the known drifting positive control.'),
    entry(193,'Retarget and projection authority','Products own selection/source revisions; map owns deterministic disposable slot projection.',
        ['Retarget requires advanced projection revision for changed selection',
         'Retained keys keep slots and occupancy without refetching source',
         'Refresh stages payloads before writes and leaves projection revision unchanged',
         'Retarget rejects missing/invalid payloads before publication'],
        'The source implements the stated split, with fresh refusal/retained-slot tests. Projection revision is not a content/source revision; refresh can change bytes at the same projection revision by design. Consumers need their product source version too. A single stamp should not be advertised as covering both authorities.'),
    entry(194,'Atlas capacity versus total admission work','Maps are deterministic and bounded by atlas and host limits.',
        ['with_limits checks atlas rows/bytes and pointer texture axes',
         'Pointer storage size uses checked multiplication and fallible reserve',
         'Selection iterators collect into sets before checking key counts',
         'Atlas byte budget does not include pointer-volume bytes',
         'Legacy from_keys uses fixed atlas cap without a host texture-edge admission'],
        'Fresh tests qualify stated atlas limits, not total host memory/work bounds. A small number of widely separated keys can imply a large pointer box. No huge allocation or infinite iterator was attempted. Total pointer budget and admission semantics are broader follow-ups, not a reason to stall the audit.'),
    entry(195,'Central source declarations versus resolved identity','Workspace pins canonically and every member inherits a root path entry.',
        ['Root contains103 explicit members',
         '11 members have no workspace path alias, including Tenant and several ports/hosts',
         'Each inspected canonical first-party git repository has one root branch/rev/tag selector',
         'First-party branch main declarations rely on Cargo.lock for exact revisions',
         'Standalone nested workspaces must restate patches, explicitly documented'],
        'Canonical source selectors are actually consistent in these declarations. Every-member/only-root-knows-location prose is too broad, but absent aliases are not by themselves functional bugs. This is a declaration check, not proof every nested resolved graph or feature closure shares one implementation.'),
    entry(196,'Portable graph verification versus local overrides','cargo_mode separates local locks from a portable graph.',
        ['Local mode uses resolver.lockfile-path and checks portable lock digest in finally',
         'Verify refuses configured source/path/patch/include redirects and external path packages',
         'Verify normally runs locked all-target check; metadata-only deliberately skips it',
         'Setup renames untracked local configs and copies locks, rather than being read-only'],
        'These are concrete mechanisms, not mere naming. Verified output under metadata-only proves provenance only, not compilation. Full helper fixture suite was source-read only: it creates temporary Git repos/Cargo homes/targets outside approved local workspace paths, so it was not executed in this audit. No fixture relocation or production helper change made.'),
    entry(197,'Example build workaround versus actual package','test-web disables incremental for the canvas package.',
        ['Example alias selects profile.dev.package.mere-canvas',
         'Current canvas manifest and lock use pictograph',
         'Active tracked config only sets build-dir',
         'Example also retains stale outside-workspace Signalman and old p2panda version prose'],
        'Q122 is the absent target, scoped to a recommended example. Current historical Windows linker status is untested. Stale prose around extracted/absorbed siblings and old versions should not be mistaken for active source resolution or a live alias.'),
    entry(198,'Manifest dataset projection versus dependency closure','Grouped Cargo export discloses selected manifest declarations.',
        ['Exporter links only endpoints in the selected snapshots',
         'Optional/build/dev/target/alias metadata is retained',
         'External and compiled closure are explicitly excluded',
         'Relations say snapshot versions are not proven to compile together',
         'Revision/relationship ordering is deterministic under tested reorderings'],
        'Four current pure Python tests pass with no external network/process or fixture directories. This is an honestly bounded projection. Git endpoint identity by repository/package is a declaration mapping, not proof that the supplied snapshots match the dependency pinned revisions.'),
    entry(199,'Rust floor and CI success scope','Policy and portable lanes check their named properties.',
        ['Rust-policy matrix runs metadata under1.97.1 and1.99.0 and checks declared rust_version',
         'Its output explicitly distinguishes declarations from compiler compatibility',
         'Portable lane invokes actual workspace all-target checking on1.99.0',
         'Viewer and guest lanes have separate commands/receipts'],
        'Source statements are mostly honest: the metadata lane does not establish an MSRV build. CI files were read, not executed or queried for current run status. A configured upload with if:always and warn-on-missing is evidence retention, not a successful delivery gate by itself.'),
    entry(200,'Build output and shared policy authority','Normal builds and gates follow repository policy.',
        ['Tracked Cargo config sets build-dir to workspace-root/target/build',
         'Audit build commands explicitly override build-dir and target-dir to stable approved paths',
         'rustfmt config is a manual family copy and explicitly says placement does not establish conformance',
         'deny.toml configures custody wrappers but does not execute cargo-deny'],
        'Build-dir and final target-dir are separate output owners; selecting only the final target is insufficient for the local workspace hygiene rule. Formatting and custody configuration are not fresh checks or proof all family copies agree. No formatting/custody sweep ran, and parallel dirty source was preserved.'),
    entry(201,'Retired patch and architecture explanations','Manifest commentary describes current source ownership.',
        ['Armillary/sceno comments still describe git-sibling consumption beside actual path dependencies',
         'Root keeps explicitly retired CubeCL/Burn patch archaeology separate from active patch rows',
         'Runtime patch is active while old cubecl-wgpu and burn-cubecl rows are commented',
         'Murm compatibility patch sources redirect old repository names to canonical current repository'],
        'Distinguish stale explanations, inactive source copies and active selectors. Retained source under support/patches remains in full-source audit scope even when not selected; it is not proof a second live implementation is loaded. Compatibility naming alone does not create a second source when patch identity resolves consistently.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Brick selection and pointer memory bounds differ from the atlas budget',
     'source':'crates/conatus/modulus/src/lib.rs:from_keys, retarget, capacity_fixed; limits.rs',
     'note':'Keys collect fully before count refusal. Pointer dimensions derive from bounding box; pointer byte budget is not tied to max_atlas_bytes. Checked/fallible allocation does not make work small. No huge pointer array or infinite input was attempted. Total memory/admission policy may exceed one-hour diversion and stays documented.'},
    {'title':'Large traversal receipt prints measurements without an asserted pass bar',
     'source':'crates/conatus/modulus/src/traversal_tests.rs:ignored receipt',
     'note':'Default sampled tests contain quality assertions and known drifting positive controls, but the ignored whole-frame/wider-spread receipt only prints tally output. A passing test exit alone is insufficient to promote that larger scope. The ignored receipt was read and not run; no claim that its actual tallies fail.'},
],'title')
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
modulus=sorted(p for p in inventory if p.startswith('crates/conatus/modulus/'))
extra=['Cargo.toml','.cargo/config.toml','.cargo/config.toml.example','scripts/cargo_mode.py',
    'scripts/test_cargo_mode.py','scripts/export_grouped_cargo.py','scripts/test_export_grouped_cargo.py',
    '.github/workflows/portable.yml','.github/workflows/rust-policy.yml','rust-toolchain.toml','rustfmt.toml','deny.toml']
complete=sorted(modulus+extra)
assert len(modulus)==12 and len(complete)==24
assert sum(inventory[p]['chars'] for p in modulus)==98453
assert sum(inventory[p]['chars'] for p in complete)==196729
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
assert all(sha(ROOT/p)==inventory[p]['sha256'] for p in complete)
conatus=[p for p in inventory if p.startswith('crates/conatus/')]
assert len(conatus)==152 and all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in conatus)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==925 and stats['read_chars']==13035948
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/mere/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Modulus12/12 and root Cargo/build/config/helpers12/12 fully read; entire Conatus tree152 source files complete'
notes['next_area']='crates/mere'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['modulus_manifest_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'26 Modulus tests,4 pure exporter tests and2 native observations pass. Q120/Q121 native; Q122 source-confirmed.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'24 more full files and196729 chars. Conatus tree152/152 complete; full Mere objective active.'}
upsert(notes['source_qualifications'],[{'area':'Modulus and root build/config/helper complete review',
    'note':'12 Modulus files plus12 root/build/config/helper files read,196729 chars. Modulus Cargo gate18 unit+8 integration passes,1 ignored receipt not run,0 doctests. Two generated CPU observations pass. Four pure exporter tests pass. Cargo-mode fixtures not run because they create temporary repos/homes/targets outside approved local locations. No actual WGSL, GPU, CI, full portable check or web alias execution.'}],'area')
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes); save('authority_map.json',authority)
doc=old['document']['path']; destination=ROOT/doc; before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only Modulus and root build-source review, 2026-10-16

Modulus12/12 and12 root manifest/build/config/helper sources are fully read,
196,729 additional characters. All152 inventoried source files under
crates/conatus are now complete. Mere coverage is925/3352 files and
13,035,948/36,854,820 characters. Full-source objective remains active.
Production/concurrent work are preserved. Resume at crates/mere/Cargo.toml.

Q120: the public material lookup and traversal reader disagree. A map with
only key[0,0,0] filled7 returns7 at world cells[524288,0,0] and[-524288,0,0].
Nearby outside x8 is air, and public trace at both distant coordinates is
Clear. material_at divides by8 then narrows to i16, wrapping the distant key
to0; the traversal reader bounds pointer coordinates instead. One native test
proves the divergence. Reuse that bounded reader or check before narrowing.
This is a contained removal of duplicated lookup authority; the trace itself
does not inherit this distant alias. No GPU or product pick was executed.

Q121: at direction exactly1e-6 the box clip treats the axis as moving but the
voxel crossing treats it as parallel. Public CPU trace through air toward
solid x4 returns Clear; the adjacent representable direction above it and
unit-direction control hit[4,4,4]. Clip uses abs(d)<threshold, crossing uses
strict signed inequalities. WGSL repeats the mismatch but was not executed.
Align the boundary classification across CPU, WGSL and exact reference, with
signed endpoint/adjacent-float controls. This is a bounded numeric endpoint,
not a demonstrated product-camera or general GPU parity failure.

Q122: the recommended config example's test-web alias applies its historical
incremental workaround to absent package mere-canvas. Current package/lock
name is pictograph. The intended package therefore receives no override.
Active tracked config contains only build-dir; the stale alias is example-only.
Correct targeting is separate from determining whether today's toolchain still
needs that Windows linker workaround. No web alias or historical failure ran.

The Modulus default Cargo gate passes18 unit and8 integration tests, with one
ignored large receipt unrun and zero doctests. Both additional native defect
observations pass with controls. Four pure grouped-exporter Python tests also
pass. CPU traversal is a manually authored WGSL mirror; no actual shader
compilation or GPU execution occurred. The frozen accumulated walk is an
explicit test-only positive control, not another production authority. The
f64 reference is a useful precision instrument under the chosen offset/tie/
budget rules, not independent proof those policies are universally correct.
The ignored large receipt prints tallies without failure assertions; its exit
status alone would not establish whole-frame quality. Default sampled tests
do assert non-vacuous bars and catch the known drifting control.

Retarget/refresh ownership is mostly sound: slots are deterministic, retained
slots survive shrinking, missing payloads refuse before writes, changed
selection needs an advanced projection revision. Refresh can change content
at the same projection revision because source revision remains product-owned.
Do not collapse those two authorities. Atlas capacity is not total memory or
input work: selection collects before count refusal, pointer volume follows
the bounding box, and pointer bytes are outside max_atlas_bytes. No huge
allocation/infinite input was attempted. Total admission/memory policy and
actual GPU parity remain documented larger work within the user's audit limit.

Root declarations have consistent canonical first-party source selectors, but
branch main is not an immutable revision; Cargo.lock owns the selected commit.
The claim every member has a root path alias is false for11 of103 declared
members, though lacking an alias is not itself a runtime bug. Several comments
still describe git-sibling consumption beside current local paths. Inactive
retired vendor patches are provenance, not proof they execute. Those retained
sources remain within full-source coverage, regardless of active selection.

cargo_mode implements real local-lock isolation and portable provenance checks.
Its metadata-only option intentionally does not compile. The fixture suite was
read but not run here because it creates temporary Git repos, Cargo homes and
targets outside approved local workspace locations. No production helper or
fixture was relocated. The exporter honestly discloses declarations among
selected snapshots, including alias/target/optional/build/dev data; it does not
claim a resolved or compiling dependency graph. Rust-policy CI similarly checks
declared MSRV, with compiler lanes separate. CI and full portable verification
were read, not executed or queried for current status.

Tracked build-dir points at workspace-root/target/build; audit build commands
explicitly override it and target-dir to C:/t/cargo-targets/mere. The settings
are separate. Placing formatting or custody policy files does not execute their
gates, and no formatting/custody sweep ran. Owned probe executable/PDB removed;
no isolated target, Cargo home or worktree created. Preservation and exact
execution scopes are in the
[Modulus/manifest review receipt](../../../support/receipts/authority-success-audit/modulus_manifest_review_receipt.json).
'''
assert b'### Read-only Modulus and root build-source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'modulus_manifest_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence=['modulus_probe.rs','run_modulus_probe.py','modulus_library_build.jsonl',
    'modulus_library_build_stderr.log','modulus_default_suite.log','modulus_probe_build.log',
    'modulus_probe_output.log','modulus_probe_receipt.json','root_manifest_declaration_check.json',
    'manifest_export_tests.log','manifest_helper_gate_receipt.json','modulus_manifest_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Modulus and root build-source reading; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_modulus_files':modulus,
    'added_complete_files':complete,'complete_conatus_tree_files':conatus,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'modulus_tests':26,'modulus_test_exit':0,'exporter_tests':4,'exporter_test_exit':0,
    'diagnostic_tests':2,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere source coverage incomplete; no production fixes.',
        'Modulus precision gate runs CPU mirrors, not actual WGSL or GPU.',
        'Ignored large receipt and cargo-mode fixture suite were not executed.',
        'No full portable dependency closure, web alias, MSRV build or CI execution.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in receipt['source_sha256'].items())
save('modulus_manifest_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'modulus_complete':len(modulus),'conatus_tree_complete':len(conatus),
    'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
