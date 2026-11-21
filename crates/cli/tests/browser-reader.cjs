// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Exercise the shipped reader's message boundaries without a browser or a network.
const {test} = require("node:test");
const assert = require("node:assert/strict");
const {readFileSync} = require("node:fs");
const {join} = require("node:path");
const vm = require("node:vm");

const source = readFileSync(join(__dirname, "../src/webrtc/reader.js"), "utf8");
function reader() {
  const invitation = {address: "127.0.0.1:12345", query: "ASK {}"};
  const fields = {invitation: {textContent: Buffer.from(JSON.stringify(invitation)).toString("base64")}};
  const context = vm.createContext({
    TextEncoder, TextDecoder, atob, Uint8Array, DataView, window: {},
    document: {getElementById(id) {return fields[id] ??= {};}}
  });
  vm.runInContext(source, context);
  return context;
}

test("a split result with unbound and escaped values is acknowledged once", async () => {
  const {framed, receiveRows} = reader();
  const sent = [];
  const channel = {send: packet => sent.push(packet)};
  const result = receiveRows(channel, new AbortController().signal);
  const rows = {columns:["x","y"], rows:[['<script>\n' + "a".repeat(80000), null]]};
  const bytes = framed(new TextEncoder().encode(JSON.stringify(rows)));
  // Split the inner four-byte length itself across the first two carrier frames.
  for (const chunk of [bytes.subarray(0,2), bytes.subarray(2,40000), bytes.subarray(40000)]) {
    channel.onmessage({data: framed(chunk).buffer});
  }
  assert.equal(JSON.stringify(await result), JSON.stringify(rows));
  assert.deepEqual(sent, [framed(Uint8Array.of(1))]);
});

test("an oversized declared reply is rejected from the length prefix", async () => {
  const {framed, receiveRows} = reader();
  const channel = {send: () => assert.fail("invalid reply was acknowledged")};
  const result = receiveRows(channel, new AbortController().signal);
  channel.onmessage({data: framed(Uint8Array.of(0,32,0,0)).buffer});
  await assert.rejects(result,/Invalid result size/);
});
