"""Repair only the fixture's generic RefCell reborrow; preserve failed v1 evidence."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
original=(HERE/'native_authority_probe.rs').read_text()
fixed=original.replace('OwnedLayout::new(&dom,&[""],960.', 'OwnedLayout::new(&*dom,&[""],960.').replace('layout.painted_rect(&dom,bare)', 'layout.painted_rect(&*dom,bare)')
assert original != fixed
with (HERE/'native_authority_v2_probe.rs').open('x',encoding='utf-8') as f:f.write(fixed)
runner=(HERE/'run_native_authority_probe.py').read_text().replace('native_authority','native_authority_v2').replace('native-authority','native-authority-v2')
with (HERE/'run_native_authority_v2_probe.py').open('x',encoding='utf-8') as f:f.write(runner)
print('Prepared distinct v2; original extracted source functions unchanged')
