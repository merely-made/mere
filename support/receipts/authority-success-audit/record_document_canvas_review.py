"""Record complete document-canvas source review and exact execution scope."""
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
old=load('platen_review_receipt.json');probe=load('document_canvas_probe_receipt.json')
assert probe['tests']==5 and probe['test_exit']==0 and probe['generated_output_removed']
raw=(HERE/'document_canvas_suite.log').read_bytes()
suite=raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
assert '86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/inker/document-canvas/'))
assert len(complete)==13 and sum(inventory[p]['chars'] for p in complete)==250223
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1002 and stats['read_chars']==14015700
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(n,title,source,evidence,scope,size,repair):
    return {'id':f'Q{n:03}','title':title,'source':source,
        'status':'reproduced through current Cargo-built default document-canvas APIs; documented, not repaired',
        'evidence':evidence,'scope':scope,'verification':'document_canvas_review_receipt.json',
        'diagnostic':'document_canvas_probe_receipt.json',
        'repair_triage':{'disposition':'documented; active goal forbids production edits','size':size,'reason':repair}}
findings=[
    finding(140,'Distinct links in one shaping run share a hit rectangle and first-label clicks resolve the later URL',
        'crates/inker/document-canvas/src/text.rs:layout_text_block_with_link_identity_base interaction geometry; types.rs:link_at and interaction_at',
        'Two ordinary same-style links FIRST/SECOND receive distinct retained IDs but exactly equal full-run rectangles. A point inside the first glyph resolves second URL. An intervening plain space still gives equal rectangles. Making the first link bold creates a shaping boundary and resolves first URL correctly. Geometry intersects annotation ranges with the underlying shaping run text_range, then includes the whole run advance.',
        'Native Windows font environment, public layout/packet hit testing and bold-font control. No claim every font/script shapes identically. Smolweb uses the packet typed interaction_at and separately checks topmost retained pointer targets, source-traced only; no hosted pointer, OS accessibility or network navigation executed.',
        'per-cluster annotation geometry; shared hit/semantic ownership correction may exceed one-hour diversion',
        'Use actual cluster/positioned-item spans for each logical annotation and keep disjoint links disjoint, including style boundaries, bidi and wrapping controls. Distinct semantic IDs do not repair overlapping geometry. Do not add a second host-side link rectangle table.'),
    finding(141,'Nested rendered block source indices can name unrelated top-level content or exceed the source array',
        'crates/inker/document-canvas/src/layout.rs:render_group and render_list; types.rs:RenderedBlock::source_block_index and block_at',
        'A quote at top-level0 has second child source_block_index1. block_at correctly returns that child geometry, but EngineDocument.blocks[1] is unrelated outside text. A quote at top-level1 emits first child index1000 beyond the two-element source array. The public field says index into EngineDocument.blocks and block_at advertises point-to-source mapping. top_level_block correctly avoids the collision as control.',
        'Native public nested layout and hit testing. Original fold test already pins child-index reuse and top-level helper protection; no regression claim for that helper. Smolweb accessibility carries a hierarchical path and independently repeats synthetic arithmetic, source-traced only; no current hosted citation/selection misattribution executed.',
        'typed source path plus compatibility/consumer migration may exceed one-hour diversion',
        'Separate original top-level source identity from an explicit nested path. Preserve host path resolution while retiring undocumented decimal packing; renaming an ambiguous integer alone cannot repair source attribution.'),
    finding(142,'Ordered lists lose their ordering marker distinction in the visual packet',
        'crates/inker/document-canvas/src/layout.rs:render_block List dispatch and render_list; types.rs:RenderedBlockKind',
        'Changing only Block::List ordered=true to false yields an exactly equal complete DocumentRenderPacket. Paint-command Debug dumps are also equal. Changed item-text control differs. List dispatch ignores ordered and render_list emits indented child blocks without number/bullet commands or ordering metadata, so a packet consumer cannot recover that distinction.',
        'Native layout and portable paint producer data. Packet equality is exact; paint comparison is Debug-dump equality, not serialized-wire proof. No visual screenshot or GPU render. Uxtree independently preserves an ordered description, so visual and semantic projections have different information.',
        'contained list-marker lowering, with nested/source identity controls',
        'Render appropriate markers or retain explicit visual list metadata through the packet, keeping decorative numbering out of authored link labels and source identity. Preserve multi-block item grouping and nested-list behavior.'),
    finding(143,'Body paragraph layout ignores the resolved role spacing-above setting',
        'crates/inker/document-canvas/src/layout.rs:render_block Paragraph and render_paragraph; style_sheet.rs:resolve',
        'Setting roles.body.spacing_above80 resolves to80 but leaves a one-paragraph packet exactly equal to baseline. Increasing body spacing_below changes geometry; code-role spacing_above80 moves its block by80. Paragraph dispatch passes only spacing_below and its wrapper hardcodes spacing_above0.',
        'Native public style/layout with ordinary finite settings and sibling-role controls. No host settings UI or persistence executed. Default body spacing-above is0, so default appearance remains unaffected.',
        'contained resolved-role field threading',
        'Thread resolved spacing_above through paragraph layout consistently with other text roles. Existing default-value tests should not stand in for user-configurable role-field coverage.'),
    finding(144,'Changing adornment presentation renumbers later retained logical-link identities',
        'crates/inker/document-canvas/src/text.rs:flatten_link; layout.rs:flatten and reserve_link_identities',
        'Same document with an empty-label link followed by a visible later link gives that later link different SemanticInteractionIds under SchemeArrow versus None. The first annotation exists only when its decoration creates nonempty painted text. Empty custom glyph repeats None; nonempty first-label control retains the later ID across the style change. Identity allocation counts flattened visible annotations, so presentation affects the logical ordinal.',
        'Native default public layout and retained tokens, with empty-custom-glyph and nonempty-label controls. Smolweb set_style_sheet preserves document/focus state and invalidates geometry, source-traced only. No hosted stale-action retarget or security bypass is claimed; session revision guards require full review.',
        'shared source-owned link allocation; retained-consumer adoption may exceed one-hour diversion',
        'Allocate/reserve logical identities from authored link occurrences independently of decoration and visible rectangles. Keep removed/zero-width interactions noninteractive while retaining later IDs; preserve fold and reflow guarantees and review session revision/currentness rules.'),
]
upsert(notes['findings'],findings,'id')
def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'All13 document-canvas files read;86 default library tests and5 native observations',
        'verification':'document_canvas_review_receipt.json'}
