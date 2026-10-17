"""Source-only reconciliation, not a Rust/runtime acceptance test."""
from pathlib import Path
import hashlib,json,re
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
source='crates/system/ux-events/src/ux_diagnostics.rs'
text=(ROOT/source).read_text(); names=['surface_open_channel','surface_dismiss_channel','all_channel_ids']
def body(name):
    start=text.index('fn '+name+'('); opening=text.index('{',start); depth=1; end=opening+1
    while depth:
        if text[end]=='{':depth+=1
        elif text[end]=='}':depth-=1
        end+=1
    return text[opening:end]
mapped=set(re.findall(r'"(ux\.[a-z_]+\.(?:opened|dismissed))"',body(names[0])+body(names[1])))
listed=set(re.findall(r'"(ux\.[a-z_]+\.(?:opened|dismissed))"',body(names[2])))
missing=sorted(mapped-listed)
assert len(mapped)==40 and len(listed)==36
assert missing==['ux.inspector_pane.dismissed','ux.inspector_pane.opened','ux.steward_pane.dismissed','ux.steward_pane.opened']
result={'qualification':'Source literals reconciled in exact current function bodies. No Rust bridge, registry startup, host, browser or external consumer execution.',
 'source':source,'source_sha256':hashlib.sha256((ROOT/source).read_bytes()).hexdigest(),
 'mapped_surface_channels':len(mapped),'listed_surface_channels':len(listed),'missing':missing,
 'fixed_count_test':'all_channel_ids_covers_every_surface_and_dispatch_kind asserts38total; actual mapper has40surface IDs plus2dispatch IDs',
 'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
with (HERE/'registry_ux_catalog_check_receipt.json').open('x',encoding='utf-8') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result,indent=2))
