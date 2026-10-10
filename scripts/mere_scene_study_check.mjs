// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
// Run with Node and the compiled scenograph presentation_rules example path.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { runInNewContext } from 'node:vm';
import { spawnSync } from 'node:child_process';
import { strict as assert } from 'node:assert';
import { createHash } from 'node:crypto';

const root=resolve(fileURLToPath(new URL('..',import.meta.url)));
const html=readFileSync(resolve(root,'support/design-studies/scene-rules.html'),'utf8');
const fixture=JSON.parse(readFileSync(resolve(root,'support/design-studies/scene-rules.json'),'utf8'));
const embedded=JSON.parse(html.match(/<script type="application\/json" id="mere-fixture">([\s\S]*?)<\/script>/)[1]);
assert.deepEqual(embedded,fixture,'embedded and canonical fixtures must agree');
const code=html.match(/<script data-mere-rule-model>([\s\S]*?)<\/script>/)[1];
const sandbox={structuredClone};runInNewContext(code,sandbox);const model=sandbox.MerePresentationRules;
const binary=process.argv[2];assert.ok(binary,'provide the compiled presentation_rules example path');
const plain=value=>JSON.parse(JSON.stringify(value));
function rust(request) {
  const result=spawnSync(binary,[],{input:JSON.stringify(request),encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);return JSON.parse(result.stdout);
}
let compared=0;
function compare(set,contexts) {
  const browser=contexts.map(context=>plain(model.resolve(set,context)));
  assert.deepEqual(browser,rust({rule_set:set,contexts}));compared+=contexts.length;
}
compare(fixture.rule_set,fixture.contexts);
const contexts=[];
for(const size of [0,25,48,49,70,83,84,100]) {
  for(const previous of [[],['compact-face']]) {
    for(const pinned of [false,true]) contexts.push({id:'access',kind:'node',facts:{media:'image'},states:['overview',...(pinned?['position_pinned']:[])],screen_size:size,previous_matches:previous});
  }
}
contexts.push({id:'missing',kind:'node',facts:{},states:[],screen_size:null});
compare(fixture.rule_set,contexts);
const competing=structuredClone(fixture.rule_set);
competing.rules.push({id:'explicit-access',label:'An explicit access override',enabled:true,priority:30,target:{kind:'node',ids:['access']},condition:{kind:'always'},effects:[{kind:'representation',value:'Card'}]});
compare(competing,contexts);
const disabled=structuredClone(fixture.rule_set);disabled.rules[2].enabled=false;compare(disabled,contexts);
const numeric={version:1,rules:[{id:'number',label:'Numeric reading',enabled:true,priority:1,target:{kind:'node',ids:[]},condition:{kind:'number',fact:'count',op:'gte',value:0},effects:[{kind:'details',visible:true}]}]};
compare(numeric,[{}, {count:0}, {count:-1}, {count:3}, {count:'0'}, {count:false}].map(facts=>({id:'number',kind:'node',facts,states:[],screen_size:null})));
const malformed=[];
const duplicate=structuredClone(fixture.rule_set);duplicate.rules.push(duplicate.rules[0]);malformed.push(duplicate);
const wrongTarget=structuredClone(fixture.rule_set);wrongTarget.rules[0].target.kind='field';malformed.push(wrongTarget);
const badThreshold=structuredClone(fixture.rule_set);badThreshold.rules[2].condition.conditions[0].exit_above=48;malformed.push(badThreshold);
const empty=structuredClone(fixture.rule_set);empty.rules[0].condition.conditions=[];malformed.push(empty);
for(const set of malformed) {
  assert.throws(()=>model.validate(set));
  const result=spawnSync(binary,[],{input:JSON.stringify({rule_set:set,contexts:[fixture.contexts[0]]}),encoding:'utf8'});
  assert.notEqual(result.status,0,'Rust must refuse the same malformed set');
}
console.log(JSON.stringify({valid_contexts_compared:compared,invalid_sets_refused:malformed.length,fixture_sha256:createHash('sha256').update(JSON.stringify(fixture)).digest('hex'),browser_model_sha256:createHash('sha256').update(code).digest('hex')},null,2));