entries=[
    entry(240,'Font selection and render bytes','The shipped face is the face Parley shaped, without render-side re-resolution.',
        ['FontInterner keys by Blob allocation ID and collection index',
         'GlyphRun stores FontFaceId; FontTable carries actual FontData',
         'Paint producer reads matching sidecar bytes once per face and emits font resources'],
        'Actual reuse and explicit authority, with fresh original per-run byte/index tests. Requested family labels are debug/a11y metadata, not render lookup. Synthetic interner test distinguishes collection faces. No glyph/font-byte mismatch reproduced.'),
    entry(241,'Packet portability and complete transport','DocumentRenderPacket is serializable; PaintList is the self-contained form.',
        ['Serializable packet omits font bytes deliberately',
         'LaidOutDocument couples packet with a nonserializable FontTable',
         'PaintList owns font/image bytes; netrender shim delegates shared translator'],
        'The source explicitly discloses this boundary, avoiding a silent serde-skip success path. Missing sidecar faces paint placeholders, not a checked error. This default gate proves producer data and original font tests, not IPC replay, netrender translator or GPU rendering.'),
    entry(242,'Hit geometry versus logical link identity','Retained packet supplies both click geometry and semantic grouping.',
        ['Logical link IDs distinguish same-URL occurrences and join wrapped rectangles',
         'Interaction geometry approximates underlying whole shaping runs',
         'link_at and typed interaction_at use last containing region'],
        'Q140 shows identity correctness is insufficient for click correctness. Smolweb pointer targeting checks current topmost identity and can refuse occluded rectangles, but cannot restore the intended geometry. A parallel host hit table would create another authority rather than fix this seam.'),
    entry(243,'Source attribution and nested layout identity','source_block_index names EngineDocument.blocks content for selection/citation.',
        ['Quote/group pack parent*1000+child',
         'List packs parent*1000+item*100+block',
         'Tables deliberately keep parent source index for cells',
         'Smolweb semantic path code independently repeats the decimal arithmetic'],
        'Q141 proves the simple-index promise fails for nested children. top_level_block explicitly avoids the known collision and its original control passes. Consumer hierarchical paths are useful adaptation, but packed integers and repeated formulas are a hidden shared ABI rather than typed shared source identity.'),
    entry(244,'List visual and semantic projections','Every Block variant has a document rendering.',
        ['List ignores ordered flag and becomes Group of child blocks',
         'Paint producer only recurses Group children',
         'Uxtree has List/ListItem roles and ordered description'],
        'Q142 proves visual information loss despite available semantic information. Render-every-variant includes placeholder badges for unknown kinds and does not prove every semantic feature survives. Source projection and lowering should preserve declared distinctions rather than let each host infer them.'),
    entry(245,'Typography/spacing authority','One resolved role carries the full typography/spacing policy.',
        ['StyleSheet resolves global inheritance and role overrides',
         'Heading/code/metadata/badge pass spacing above and below',
         'Paragraph wrapper hardcodes above0',
         'TextBaseStyle also has public standalone defaults'],
        'Q143 proves a configurable field silently ignored. Standalone helper defaults are a separate entry contract, not necessarily a competing host theme; actual document entry uses the sheet. Image placeholder metrics still use global body size/shared leading instead of resolved body role, a source-qualified scope to document.'),
    entry(246,'Semantic allocation and presentation policy','Logical-link identity is retained across geometry changes.',
        ['Document allocator increments by flattened link annotation counts',
         'Hidden fold blocks reserve the counts so open/close keeps later IDs',
         'Empty-label annotation existence depends on adornment text'],
        'Fold/reflow original tests pass, but Q144 demonstrates presentation-derived IDs on a style change. Current Smolweb style setter preserves focus/session ownership. Full stale-action defenses are not qualified by this library probe; source-based allocation needs a consumer migration review.'),
    entry(247,'Generic font admission and native font availability','Explicit generic ledger refuses invalid batches before changing caller mappings.',
        ['register_generic_fonts validates all entries in an isolated context first',
         'Accepted ledger then registers into caller FontContext',
         'Raw supplemental fonts register separately and can fall back to system fonts',
         'Normal document entry creates a fresh LayoutEnvironment'],
        'Fresh original refusal test preserves prior generic mappings on invalid late entry. Native default fonts worked in86 tests/probes; that does not prove font availability in browser guests. NoFontResolver text-still-renders prose needs target/font-environment qualification; no wasm failure executed.'),
    entry(248,'Source presentation versus source facts','Reader policy changes display while preserving authored data.',
        ['Presented inline/block facts remain in EngineDocument',
         'Respect/Reader chooses run foreground/background/underline',
         'Block alignment/indent is carried independently',
         'Menu targets become typed Link/Submit and in-page targets stay distinct'],
        'Implemented reuse rather than stripping source facts. Shared policy affects paint only in its documented inline scope; do not infer Reader discards every block alignment. Typed submissions/in-page/folds are not fetch URLs. Hosts must use the typed interaction seam where commands differ.'),
    entry(249,'Document layout and accessibility owners','Uxtree owns semantic projection; document-canvas owns only visual output.',
        ['Uxtree independently projects EngineDocument to AccessKit structure',
         'Document-canvas now retains LinkSemantics and typed regions',
         'Smolweb DocumentSession builds semantic snapshots from source plus measured packet and retains IDs'],
        'Introductory visual-versus-semantic split is now too absolute. These are separate projection algorithms over shared facts, with different identifiers and list/table representations; not independent durable source stores. Host/session namespace, revision and action authority still require their full source review.'),
    entry(250,'Windowing and bounded work','A visible packet can render a viewport-sized band rather than a giant texture.',
        ['window_rect filters/ translates whole blocks and interactions',
         'Group children filter recursively',
         'An intersecting Text clones all runs, even off-band lines',
         'Packet content width may exceed viewport for NoWrap/tables despite stale type comment'],
        'Original window coordinate tests pass. Texture extent is bounded by supplied band, but CPU cloning/glyph work for a very tall single paragraph is not bounded by visible lines here. No large allocation or performance benchmark executed; broader culling is documented follow-up.'),
    entry(251,'Table layout and source/lowering reuse','Cells reuse ordinary text and interaction machinery.',
        ['Measure pass has throwaway interner and does not allocate document IDs',
         'Final cells reserve IDs and align glyphs plus hit regions together',
         'Narrow/unbreakable tables explicitly overflow rather than overlap adjacent cells'],
        'Original table controls pass, including ragged cells, long links and alignment. That does not erase Q140 whole-run approximation within a cell. Measurement is an intentional second layout pass, not a competing durable authority. No full CSS/table model claimed.'),
    entry(252,'Image/resource and color admission','Host supplies decoded image bytes and theme primitives.',
        ['DecodedImage is public width/height/RGBA data, producer trusts it',
         'Producer binds by URL, deduplicates per list and contains image in placeholder area',
         'Palette says premultiplied RGBA; default placeholder_image RGB exceeds alpha',
         'Font/image keys and generation start at zero per list'],
        'Normal decoded-image original test passes. Malformed resource byte count, downstream color convention, and cross-list key/cache scope require their owner checks; no GPU/resource-cache bug is inferred from local defaults alone. Exact sidecars are present but structural serde is not automatic admission.'),
    entry(253,'Fresh test receipt versus optional/backend claims','A passing default library suite qualifies the document producer.',
        ['86 original library tests execute native default layout/text/paint data',
         'Five generated observations exercise public current Cargo library',
         'Three optional netrender shim tests are feature-gated and only read',
         'Netrender backend uses shared translate_paint_list rather than another local renderer'],
        'Default CPU evidence is fresh and scoped. The native font environment, source fixtures and exact dependency artifact hashes are recorded. No netrender feature, GPU, physical assistive technology, remote/wasm or hosted-session proof claimed. Old Genet ancestry and graph-canvas/gpui prose do not change current Mere source ownership.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Text windowing does not bound off-band glyph work inside a tall paragraph',
     'source':'crates/inker/document-canvas/src/types.rs:window_rect/window_block; paint_list.rs:emit_glyph_run',
     'note':'Whole intersecting Text clones every run; producer emits all glyphs. Band texture extent and visible-line work are distinct bounds. No huge synthetic page or renderer benchmark attempted. Cluster/line culling and retained font resources may exceed one-hour diversion; document instead of delaying audit.'},
    {'title':'Document producer resource and color conventions need downstream owner admission review',
     'source':'crates/inker/document-canvas/src/paint_list.rs; style.rs; types.rs:DecodedImage',
     'note':'Public image bytes not checked against dimensions; per-list resource keys/generation reset; premultiplied default placeholder_image has RGB above alpha. Normal matching resources work. No malformed upload, cross-list cache or renderer color defect reproduced; avoid presenting source observations as tested backend failures.'},
    {'title':'Nested source-path and logical-link allocation migrations need consumer/revision review',
     'source':'crates/inker/document-canvas/src/layout.rs/text.rs/types.rs; crates/system/document-lanes/src/smolweb/a11y.rs',
     'note':'Q141 and Q144 demonstrate public mismatches. Current host code carries hierarchical paths, repeats decimal source arithmetic and retains focus across style changes. Full stale-action/currentness and typed path adoption can exceed one-hour diversion. Keep top_level_block guard and session defenses while documenting the owner rule.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/inker/inker/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete document-canvas13/13 source review, including optional backend source'
notes['next_area']='crates/inker/inker'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['document_canvas_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'Current default Cargo library build and86 original tests pass;5 generated native observations pass.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'13 complete files250223 chars added. Optional netrender source/tests read, not executed. Consumer excerpts are source tracing only.'}
upsert(notes['source_qualifications'],[{'area':'Complete document-canvas source review',
    'note':'All13 inventoried files250223 chars fully read. Default current Cargo library build and86 library tests pass. Five native public observations with controls pass under current Windows fonts. Packet list equality exact; paint commands compared by full Debug dumps, not serialized-wire equality. Netrender feature/3 shim tests, GPU, hosted document session, browser/wasm and OS a11y not executed. Source-traced consumer excerpts get no full-file coverage credit. Probe executable/PDB removed; no isolated cache.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
consumers={
    'crates/system/document-lanes/src/smolweb.rs':{'sha256':sha(ROOT/'crates/system/document-lanes/src/smolweb.rs'),
        'read_scope':'style setter and retained link identity/pointer/current typed-hit excerpts',
        'counts_as_complete_file_coverage':False},
    'crates/system/document-lanes/src/smolweb/a11y.rs':{'sha256':sha(ROOT/'crates/system/document-lanes/src/smolweb/a11y.rs'),
        'read_scope':'hierarchical block-path lookup excerpt and synthetic-index symbol traces',
        'counts_as_complete_file_coverage':False}}
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only document-canvas complete source review, 2026-10-16

All13 inventoried document-canvas source/manifest files are read,250,223 more
characters. Repository coverage is1002/3352 files and14,015,700/36,854,820
characters. Full-source goal remains active. Resume at crates/inker/inker/
Cargo.toml. Production/concurrent source remains unchanged. Authority entries
A240-A253 distinguish actual shared ownership, copied projection rules and
execution boundaries.

Q140: two ordinary same-style links receive distinct semantic IDs but equal
whole-shaping-run hit rectangles. Clicking inside FIRST resolves SECOND's URL.
An intervening space does not prevent it; bold FIRST creates a shaping boundary
and the control resolves FIRST correctly. The code tests each annotation against
underlying run text_range and uses the whole run advance. Correct link identity
does not repair wrong click geometry. Per-cluster/item bounds and shared typed
hit ownership need a careful repair, potentially beyond the one-hour diversion.
Native current Windows fonts/public packet APIs are exercised, not a browser
pointer or network request.

Q141: nested source indices contradict their public type meaning. block_at on
quote0's second child returns source index1, which names unrelated top-level
outside content in EngineDocument.blocks. Quote1's first child carries1000,
outside a two-element source array. top_level_block deliberately avoids this
known collision, and its control remains correct. Smolweb uses a hierarchical
path but independently repeats the decimal packing. Preserve that adaptation
while replacing ambiguous integers with explicit source/path identity; this is
a broader contract/consumer migration. No current hosted citation failure tested.

Q142: changing only List ordered=true to false produces an exactly equal visual
packet. Portable paint-command Debug dumps are also equal, not a serialized-wire
proof. A changed-text control differs. List dispatch discards ordered; the packet
has no marker/ordering field, and paint only recurses the resulting Group. Uxtree
retains an ordered description separately. Render markers or retain the visual
distinction without putting decoration into authored link labels/source IDs.

Q143: roles.body.spacing_above80 resolves to80 but yields the same paragraph
packet as0. Body spacing_below and code spacing_above controls change geometry.
The paragraph wrapper hardcodes0 instead of passing the resolved above spacing.
This is a contained configuration-field repair; defaults remain unaffected.

Q144: hiding an earlier empty-label link's adornment renumbers a later retained
link ID in the same document. SchemeArrow makes the empty annotation nonempty;
None or an empty custom glyph removes it before document identity allocation.
A nonempty first-label control retains the later ID across styles. Allocate or
reserve identities from authored occurrences, separately from visible rectangles.
Smolweb's style setter preserves document/focus state and invalidates geometry,
source-traced only; no hosted stale action or security bypass is claimed. The
session's revision/currentness rules need full review before a consumer change.

Several ownership claims are actually implemented well. FontInterner records
the exact Parley Blob allocation plus collection index, glyph runs refer to that
sidecar, and paint ships those exact bytes. Original per-run/index tests pass.
Requested family/weight/style are not used to re-resolve render bytes. Packet
serialization explicitly omits font bytes and calls PaintList the self-contained
transport form, avoiding a hidden serde-skip success path. Netrender is a thin
shared-translator shim, not another local text renderer. The default gate does
not execute that feature or prove IPC/GPU rendering. Native font availability
does not establish browser guest font availability.

Logical link identities distinguish repeated URLs and survive wrapping/fold
changes in the existing tests. Hidden blocks reserve IDs. Q144 shows that this
policy still depends on style-derived annotation existence. Visual and semantic
projections remain distinct implementations: Uxtree projects EngineDocument,
document-canvas now retains LinkSemantics, and Smolweb builds session semantic
snapshots using source plus measured packet. The old visual-only/semantic-only
intro is too absolute; source, layout, namespace and action authority must be
named separately. Those excerpts do not count as full reading of document-lanes.

Windowing correctly filters/translates blocks and interactions in original tests,
but an intersecting Text block still clones/emits every off-band run. Band texture
extent is not a bound on glyph work for a tall single paragraph. No huge allocation
or performance benchmark ran. Image byte-count admission, premultiplied palette
defaults and per-list key/generation scope require downstream owner review;
normal image/font sidecars work, and no backend failure was inferred. Table
measurement intentionally uses a throwaway font interner and allocates no document
IDs, then final layout shares ordinary glyph/interaction machinery. Its original
ragged/overflow/alignment controls pass without establishing full CSS tables.

Fresh default Cargo library build and86 original library tests pass. Five native
public observations with controls pass. Three optional netrender shim tests were
read but not executed. No GPU, renderer, hosted session, browser/wasm, native
adapter or physical assistive technology ran. Owned probe executable/PDB removed;
no isolated target, Cargo home or worktree created. Exact hashes, consumer trace
scope and test evidence appear in the
[document-canvas review receipt](../../../support/receipts/authority-success-audit/document_canvas_review_receipt.json).
'''
assert b'### Read-only document-canvas complete source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'document_canvas_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['document_canvas_probe.rs','run_document_canvas_probe.py','document_canvas_probe_receipt.json',
    'document_canvas_probe_build.log','document_canvas_probe_output.log','document_canvas_library_build.jsonl',
    'document_canvas_library_build_stderr.log','document_canvas_suite.log','document_canvas_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete document-canvas source reading; full Mere objective incomplete',
    'production_edits_this_pass':False,'new_findings':[f['id'] for f in findings],
    'authority_entries':[e['id'] for e in entries],'coverage':stats,'added_complete_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'source_traced_consumers':consumers,'next_read':next_read,'default_library_tests':86,'default_test_exit':0,
    'diagnostic_tests':5,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere objective incomplete; no production changes.',
        'Native default CPU/font/layout/paint data only; optional netrender feature and three tests unexecuted.',
        'No renderer/GPU, current hosted session, browser/wasm, native adapter or physical AT execution.',
        'Consumer excerpts are source tracing only, not full-file coverage or native host proof.',
        'Paint command equality in list probe is Debug-dump equality; packet equality is exact.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('document_canvas_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
