// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
"use strict";
const el = id => document.getElementById(id);
const utf8 = new TextDecoder("utf-8", {fatal: true});
const encoder = new TextEncoder();
const invite = JSON.parse(utf8.decode(Uint8Array.from(atob(el("invitation").textContent), c => c.charCodeAt(0))));
const maxReply = 1024 * 1024;

function framed(bytes) {
  const packet = new Uint8Array(4 + bytes.length);
  new DataView(packet.buffer).setUint32(0, bytes.length);
  packet.set(bytes, 4);
  return packet;
}

// Murm frames are chunks of one application byte stream. Query and reply
// boundaries are the application's own four-byte big-endian length prefix.
function sendBytes(channel, bytes) {
  for (let offset = 0; offset < bytes.length; offset += 8192) {
    channel.send(framed(bytes.subarray(offset, offset + 8192)));
  }
}

function gather(pc, signal) {
  return new Promise((resolve, reject) => {
    function check() { if (pc.iceGatheringState === "complete") resolve(); }
    signal.addEventListener("abort", () => reject(signal.reason), {once: true});
    pc.addEventListener("icegatheringstatechange", check, {signal});
    check();
  });
}

async function signalPeer(pc, transport, signal) {
  await pc.setLocalDescription(await pc.createOffer());
  await gather(pc, signal);
  await transport.ready;
  const bi = await transport.createBidirectionalStream();
  const writer = bi.writable.getWriter();
  await writer.write(Uint8Array.from(invite.token));
  await writer.write(framed(encoder.encode(pc.localDescription.sdp)));
  const reader = bi.readable.getReader();
  const buffer = new Uint8Array(32768 + 4);
  let filled = 0;
  for (;;) {
    const {value, done} = await reader.read();
    if (done) break;
    if (filled + value.length > buffer.length) throw new Error("Oversize signaling answer");
    buffer.set(value, filled);
    filled += value.length;
  }
  if (filled < 5 || new DataView(buffer.buffer).getUint32(0) !== filled - 4)
    throw new Error("Incomplete signaling answer");
  await pc.setRemoteDescription({type:"answer", sdp:utf8.decode(buffer.subarray(4, filled))});
  await writer.write(Uint8Array.of(1));
  await writer.close();
}

function receiveRows(channel, signal) {
  const buffer = new Uint8Array(maxReply + 4);
  let filled = 0;
  return new Promise((resolve, reject) => {
    signal.addEventListener("abort", () => reject(signal.reason), {once:true});
    channel.onclose = () => reject(new Error("Peer closed before sending the result"));
    channel.onerror = () => reject(new Error("WebRTC data channel failed"));
    channel.onopen = () => {
      el("status").textContent = "Querying the owner over WebRTC…";
      sendBytes(channel, framed(encoder.encode(invite.query)));
    };
    channel.onmessage = event => {
      try {
        const packet = new Uint8Array(event.data);
        if (packet.length < 4 || packet.length > 65536 ||
            new DataView(packet.buffer).getUint32(0) !== packet.length - 4) throw new Error("Invalid carrier frame");
        const bytes = packet.subarray(4);
        if (filled + bytes.length > buffer.length) throw new Error("Oversize result");
        buffer.set(bytes, filled);
        filled += bytes.length;
        if (filled < 4) return;
        const length = new DataView(buffer.buffer).getUint32(0);
        if (length > maxReply || length < 1 || filled > length + 4) throw new Error("Invalid result size");
        if (filled === length + 4) {
          const reply = JSON.parse(utf8.decode(buffer.subarray(4, filled)));
          sendBytes(channel, Uint8Array.of(1));
          resolve(reply);
        }
      } catch (error) { reject(error); }
    };
  });
}

function render(reply) {
  if (reply.error) throw new Error(reply.error);
  const {columns, rows} = reply;
  if (!Array.isArray(columns) || !Array.isArray(rows) || !columns.every(c => typeof c === "string") ||
      !rows.every(r => Array.isArray(r) && r.length === columns.length && r.every(v => v === null || typeof v === "string")))
    throw new Error("Invalid result table");
  const table = el("results");
  table.replaceChildren();
  const header = table.createTHead().insertRow();
  for (const name of columns) {
    const cell = document.createElement("th");
    cell.scope = "col";
    cell.textContent = name;
    header.append(cell);
  }
  const body = table.createTBody();
  for (const values of rows) {
    const row = body.insertRow();
    for (const value of values) row.insertCell().textContent = value ?? "—";
  }
  table.hidden = false;
  el("status").textContent = rows.length + (rows.length === 1 ? " row" : " rows") + " from the graph owner";
}

async function run() {
  el("run").disabled = true;
  el("results").hidden = true;
  el("route").textContent = "";
  el("status").textContent = "Connecting to the owner…";
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(new Error("Connection timed out. Check the invitation and UDP reachability.")), 45000);
  let pc, transport;
  try {
    pc = new RTCPeerConnection({iceServers: []});
    const channel = pc.createDataChannel(invite.label, {ordered:true});
    channel.binaryType = "arraybuffer";
    transport = new WebTransport("https://" + invite.address + "/offer", {
      serverCertificateHashes:[{algorithm:"sha-256", value:Uint8Array.from(invite.fingerprint)}]
    });
    transport.closed.catch(() => {});
    const [reply] = await Promise.all([receiveRows(channel, controller.signal), signalPeer(pc, transport, controller.signal)]);
    render(reply);
    el("route").textContent = "Received over a WebRTC data channel";
    // Give the owner time to receive the application acknowledgement.
    await new Promise(resolve => {
      if (channel.readyState === "closed") return resolve();
      channel.addEventListener("close", resolve, {once:true});
      setTimeout(resolve, 2000);
    });
  } catch (error) {
    el("status").textContent = "Query failed. " + error.message;
  } finally {
    clearTimeout(timer);
    controller.abort();
    transport?.close();
    pc?.close();
    el("run").disabled = false;
  }
}

el("peer").textContent = invite.address;
el("query").textContent = invite.query;
if (window.isSecureContext && window.WebTransport && window.RTCPeerConnection) {
  el("run").disabled = false;
  el("run").onclick = run;
} else {
  el("status").textContent = "Open the saved HTML file in a browser with WebRTC and WebTransport certificate pinning, such as Chrome.";
}
