"""Record complete Nematic review, actual reparses and feature-qualified gates."""
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
def log(n):
 raw=(HERE/n).read_bytes()
 return raw.decode('utf-16' if raw.startswith(b'\xff\xfe') else 'utf-8-sig')
old=load('document_lanes_review_receipt.json');probe=load('nematic_probe_receipt.json')
assert probe['tests']==9 and probe['test_exit']==0 and probe['generated_output_removed']
suite=log('nematic_default_suite.log');no_default=log('nematic_no_default_suite.log')
assert '237 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert '3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out' in suite
assert '234 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out' in no_default
assert 'feed::tests::an_entry_address_renders_the_entry_as_its_own_document' in no_default
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in probe['libraries'].values())
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p in inventory if p.startswith('crates/nematic/nematic/'))
assert len(complete)==75 and sum(inventory[p]['chars'] for p in complete)==452682
assert all(coverage[p]['sha256']==inventory[p]['sha256'] and coverage[p]['through']==inventory[p]['chars'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1112 and stats['read_chars']==15047472
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
notes,authority=load('review_notes.json'),load('authority_map.json')
def finding(n,title,source,evidence,scope,size,repair):
 return {'id':f'Q{n:03}','title':title,'source':source,
  'status':'reproduced through current default Cargo-built Nematic APIs; documented, not repaired',
  'evidence':evidence,'scope':scope,'verification':'nematic_review_receipt.json','diagnostic':'nematic_probe_receipt.json',
  'repair_triage':{'disposition':'documented; preserve production and original test source','size':size,'reason':repair}}
findings=[
finding(160,'HTML active-URL filtering accepts control spellings that normalize to javascript',
 'crates/nematic/nematic/src/html.rs:safe_url and Converter::push_inline',
 'Ordinary javascript: is refused, but HTML entity-decoded LF, TAB or CR inside java...script makes the homemade scheme recognizer treat it as no scheme and accept the link without a diagnostic. The current url::Url parser canonicalizes each admitted target to scheme javascript. An ordinary HTTPS control remains accepted. This contradicts the module claim that active URL schemes never reach the portable model.',
 'Native HTML parser/lowerer plus actual URL canonicalization, three controls and HTTPS/refusal controls. No script, browser, export click or host navigation executed; no XSS execution or current host policy bypass is claimed.',
 'contained canonicalization/admission correction; host-wide URL policy review can exceed one-hour diversion',
 'Classify the canonical URL form using the existing parser, with an explicit base/relative-reference policy, before applying the allowlist. Keep HTML entity decoding, control normalization and export/host navigation policy distinct. Do not use a second ad hoc colon-prefix check as URL authority.'),
finding(161,'HTML whitespace-only text nodes disappear between inline elements',
 'crates/nematic/nematic/src/html.rs:collapse_whitespace and push_text',
 '<em>red</em> <em>fox</em> lowers to the plain text redfox. Plain red fox and red followed by a styled fox retain their source space. collapse_whitespace only appends a pending trailing space when output is already nonempty, so a whitespace-only DOM text node vanishes between the adjacent inline containers.',
 'Native HTML lowering and public to_text with two text-bearing whitespace controls. No layout screenshot, browser rendering or full CSS whitespace claim.',
 'contained inline whitespace normalization/state correction',
 'Retain an inter-node separator in normal-flow inline content and trim only at paragraph/block edges. Cover adjacent links/emphasis/strong/code and whitespace-only DOM nodes, while preserving preformatted handling.'),
finding(162,'The native-default Djot parser silently flattens lists and quotes and drops tables',
 'crates/nematic/nematic/src/knot/djot.rs:parse_djot_knot_body_validated; inker routing default; Nematic manifest and design phase5',
 'Ordinary Djot quote/list fixtures retain words as top-level paragraphs without their containers. A table emitted in the writer-supported pipe shape produces blocks=[]; all three have empty diagnostics. The CommonMark compatibility quote control preserves Quote, and the default text/x-knot route selects this Djot engine. Parser container events other than its small selected subset fall through to the wildcard.',
 'Native default parser, actual default route selection and compatibility control. No persisted user file, hosted note editor or GPU rendering executed. The source still labels parts as PoC, while the manifest/design/default route claim native parity/default migration.',
 'stack-based Djot event lowering and projection qualification can exceed one-hour diversion',
 'Implement the intended structural vocabulary or surface explicit unsupported/raw content without silently losing it. Reconcile the completed parity/default claim against real constructs and diagnostics. Do not infer native grammar completion from the fence/semantic-block sample tests.'),
finding(163,'The Djot round-trip serializer erases recognized link destinations and predicates',
 'crates/nematic/nematic/src/knot/djot.rs:blocks_to_djot, emit_block Paragraph/Heading',
 'A recognized Topic link to mere://node/topic with rel=cites parses into an outgoing link and one LinkStatement. blocks_to_djot writes only Topic newline using inline_text; reparsing keeps the label but yields no destination and no statement. The public serializer calls itself the parser dual and claims round-trip on the recognized subset, which now includes typed links and rel.',
 'Native parse/export/reparse and actual link_statements controls. No graph ingest, edge deletion or user save executed.',
 'inline serializer/schema adoption can exceed one-hour diversion when preserving the full vocabulary',
 'Serialize inline structure, destination, title and predicate with grammar-correct escaping, or explicitly qualify the projection as lossy. Keep one Djot emitter for recognized spans and exercise the actual supported parse-emission subset rather than only plain semantic sample blocks.'),
finding(164,'Per-block clip provenance is emitted but disappears instead of becoming the promised metadata',
 'crates/nematic/nematic/src/knot/expand/build.rs:build_clip_knot_with_block_provenance; knot.rs:apply_frontmatter',
 'The clip builder emits block_sources containing the second source URI and anchor. Both CommonMark and Djot Knot parsers drop that information entirely: no block_sources MetadataRow, no source URI in the resulting document, and no block_sources on re-export. Document-level source remains as a control. apply_frontmatter applies only its named keys. The builder explicitly promises a metadata row even though full provenance reconstruction is deferred.',
 'Native builder, sparse sidecar, both parsers and re-export with a differing-source override. No citation overlay, composed user clip or consumer provenance restoration executed.',
 'contained metadata-retention fix; full indexed provenance restoration remains a larger gated design',
 'Preserve the advertised metadata or state accurately that parsing discards it. Do not confuse the unimplemented minimum retention with the explicitly deferred full sidecar restoration. If implementing restoration later, account for note_kind/tags prefix blocks and source-index remapping, escaping and delimiter admission.'),
finding(165,'Extension and MIME routes select incompatible grammars for the same native Knot file',
 'crates/nematic/nematic/src/file.rs:FileEngine::pick; crates/inker/inker/src/routing.rs default native Knot MIME',
 'The same file:///note.knot and definition-list body route through FileEngine to CommonMark when MIME is unknown, yielding no MetadataRow. Adding text/x-knot selects Djot and produces Trust=tofu metadata. FileEngine keeps a KnotEngine compatibility delegate for .knot. The governing design says Djot is the author/default grammar and compatibility remains by pin; no extension exception is recorded there.',
 'Native default policy, full engine registry dispatch and actual document blocks. No file read, sniffing host pipeline or editor UI executed.',
 'contained native file default correction plus shared selection review',
 'Resolve native .knot through the declared grammar consistently while retaining explicit compatibility selection. Share format/grammar selection across file, MIME and session routes; Q154 already demonstrates another selector ignoring the selected engine. Do not remove the intentionally supported compatibility engine.'),
finding(166,'Nested tight Markdown lists put the nested child before the parent item text',
 'crates/nematic/nematic/src/markdown.rs:Converter::end_tag TagEnd::Item and attach_block',
 'For - outer followed by an indented - inner, the outer item contains [inner List, outer Paragraph]. Inserting a blank line creates a loose item and preserves [outer Paragraph, inner List]. Tight item text stays in its inline frame until End(Item), while nested blocks attach earlier; the final flush appends the parent after them.',
 'Native CommonMark parser/lowerer with tight and loose nested-item controls. No visual frame or screen reader executed; the portable source order itself is reversed.',
 'contained pending-inline flush at child-container boundaries',
 'Flush pending tight-item inline content at the correct source boundary before attaching nested blocks. Cover mixed text, nested lists, quotes and subsequent content so a local ordering repair does not drop or duplicate tight/loose item text.'),
finding(167,'Plain-text blank-line splitting fails on CRLF input',
 'crates/nematic/nematic/src/text.rs:paragraphs and paragraph_spans',
 'alpha LF LF beta yields two Paragraph blocks; the same physical text with CRLF yields one Paragraph with an empty line represented by soft breaks. paragraphs splits the literal byte sequence LF LF even though paragraph_spans uses lines() and otherwise understands CRLF. The documented blank-line-separated paragraph rule therefore changes with ordinary line endings.',
 'Native plain-text engine and paired LF/CRLF fixtures. No file load, renderer or text reflow screenshot executed.',
 'contained line-oriented paragraph delimiter correction',
 'Group paragraphs by physical empty lines under the intended line-ending policy and retain within-paragraph soft breaks. Keep verbatim protocol mode separate, where original blank lines and columns are meaningful.'),
finding(168,'The feature-disabled library suite asserts HTML body structure that the declared fallback cannot provide',
 'crates/nematic/nematic/src/feed/tests.rs:an_entry_address_renders_the_entry_as_its_own_document; feed/article.rs:cfg(not(html-fragment))',
 'Fresh cargo test -p nematic --lib --no-default-features runs235 tests:234 pass and this one fails at the assertion that the article body has its own heading. With HTML disabled, the implementation explicitly emits an unsupported-body diagnostic and falls back to Short teaser, exactly the state shown by the failure. The unconditional test still requires the default-feature HTML structure. Default237 library tests plus3 example tests pass.',
 'Fresh original feature-disabled library suite, observed shell exit1 and one named failure; no claim the fallback implementation is broken or all features pass. No production/test source edited.',
 'contained feature-qualified test expectations',
 'Assert the HTML structure only for the enabled feature and qualify the disabled summary/diagnostic path explicitly. Keep the optional feature gate reproducible instead of treating the default suite as evidence for both configurations.'),
]
findings[-1]['status']='original --no-default-features library suite fails with one feature-unqualified assertion; documented, not repaired'
findings[-1]['diagnostic']='nematic_no_default_suite.log'
upsert(notes['findings'],findings,'id')
for q in ['Q146','Q147']:
 f=next(f for f in notes['findings'] if f['id']==q)
 f['parser_followup']={'verification':'nematic_review_receipt.json','diagnostic':'nematic_probe_receipt.json',
  'evidence':('Actual KnotEngine parse/export/reparse turns escaped literal *x* into an Emphasis span. This upgrades the prior exact-output collision proof to an executed parser example; no user persistence or trust escalation.' if q=='Q146' else
   'A FeedEngine-parsed stable-id survives in its source FeedEntry.guid, then to_knot plus KnotEngine reparse produces a FeedEntry with guid=None. This upgrades the prior serializer collision proof; no live feed database was mutated.')}
def entry(n,domain,claim,implementations,assessment):
 return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
  'assessment':assessment,'state':'All75 Nematic inventoried files read; default237+3 tests pass,9 native observations, optional235-test gate has1 failure',
  'verification':'nematic_review_receipt.json'}
entries=[
entry(282,'Shared protocol parsing versus portable lowering','Nematic owns AST-to-block lowering; Errand/protocol crates own source grammar.',
 ['Gemtext and Gopher engines consume errand::parse ASTs', 'Nex consumes shared classification/base_url', 'Scroll consumes shared scrolltext lines/spans', 'Spartan reuses Gemtext Lowering and shared typed prompt grammar'],
 'Actual parser reuse is present, not an independent network engine in each file. Grouping lines into portable paragraphs/menus is a distinct projection responsibility. These wrappers do not establish transport/certificate/reassembly success.'),
entry(283,'File, MIME, session and protocol body selection','One engine ID should identify the parser producing the document.',
 ['Inker MIME/pin/scheme rules', 'FileEngine extension table', 'document-lanes scheme/body selector', 'Scroll/Spartan choose body delegates from response MIME'],
 'Q165 and earlier Q154/Q156 show actual divergence across owner boundaries. Protocol-specific MIME lowering can be appropriate; repeated application format selection without passing the resolved identity is the competing authority. Native .knot should not silently pick compatibility through one path.'),
entry(284,'Native Djot readiness and parity','Default migration completed with CommonMark parity.',
 ['Manifest calls Djot the author grammar', 'Design phase5 and default MIME registration declare migration landed', 'Source still labels parts PoC/experimental and says not registered', 'Event walker only lowers selected containers'],
 'Q162 proves quote/list/table parity is absent and one table vanishes silently. Successful protocol-fence/wikilink reuse is a real but narrower migration. The live default registration is authoritative over stale experimental prose, while completed parity claims exceed actual lowering.'),
entry(285,'Djot parse/emit shared vocabulary','blocks_to_djot is the dual of its recognized parser subset.',
 ['Parser gained Link and predicate capture', 'Paragraph/Heading emitter flattens inline_text', 'Feed div schema/producer capture only title/url/date and display summary', 'Golden round-trip covers a narrow plain semantic sample'],
 'Q163 proves parser evolution was not propagated to its serializer. Separate parser, emitter, schema and test sample create a circular completeness claim. The producer also emits table/list/image constructs that its parser cannot recreate; full schema coverage needs explicit supported/lossy distinctions.'),
entry(286,'Knot frontmatter authority and trust labels','CommonMark and Djot share note metadata semantics.',
 ['One split_frontmatter and apply_frontmatter implementation', 'Flat YAML-shaped parser ignores unknown keys', 'parse_trust maps authored strings onto DocumentTrustState', 'Clip builder independently writes overlapping fields'],
 'Shared apply is real reuse. Raw scalar/flow-list serialization duplicates Inker formatting and inherits Q146 ambiguity. An authored trust label is metadata, not independent TLS/signature proof; no current host authentication bypass was established. Unknown frontmatter is not preserved generically.'),
entry(287,'Per-block provenance persistence','Producer metadata survives parsing before full sidecar restoration lands.',
 ['BlockProvenanceMap stores source/anchor per top-level index', 'Clip builder emits sorted block_sources wire entries', 'Neither parser applies block_sources', 'Tests assert emitted substrings/order only'],
 'Q164 distinguishes actual producer output from false retention claims. Full restoration is explicitly gated, but even the promised metadata row is absent. Source indices can shift after metadata prefixes; a future consumer must remap rather than treat original integers as persistent source identity.'),
entry(288,'Passive HTML URL admission','Active schemes never reach the portable model.',
 ['html5ever/StaticDocument decodes entities', 'Local safe_url reimplements scheme syntax', 'No-recognized-scheme branch accepts the original string', 'Existing url crate canonicalizes those targets differently'],
 'Q160 reproduces a disagreement between the custom authority classifier and real URL parser. Plain javascript refusal tests are too narrow. No script execution is claimed; host navigation/export still owns its own action boundary.'),
entry(289,'Inline text and whitespace semantics','Passive HTML lowering preserves readable text while removing active/style constructs.',
 ['DOM converter has its own collapse_whitespace and trim_spans', 'Inline child containers trim their own spans', 'Whitespace-only text returns no content', 'Preformatted text has a distinct collector'],
 'Q161 proves word concatenation at an ordinary inline boundary. A semantic HTML subset and Fleece article extraction serve different selection purposes, so their existence alone is not duplication of durable authority. Shared low-level normalization rules need care around block/preformatted boundaries.'),
entry(290,'CommonMark source order and container stacking','A pure parser-to-portable model preserves nested content order.',
 ['Block stack stores lists/items/quotes', 'Tight item text has a separate inline frame', 'Nested blocks attach immediately to the item', 'End(Item) appends the pending parent paragraph'],
 'Q166 proves the two accumulation paths reverse parent/child order for tight nesting. Loose-list and simple-item tests miss the mixed container boundary. Unsupported raw HTML/image lowering is documented lossy v1 but emits no corresponding document diagnostic; do not treat CommonMark branding as complete semantic coverage.'),
entry(291,'Plain text and literal protocol representation','Text engine has paragraph and verbatim modes.',
 ['Paragraph mode splits LF LF then uses lines()', 'Verbatim mode trims terminal CR/LF and retains interior spacing', 'Finger and Nex content delegate verbatim', 'Other prose/file fallbacks use paragraph mode'],
 'Q167 is inconsistent physical line handling within paragraph mode. Verbatim ownership is intentional rather than another parser authority. Terminal newline trimming means verbatim is presentation-oriented, not byte-faithful archival encoding.'),
entry(292,'Micron compatibility subset versus qualified grammar','Unknown/uncaptured constructs stay explicit rather than silently live.',
 ['MicronSubsetEngine has a separate stable compatibility ID and narrow LF-only cases', 'Micron syntax retains full source plus typed fields/partials/presentation', 'MicronEngine projects supported facts and adds deduplicated loss diagnostics', 'CRLF lexemes are accepted only in the richer syntax lane'],
 'Two paths have named scopes and current registration, not identical implementations falsely called one. Rich projection keeps unsupported requests/remote anchors inert and visible. Syntax source retention does not imply every lowerer or exporter preserves those facts.'),
entry(293,'Micron navigation authority in line and block spaces','Syntax declarations feed one navigation model then a block-index remap.',
 ['Syntax reserves first anchor names', 'Navigation computes line extents/named headings', 'MicronEngine maps line offsets around inserted diagnostic badge', 'Inker FoldState stores source-key overrides and document-canvas measures them'],
 'This is a deliberate multi-space projection with broad original probe-page controls. Block count remains the weak currentness check Q145. The committed stock-client receipts establish bounded language observations, not a new headed/physical parity run in this audit.'),
entry(294,'Micron form preparation versus destination authority','Source equality and bounded values prepare an ephemeral request; host owns identity and send.',
 ['FormState admits source/field/value/map budgets', 'Typed group compatibility/radio exclusion', 'PreparedForm contains a target plus deterministic BTreeMap', 'Custom Debug redacts values and documentation denies destination authority'],
 'Explicit source checks are not sold as authorization. Unsupported controls/table/dynamic partial cases refuse preparation when actions exist. Pure tests do no I/O. Packet/envelope limits, address binding, confirmation and cancellation remain host-owned; no live form submission occurred.'),
entry(295,'Feed XML/JSON preservation and shared shape','Both feed parsers share the same document projection.',
 ['XML AST comes from errand::parse::feed', 'JSON Feed has a local serde model then populates shared Feed', 'Shared blocks retain guid/published/updated/enclosures/content_address', 'JSON model does not include attachment fields or a version discriminator'],
 'AST/block reuse is real, but accepted JSON fields are a separate admission policy and omit media metadata available in the shared shape. Missing/unknown field behavior is not proof of JSON Feed semantic validation. Q147 now has an actual reparse proof of downstream guid loss; no database identity mutation is claimed.'),
entry(296,'Feed entry bodies and language/base identity','An entry-address fragment selects its own passive article view.',
 ['Entry GUID percent encode/decode and first matching entry', 'HtmlFragmentEngine lowers body then recursive URL resolution uses article link', 'No-HTML build explicitly falls back to summary with diagnostic', 'Article document currently sets lang None'],
 'Native default body tests and Q168 optional failure are scoped separately. A relative article link is not joined to an absolute feed base before URL parse, and feed language is not carried into the article; these are source-qualified follow-ups, not tested current host regressions.'),
entry(297,'Protocol wrapper identities and integrity claims','Body-only adapters do not claim to validate complete protocols.',
 ['Guppy delegates reassembled text and makes no reassembly verdict', 'Titan delegates response body while upload stays transport-owned', 'Misfin keeps trust Unknown and explicitly names missing envelope validation', 'Finger/Nex text delegation re-tags provenance visibly'],
 'Thin adapters are useful shared lowering and telemetry identities, not siloed transports. Provenance source_label is also used as inner engine ID, while elsewhere it means human source label; this overload needs consumer interpretation, not another durable store. Spartan retains stale prose claiming Scroll has signatures, contrary to Scroll own current tests.'),
entry(298,'Scroll input controls and ordinal projection','Real scrolltext grammar is shared and lowered with explicit limitations.',
 ['Scroll parser owns grammar; native lowering keeps relation polarity as predicate', 'InputLink becomes ordinary Link with a degradation reason saying no input affordance exists', 'Spartan already uses shared InlineSpan::Submit', 'Scroll ordinals are retained as item text plus ordered List flag'],
 'The loss is disclosed, but its block-model explanation is stale. Protocol input request semantics must be checked before blindly mapping to a host POST path. Rendering/export list decorations can duplicate retained ordinal text; richer ordered-marker authority is a separate contract decision, documented rather than invented here.'),
entry(299,'Fresh tests versus claimed round-trip coverage','Default green gates support the actual tested configuration and subset.',
 ['237 current default library tests and3 example tests pass', 'Nine generated observations include real parser/export/reparse', 'No-default library gate runs235 and fails one default-only HTML assertion', 'Earlier Q146/Q147 now have actual parser follow-ups'],
 'Do not call optional configuration green or the recognized grammar complete from default fixtures. All75 inventoried source/manifest/example/Micron fixture files were read. No browser, GPU, network, OS a11y, live protocol server or user persistence was executed; old fixture receipts remain historical observations.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
 {'title':'Djot nested div and attribute attachment use a single context rather than a nesting stack',
  'source':'crates/nematic/nematic/src/knot/djot.rs:parse_djot_knot_body_validated',
  'note':'A new Div overwrites the prior DivCtx and clears the inline accumulator; heading/code/description contexts share that accumulator. Pending attrs can remain until a later div/link. Q162 establishes broad missing event handling, but nested-div source loss/attribute reassignment has not been separately probed. A full source-preserving container repair can exceed one-hour diversion.'},
 {'title':'Scroll input-link degradation cites an obsolete absence of input affordances',
  'source':'crates/nematic/nematic/src/scroll.rs:Lowering::push InputLink; spartan.rs typed prompt branch',
  'note':'Scroll keeps InputLink as an ordinary navigable Link and emits a diagnostic saying the block model has no input affordance; shared Submit exists and Spartan uses it. Determine actual Scroll request/input semantics before adopting the session POST/mutation adapter. This is not a claim a transport mutation was executed; document the owner boundary rather than choose a protocol policy unilaterally.'},
 {'title':'Feed article relative link/base and language propagation need controls',
  'source':'crates/nematic/nematic/src/feed/article.rs:render and body_blocks',
  'note':'Article link may be relative in parsed shape; Url::parse(base) then skips recursive resolution, and canonical_uri keeps that relative string. Article lang is None even when parent feed lang exists. No actual relative-feed or language projection probe executed. Need parser/base resolution owner review without reintroducing host fetch authority into body lowering.'},
 {'title':'Knot export and inline-extension traversal have different presentation/source scopes',
  'source':'crates/nematic/nematic/src/knot/expand.rs; crates/inker/inker/src/document/render.rs',
  'note':'Fence expansion and inline rewrites recurse quotes/lists but not Presented wrappers or tables. Current parser paths do not emit every model variant. Exported source text can gain hashtags/wikilinks or semantic-fence meaning on reparse; Q146 proves one actual literal-loss example. Full stored-source versus derived semantic preservation may exceed one-hour diversion.'},
],'title')
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/system/errand/') and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Complete Nematic75/75 source review including examples and all inventoried Micron fixtures'
notes['next_area']='crates/system/errand'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete','coverage':stats,
 'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':['nematic_review_receipt.json'],
 'next_read':next_read,'production_edits_this_turn':False,'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
 'diagnostic_outcomes':'Default Cargo build and237 library+3 example tests pass;9 observations pass. No-default library suite234 pass/1 fail (HTML assertion not feature-qualified).',
 'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
 'coverage_note':'75 complete files452682 chars added. Named governing design migration excerpt read separately; historical wire JSON/receipt data is not new execution.'}
