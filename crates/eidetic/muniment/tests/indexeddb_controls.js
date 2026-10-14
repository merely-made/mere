// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Disposable-only platform fixtures. Raw opens model inspected old bundles;
// they deliberately do not participate in the revised Web Lock protocol.
export function legacy(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onupgradeneeded = () => {
            request.result.createObjectStore("records").put(new Uint8Array([3, 1, 4]), "sentinel");
            const unknown = request.result.createObjectStore("unknown", { keyPath: "id" });
            unknown.createIndex("label", "label", { unique: true });
            unknown.put({ id: "extra", label: "untouched", bytes: new Uint8Array([9, 2, 6]) });
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}
export function closeLegacy(db) { db.close(); }
export function task() { return new Promise(resolve => setTimeout(resolve, 0)); }
export async function inspect(name, version) {
    const db = await new Promise((resolve, reject) => {
        const request = indexedDB.open(name, version);
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
        request.onupgradeneeded = () => request.transaction.abort();
    });
    try {
        return await new Promise((resolve, reject) => {
            const names = [...db.objectStoreNames].sort();
            const tx = db.transaction(names, "readonly");
            const result = {};
            for (const name of names) {
                const store = tx.objectStore(name);
                const indexes = [...store.indexNames].map(key => {
                    const index = store.index(key);
                    return [key, index.keyPath, index.unique, index.multiEntry];
                });
                const keys = store.getAllKeys();
                const values = store.getAll();
                values.onsuccess = () => { result[name] = { keyPath: store.keyPath, autoIncrement: store.autoIncrement, indexes, keys: keys.result, values: values.result }; };
            }
            tx.oncomplete = () => resolve(JSON.stringify(result));
            tx.onabort = () => reject(tx.error);
        });
    } finally { db.close(); }
}
export function oldVersionError(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onerror = () => resolve(request.error.name);
        request.onsuccess = () => { request.result.close(); reject(new Error("old v1 open unexpectedly succeeded")); };
    });
}

export function uniqueLegacy(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onupgradeneeded = () => {
            const store = request.result.createObjectStore("records");
            store.createIndex("bytes", "", { unique: true });
            store.put(new Uint8Array([7]), "sentinel");
        };
        request.onsuccess = () => { request.result.close(); resolve(); };
        request.onerror = () => reject(request.error);
    });
}
export function changeVersion(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 3);
        request.onsuccess = () => { request.result.close(); resolve(); };
        request.onerror = () => reject(request.error);
    });
}
export async function competingFrame(name) {
    const frame = document.createElement("iframe");
    document.body.append(frame);
    try {
        return await frame.contentWindow.navigator.locks.request(
            `muniment:indexeddb:${name}`, { mode: "exclusive", ifAvailable: true },
            lock => lock === null,
        );
    } finally { frame.remove(); }
}

// One disposable open only. The observer runs at the actual platform upgrade
// event, before the controller's handler, and restores the platform entrypoint.
export function cancelAtUpgrade(callback) {
    const original = indexedDB.open;
    indexedDB.open = function (...args) {
        indexedDB.open = original;
        const request = original.apply(this, args);
        request.addEventListener("upgradeneeded", () => callback(), { once: true });
        return request;
    };
}
export function watchTransactions() {
    const original = IDBDatabase.prototype.transaction;
    const observation = { events: [], restore: () => { IDBDatabase.prototype.transaction = original; } };
    IDBDatabase.prototype.transaction = function (...args) {
        const transaction = original.apply(this, args);
        if (args[1] === "readwrite") {
            transaction.addEventListener("error", () => observation.events.push("error"));
            transaction.addEventListener("abort", () => observation.events.push("abort"));
            transaction.addEventListener("complete", () => observation.events.push("complete"));
        }
        return transaction;
    };
    return observation;
}
export function finishWatching(observation) {
    observation.restore();
    return observation.events.join(",");
}
