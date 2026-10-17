"""Record complete Nisus/Tenant reading, separating source findings from executed gates."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
def upsert(items, values, key):
    for value in values:
        at = next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None:
            items.append(value)
        else:
            items[at] = value

old = load('numen_review_receipt.json')
suite = load('nisus_library_suite_receipt.json')
assert suite['exit']==0 and sha(HERE/'nisus_library_suite.log')==suite['log_sha256']
assert '6 passed; 0 failed' in (HERE/'nisus_library_suite.log').read_text()
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in suite['source_sha256'].items())
notes,authority = load('review_notes.json'),load('authority_map.json')
findings = [{
    'id':'Q116','title':'LightBlock validates positive-infinite intensities despite its finite-value contract',
    'status':'source-confirmed public validation defect; no native Tenant probe; documented, not repaired',
    'source':'crates/conatus/tenant/src/light.rs:LightBlock::is_valid; frame.rs:Tenant::set_lights',
    'evidence':'The is_valid contract says every value is finite. Sun, ambient and point intensities are admitted with intensity>=0 only; positive infinity satisfies that predicate. Their colors/positions are checked with is_finite and point radius has a separate finite check, so the missing intensity guard is specific. set_lights trusts is_valid before forwarding intensities to lights and ambient. The source test rejects NaN ambient but does not test positive infinity.',
    'scope':'All eight Tenant source files read. No Tenant build, value probe, adapter, GPU frame or consumer execution was performed. This is source-confirmed boolean admission, not evidence of a rendered artifact, device loss or observed shader failure.',
    'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained shared intensity admission repair',
        'reason':'Require each intensity to be finite and non-negative before state mutation, retaining existing no-change refusal semantics. Include positive/negative infinity and finite controls for all three light kinds. Broader normalization, transform and renderer gates remain separately documented.'},
    'verification':'nisus_tenant_review_receipt.json'}]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'Nisus2/2 and Tenant8/8 source read; six Nisus tests pass; Tenant source only',
        'verification':'nisus_tenant_review_receipt.json'}
entries = [
    entry(168,'Revision-gated chunk writes','Nisus accepts a patch atomically against its current revision.',
        ['Expected revision checked before enumeration', 'All coordinates staged in a BTreeMap before writes',
         'Last edit to a cell wins; material equality removes noops',
         'Revision overflow is refused before cell mutation', 'Noop patch retains revision and returns empty changes'],
        'These are actual shared value mechanics. Six current library tests pass, including stale/out-of-bounds/batch/noop controls and revision/effective changes. This does not certify panic-atomic generic Clone or a durable product transaction. No distinct occupancy implementation was found in these two files.'),
    entry(169,'Chunk facts versus derived collision and dirt','Product owns chunk identity and occupancy policy; runtime owns collider updates.',
        ['VoxelChunk equality/serialization omit dirty regions',
         'Custom deserialization calls from_cells and validates extent/cell count',
         'VoxelPatch.occupancy_edits compares caller-supplied occupancy before/after',
         'Coordinates use Y/Z/X record order; comment explicitly excludes a GPU ABI'],
        'The code preserves the stated separation. Material-only changes do not become collision edits; draining dirt leaves record equality unchanged. Tests qualify these value paths, not a product save or refreshed collider. VoxelPatch is public serializable data rather than a validated externally committed patch capability.'),
    entry(170,'Tenant host device ownership','The renderer borrows the host device, encoder and target.',
        ['HostDevice carries cloned wgpu handles', 'Tenant::new calls Context::from_host',
         'encode takes a caller encoder and target', 'FrameReport forwards the lower renderer internal submission count',
         'DeviceNeeds describes baseline/default requirements and runtime degradation'],
        'The wrapper contains actual same-device plumbing, not a second device request. Dependency implementation and capability fallback are pinned kiss3d behavior beyond this Mere source slice. No fresh Tenant/GPU build or frame was executed, so neither zero submissions nor baseline capability fallback is newly qualified.'),
    entry(171,'Lighting input admission','LightBlock says every value is finite before replacement.',
        ['Colors/directions/positions have explicit finite checks',
         'Point radius is positive and finite', 'Intensity checks use >=0 without is_finite',
         'set_lights returns true and forwards the block after that guard'],
        'Q116 is a source-confirmed false-valid path for positive infinity in all intensity slots. NaN-negative controls alone do not qualify the finite contract. Nonzero sun direction is not a guarantee finite f32 normalization remains usable for extreme finite values; that is an additional unprobed numeric candidate.'),
    entry(172,'Transform validity versus usable derived transforms','Body placement and camera data are checked before use.',
        ['Pose.is_finite checks components, then Pose.parts normalizes a quaternion',
         'Camera.is_valid checks finite matrices, orthonormal basis, affine row and ordered finite planes',
         'FixedCamera constructs Pose3 and inverts clip_from_world',
         'Finite projection is not separately required to be invertible; reflection determinant is not checked'],
        'Zero or extreme finite quaternion norm, singular projection, reflected view and normalized sun direction need separate admission qualification. They are source candidates, not fresh native failures. Do not equate finite components with valid normalization/inversion or ordinary camera tests with all transform inputs.'),
    entry(173,'Frame recording and export authority','encode returns a report for a recorded frame and exports the last frame resources.',
        ['encode explicitly refuses target format/samples, prepass size and missing camera',
         'Prepass method documents depth format/usage requirements and forwards to the lower Window',
         'exports forwards shadow/depth/light handles and WGSL layout source',
         'Caller must submit before recording the next frame'],
        'Success means recording through the backend, not queue completion or displayed pixels. Format/usage validation delegated below the wrapper must be inspected before alleging missing downstream validation. Export freshness and writer lifetime are caller sequencing contracts rather than durable graph authority.'),
    entry(174,'GPU frame test versus actual device execution','The receipt demonstrates depth composition on a real adapter.',
        ['host returns None for adapter or device admission failure and test returns with a skip message',
         'Test performs actual encoded rendering, caller submission and pixel readback when a host exists',
         'Red palette/alpha controls and split-depth occlusion assertions exercise visible behavior',
         'Test checks zero internal submissions and host-device survival after Tenant drop'],
        'Source contains a meaningful real-device test rather than a synthetic success flag. It was not run in this pass. A green aggregate count would still require evidence that its adapter branch ran. This does not prove glTF animation, every tonemap, capability degradation or product-headed composition.'),
    entry(175,'Mesh/material wrappers versus hidden backend types','Tenant presents plain arrays and hides kiss3d types from consumers.',
        ['Palette conversion deduplicates sRGB bytes in first-seen order',
         'Mesh validation checks triangles, count, palette indices and finite positions',
         'Rigid meshes and glTF use the same scene/material backend',
         'set_mesh replaces the scene node but retains Body.player from a former glTF body'],
        'The public type boundary is real. Retained animation player ownership after replacing glTF with a rigid mesh needs a specific behavioral test before claiming active animation still targets the displayed body. Palette/normal arithmetic extreme inputs and backend glTF import are not qualified by ordinary CPU source tests.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Nisus max_edits limits accepted batches after unbounded iterator collection',
     'source':'crates/conatus/nisus/src/lib.rs:VoxelChunk::apply_edits',
     'note':'The entire iterator is collected before checking its length against max_edits. This correctly refuses oversized finite batches before mutation but is not an enumeration/allocation work bound, and can consume arbitrarily much caller-supplied iteration. No infinite iterator or large allocation was attempted. Decide whether exact requested count or bounded consumption is the intended API contract before promoting a resource-cap claim.'},
    {'title':'Tenant finite transform components do not establish valid normalization or inversion',
     'source':'crates/conatus/tenant/src/body.rs:Pose::parts; camera.rs:Camera::is_valid and FixedCamera::new; light.rs:place',
     'note':'Finite quaternion components can have zero/overflowing squared norm; finite orthonormal view can be a reflection; projection may be singular; finite nonzero sun direction can have an unusable f32 norm. Source tracks admission assumptions separately from derived values. No Tenant build or numeric probe executed. Define valid input semantics and exact pinned backend behavior before a broader repair, which may exceed the one-hour audit diversion.'},
    {'title':'Tenant mesh replacement retains the previous glTF animation player',
     'source':'crates/conatus/tenant/src/frame.rs:Body, set_mesh, play, advance',
     'note':'set_mesh removes/replaces the node but leaves player unchanged. A former glTF animation may keep references to detached model nodes while play still reports success. No glTF fixture/runtime was executed. Confirm backend player ownership and choose replacement semantics before changing it.'},
],'title')
inventory,coverage = load('source_inventory.json')['files'],load('read_coverage.json')
complete = sorted(p for p,v in inventory.items() if p.startswith(('crates/conatus/nisus/','crates/conatus/tenant/'))
    and coverage.get(p,{}).get('sha256')==v['sha256'] and coverage[p]['through']==v['chars'])
assert len(complete)==10 and sum(inventory[p]['chars'] for p in complete)==63975
assert all(sha(ROOT/p)==inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==878 and stats['read_chars']==12565080
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p,v in sorted(inventory.items()) if p.startswith('crates/conatus/conatus/')
    and coverage.get(p,{}).get('through',0)<v['chars'])
next_read = {'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area'] = 'Numen16/16, Nisus2/2 and Tenant8/8 fully read'
notes['next_area'] = 'crates/conatus/conatus'
notes['checkpoint'] = {'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['numen_review_receipt.json','nisus_tenant_review_receipt.json'],'next_read':next_read,
    'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'Numen65 and Nisus6 library tests pass;3 Numen observations reproduce Q114/Q115. Q116 source-confirmed, Tenant not executed.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'26 additional full files and214229 chars this continuation. Full Mere review remains active.'}
upsert(notes['source_qualifications'],[{
    'area':'Nisus and Tenant completed source review',
    'note':'All10 inventoried files read,63975 chars. Six current Nisus library tests pass. Tenant is source-reviewed only; no Tenant build, adapter/frame, glTF or numeric probe. Source-confirmed Q116 light intensity admission. Broader transform, animation replacement and input iterator concerns stay qualified; no production changes or isolated cache.'
}],'area')
save('review_notes.json',notes)
save('authority_map.json',authority)
doc = old['document']['path']
destination = ROOT/doc
before = destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition = '''

### Read-only Nisus and Tenant complete source review, 2026-10-16

Nisus2/2 and Tenant8/8 source files are fully read,63,975 additional characters.
This continuation completes26 files and214,229 characters including Numen.
Mere coverage is878/3352 files and12,565,080/36,854,820 characters. The full-source
goal remains active. Production/concurrent work are unchanged. Resume with
crates/conatus/conatus, then the remaining adjacent spatial owners.

Q116: LightBlock::is_valid promises every value is finite, but admits all three
light intensity slots with >=0 only. Positive infinity passes. Colors/directions/
positions and point radius have explicit finite checks, making the missing
intensity guards specific. set_lights trusts this result and forwards intensity
to backend lights/ambient. The source test rejects NaN ambient but does not cover
positive infinity. This is source-confirmed validation behavior; no Tenant build,
numeric probe, adapter, frame or shader failure was executed. The contained
repair is finite/non-negative intensity admission before mutation, retaining
the documented unchanged-state refusal contract.

Nisus has a clear shared owner: chunk storage/revision/edit mechanics, with
product-owned durable identity/occupancy and runtime-owned collider refresh.
It stages/validates coordinates before changes, handles duplicate cells in a
stable order, keeps noops at the same revision, and checks revision overflow
before mutation. Material-only changes do not become occupancy edits. Dirt is
excluded from record equality/serialization, and deserialization calls the
validated constructor. All six current library tests pass. These qualify value
mechanics, not product durable saves or collider refresh. Its max_edits check
follows full iterator collection, so an accepted-batch cap is not a work/allocation
cap. That distinction is documented without an infinite or huge input probe.

Tenant contains real host-device plumbing: Context::from_host, caller encoder
and target, forwarded internal-submission report, plain public arrays and wgpu
handles. It does not request a second device in its production wrapper. Pinned
kiss3d behavior owns fallback, rendering, material/animation and lower validation.
Frame success means recorded commands, not queue completion or visible pixels.
The frame test source is substantive: actual rendering/readback, palette and
split-depth controls, zero-submission check and host-device survival. It also
returns successfully with a skip message when adapter/device admission fails.
That test was read, not run here; no fresh GPU or product proof is claimed.

Hairier numeric/ownership issues remain qualified: finite quaternion components
do not ensure usable normalization; camera validation admits finite singular
projections and does not distinguish reflected orthonormal views from proper
rigid rotations; finite nonzero sun direction may normalize poorly at extreme
values. Mesh replacement also retains a former glTF body's animation player
after removing its displayed node. No transform, glTF or replacement probe ran.
Exact pinned-backend behavior and replacement semantics should be resolved in
their own work, within the user's limit on audit diversions. Do not infer absent
downstream prepass validation from a wrapper that delegates it.

The fresh execution total for this continuation is71 library tests (65 Numen,
6 Nisus) plus3 native Numen defect observations. Q114/Q115 are reproduced;
Q116 is source-confirmed. No Tenant execution or full workspace suite ran.
No isolated target, Cargo home or worktree was created; normal Cargo output
uses C:/t/cargo-targets/mere. Preservation and gate details are in the
[Nisus/Tenant receipt](../../../support/receipts/authority-success-audit/nisus_tenant_review_receipt.json).
'''
assert b'### Read-only Nisus and Tenant complete source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'nisus_tenant_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence = ['run_nisus_suite.py','nisus_library_suite.log','nisus_library_suite_receipt.json','nisus_tenant_diff_check.txt']
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Nisus/Tenant source review; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'nisus_library_tests':6,'nisus_library_test_exit':0,'tenant_execution':False,
    'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere source coverage incomplete; no production fixes.',
        'Tenant validation and rendering code were source reviewed only.',
        'No product, GPU, glTF, hosted frame or full workspace suite executed.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in receipt['source_sha256'].items())
save('nisus_tenant_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'added_files':len(complete),'findings':len(notes['findings']),
    'authority_entries':len(authority['entries']),'next_read':next_read}))