upsert(notes['source_qualifications'],[{'area':'Complete Nematic source review',
 'note':'All75 inventoried source/manifest/example/Micron fixture files452682 chars read. Default build and237 library+3 example tests pass. Nine generated observations pass, including real parser/export/reparse, route/registry composition and URL canonicalization. No-default library gate234 pass/1 fail at unguarded HTML-body expectation; disabled implementation explicitly falls back. Q146/Q147 parser proofs added. No browser, script execution, GPU, network, physical AT, user save or live protocol endpoint. Exact hashed rlibs retained for evidence despite root Cargo aliases changing under feature gates. Probe executable/PDB absent; stable target reused, no isolation.'}],'area')
save('review_notes.json',notes);save('authority_map.json',authority)
doc=old['document']['path'];destination=ROOT/doc;before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
heading='### Read-only Nematic complete source review, 2026-10-16'
assert heading.encode() not in before
addition='\n\n'+heading+'\n\nAll75 inventoried Nematic files are read,452,682 more characters, including the manifest, example and checked-in Micron source fixtures. Repository coverage is1112/3352 files and15,047,472/36,854,820 characters. Full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at '+next_read['path']+'. Authority entries A282-A299 distinguish actual shared parser reuse from competing selectors and overclaimed parity/round-trip.\n\n'
for f in findings:
 addition+=f['id']+': '+f['title']+'. '+f['evidence']+'\n\n'+f['scope']+' Repair scope: '+f['repair_triage']['size']+'. '+f['repair_triage']['reason']+'\n\n'
