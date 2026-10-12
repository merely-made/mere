# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

from pathlib import Path
import dataclasses,collections,subprocess,sys,json
root=Path.cwd().resolve();sys.path.insert(0,str(root/'scripts'))
import mere_doc_audit as audit
paths=['design_docs/DOC_README.md','design_docs/mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md','design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md','design_docs/mere_docs/research/2026-10-09_scenograph_codex_handoff.md','design_docs/mere_docs/testing/receipts/2026-10-12_coupling_effects/README.md']
baselines={str((root/p).resolve()):subprocess.check_output(['git','show','e0dae9c9:'+p]).decode('utf-8') for p in paths}
original=Path.read_text
def baseline_read(self,*args,**kwargs):
 return baselines.get(str(self.resolve())) if str(self.resolve()) in baselines else original(self,*args,**kwargs)
try:
 Path.read_text=baseline_read
 before=audit.audit(root)
finally:
 Path.read_text=original
after=audit.audit(root)
def findings(report):
 result=collections.Counter()
 for field in dataclasses.fields(report):
  values=getattr(report,field.name)
  if isinstance(values,list):
   for value in values:
    result[(field.name,value.source.split(':')[0],value.subject,value.detail)]+=1
 return result
added=findings(after)-findings(before);removed=findings(before)-findings(after)
result={'baseline':'e0dae9c9','amended_documents':paths,'before_counts':before.counts(),'after_counts':after.counts(),'introduced_findings':[list(k)+[v] for k,v in added.items()],'resolved_findings':[list(k)+[v] for k,v in removed.items()]}
out=Path('C:/Users/mark_/Code/testing/mere/coupling-trace/docs-comparison.json');out.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,indent=2));assert not added,'new documentation findings'
