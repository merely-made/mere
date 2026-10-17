"""Complete the fixture leaf trait; preserve all failed v1 artifacts."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
text=(HERE/'accessibility_authority_probe.rs').read_text()
assert text.count('fn paint(&mut self,_:&mut sprigging::PaintCx')==1
text=text.replace("fn paint(&mut self,_:&mut sprigging::PaintCx<'_>){}", "fn paint(&mut self,_:&mut sprigging::PaintCx<'_>){}\n        fn paint_dirty(&self)->bool{false}")
with (HERE/'accessibility_authority_v2_probe.rs').open('x',encoding='utf-8') as f:f.write(text)
runner=(HERE/'run_accessibility_authority_probe.py').read_text().replace('accessibility_authority','accessibility_authority_v2').replace('accessibility-authority','accessibility-authority-v2')
with (HERE/'run_accessibility_authority_v2_probe.py').open('x',encoding='utf-8') as f:f.write(runner)
print('Prepared v2 fixture trait completion; original source function extracts unchanged')