addition+='Q146/Q147 parser follow-up: actual KnotEngine parse/export/reparse converts an escaped literal *x* to an Emphasis span. A FeedEngine-parsed stable-id becomes guid=None after export and KnotEngine reparse. These strengthen the prior non-injectivity proofs into actual parser examples. No user save, graph edge/database mutation or trust escalation was executed.\n\n'
addition+='The native-default migration is materially narrower than its completed-parity claim. Current source registers Djot and Inker MIME routing selects it; stale source comments still call it experimental/not registered. The governing polyglot design phase5 says parity and native-default landed, with CommonMark available by pin. Reusing fence expansion and inline rewrites is real, but does not implement every structural container or keep new link predicates in the serializer. The native FileEngine extension default still selects CommonMark. Q154/Q156 in the session lane already demonstrate independent selection and dropped MIME/final identity; a shared resolved parser authority needs a broader owner repair, documented rather than used to prolong the audit.\n\n'
addition+='Gemtext/Gopher/Nex/Scroll/Spartan grammar reuse is present through Errand/protocol ASTs. Nematic owns portable grouping/lowering. Body wrappers for Guppy/Titan/Misfin/Finger are explicit about transport/reassembly/upload/envelope scope and keep trust Unknown; they are not parallel network implementations. The source_label field is overloaded with inner engine IDs as well as human source labels. Scroll still turns input links into ordinary links with a diagnostic claiming no input affordance, although shared Submit exists; its actual request semantics need review before mapping to a POST adapter. Spartan stale prose about Scroll signatures contradicts Scroll current explicit tests. These are recorded authority/documentation mismatches, not live protocol success claims.\n\n'
addition+='Micron has a separately named compatibility subset and a richer source-preserving syntax lane. Syntax anchor admission, line-space navigation, block-index remapping around diagnostic badges, source-key fold state and measured layout are separate spaces with real reuse. Original probe-page controls validate bounded captured behavior. Form preparation checks exact source and configured budgets, rejects ambiguous/unsupported controls, redacts values in Debug and explicitly leaves destination identity/confirmation/cancellation/envelope limits with the host. No form I/O occurred. Source-preserving syntax does not make every portable export byte-faithful.\n\n'
addition+='Feed XML/JSON share the Feed/block projection but JSON admits a smaller local serde field model, omitting media attachment metadata and a version discriminator. Entry rendering does not propagate feed language, and a relative article base fails absolute Url::parse before recursive resolution. Nested Djot div state and pending attribute attachment, source-index remapping for restored block_sources, and literal/extension preservation remain larger source-qualified follow-ups. Unknown-attribute and full block-provenance reconstruction were already explicitly deferred; Q164 is the narrower false claim that even a MetadataRow survived.\n\n'
addition+='Fresh default Cargo library build succeeds;237 library tests plus3 example tests pass. Nine native observations pass, including actual reparses. A separate no-default-features library gate runs235 tests with234 passing and1 failing at an unconditional HTML-body assertion, while the disabled implementation explicitly falls back as designed. This optional configuration is not reported green. No browser/script execution, GPU, live network/server, OS accessibility, physical assistive technology or user persistence ran. Historical Micron JSON/wire receipts were fixture inputs, not new headed receipts. Probe executable/PDB removed; exact hashed library artifacts avoid mutable Cargo root-alias ambiguity across feature configurations. Stable shared target retained; no isolated target, Cargo home or worktree. Evidence and source hashes are in the [Nematic review receipt](../../../support/receipts/authority-success-audit/nematic_review_receipt.json).\n'
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'nematic_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['nematic_probe.rs','run_nematic_probe.py','nematic_probe_receipt.json','nematic_probe_build.log','nematic_probe_output.log',
 'nematic_library_build.jsonl','nematic_library_build_stderr.log','nematic_default_suite.log','nematic_no_default_suite.log','nematic_diff_check.txt']
