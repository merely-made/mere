// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// The one-run test for run_static_constructors_once: load the ctor_once
// fixture's --target web glue in Node and require that the module's
// constructors ran exactly once, after init, after ordinary export calls and
// after a second call of the helper.
//
//   node run.mjs <pkg-dir> [--glue-also-runs-ctors]
//
// --glue-also-runs-ctors stands in for page glue that runs the constructors
// itself (the fixture must then be linked with
// -C link-arg=--export=__wasm_call_ctors); it must make this test fail.
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const [dir, ...flags] = process.argv.slice(2);
const glue = await import(pathToFileURL(`${dir}/ctor_once.js`).href);
const raw = glue.initSync({ module: readFileSync(`${dir}/ctor_once_bg.wasm`) });
if (flags.includes("--glue-also-runs-ctors")) {
  if (typeof raw.__wasm_call_ctors !== "function") {
    console.error("no __wasm_call_ctors export: link the fixture with --export=__wasm_call_ctors");
    process.exit(2);
  }
  raw.__wasm_call_ctors();
}
const seen = { after_init: glue.ctor_runs() };
for (let i = 0; i < 3; i++) glue.echo(`call ${i}`);
seen.after_three_calls = glue.ctor_runs();
glue.run_helper_again();
seen.after_second_helper_call = glue.ctor_runs();
const wrapped = Object.keys(raw).filter((name) => name.endsWith(".command_export"));
const passed = Object.values(seen).every((runs) => runs === 1) && wrapped.length === 0;
console.log(JSON.stringify({ pkg: dir, flags, seen, command_export_wrappers: wrapped.length, passed }));
process.exit(passed ? 0 : 1);