design='design_docs/nematic_docs/implementation_strategy/2026-05-08_polyglot_knot_design.md'
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind':'Complete Nematic source reading; full Mere objective incomplete','production_edits_this_pass':False,
 'new_findings':[f['id'] for f in findings],'updated_findings':['Q146','Q147'],'authority_entries':[e['id'] for e in entries],
 'coverage':stats,'added_complete_files':complete,'source_sha256':{p:sha(ROOT/p) for p in complete},
 'preserved_production_and_test_hashes':preserved,'next_read':next_read,
 'default_library_tests':237,'default_example_tests':3,'default_test_exit':0,
 'no_default_library_tests':235,'no_default_passed':234,'no_default_failed':1,'no_default_shell_exit':1,
 'no_default_failure':'feed::tests::an_entry_address_renders_the_entry_as_its_own_document: HTML structure assertion despite disabled html-fragment',
 'diagnostic_tests':9,'diagnostic_exit':0,'generated_output_removed':True,'retained_isolation':[],
 'governing_design_trace':{'path':design,'sha256':sha(ROOT/design),'scope':'native/default migration and recognized serializer phase excerpts, not complete prose-document read'},
 'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
 'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
 'limits':['Full Mere objective incomplete; no production/original test changes.',
  'Default237 library+3 example pass; no-default234/235 pass with one feature-unqualified assertion; no claim all feature gates pass.',
  'Nine current native CPU observations; actual parser/export/reparse and route/registry composition included.',
  'HTML active scheme admission/canonicalization only; no script/browser execution or host navigation bypass proved.',
  'No GPU, network/server, OS/physical accessibility, user persistence, graph edge or feed database mutation.',
  'Micron captures remain historical fixtures; current tests do not re-run the independent stock client.',
  'Full provenance restoration/unknown attribute preservation are explicit prior deferrals; minimum claimed retention is separately disproved.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
save('nematic_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'findings':len(notes['findings']),'authority_entries':len(authority['entries']),'next_read':next_read}))
