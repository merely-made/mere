// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// This module owns native event lifetimes. A Rust waiter is never the owner of
// an IndexedDB callback. In particular, blocked opens cannot be removed from
// the browser queue: cancellation retains the lock until that request drains.
const failure = (kind, error, upgraded = false) => ({
    kind, message: error?.message ?? String(error),
    name: error?.name ?? "", schemaUpgraded: upgraded,
});

export function startOpen(name, store) {
    const state = { status: "acquiring", canceled: false, cause: null,
        request: null, upgrade: null, owner: null, schemaUpgraded: false };
    let deliver, refuse, unlock;
    state.result = new Promise((resolve, reject) => { deliver = resolve; refuse = reject; });
    // A caller can drop before attaching its Rust JsFuture rejection handler.
    state.result.catch(() => {});
    state.cancel = () => {
        state.canceled = true;
        state.cause ??= failure("canceled", "Opening canceled");
        if (state.owner) {
            state.status = "canceling-draining";
            closeOwner(state.owner);
        }
        if (state.upgrade) {
            try { state.upgrade.abort(); } catch (_) { /* terminal event decides */ }
        }
        if (state.request) state.status = "canceling-draining";
    };
    if (!globalThis.navigator?.locks || !globalThis.indexedDB) {
        state.status = "refused";
        refuse(failure("unavailable", "Web Locks and IndexedDB are required"));
        return state;
    }
    // Database identity, not store identity: all writable opens use this key.
    const held = new Promise(resolve => { unlock = resolve; });
    let delivered = false;
    const terminalRefusal = error => {
        state.cause ??= error;
        state.status = "refused-draining";
        unlock();
    };
    const cleanRequest = request => {
        request.onblocked = request.onupgradeneeded = request.onsuccess = request.onerror = null;
        state.request = null;
        state.upgrade = null;
    };
    const lockJob = navigator.locks.request(
        `muniment:indexeddb:${name}`, { mode: "exclusive", ifAvailable: true },
        lock => {
            if (!lock) {
                terminalRefusal(failure("busy", "Another page owns this database"));
                return;
            }
            if (state.canceled) { terminalRefusal(state.cause); return; }
            let request;
            try { request = indexedDB.open(name, 2); }
            catch (error) { terminalRefusal(failure("open", error)); return; }
            state.request = request;
            state.status = "opening";
            request.onblocked = () => {
                state.cause ??= failure("blocked", "Another page has this database open. Close it; this attempt will finish without opening a writer.");
                state.canceled = true;
                state.status = "blocked-draining";
                globalThis.dispatchEvent?.(new Event("muniment-open-blocked"));
            };
            request.onupgradeneeded = event => {
                const tx = request.transaction;
                state.upgrade = tx;
                tx.addEventListener("complete", () => {
                    state.schemaUpgraded = true;
                    state.upgrade = null;
                }, { once: true });
                tx.addEventListener("abort", () => { state.upgrade = null; }, { once: true });
                try {
                    if (state.canceled) { tx.abort(); return; }
                    if (event.oldVersion === 0) request.result.createObjectStore(store);
                    else if (!request.result.objectStoreNames.contains(store)) {
                        state.cause = failure("schema", "Existing database is missing the requested store");
                        tx.abort();
                    }
                    // v1 -> v2 changes only the version. Unknown stores,
                    // indexes and every existing value are preserved verbatim.
                } catch (error) {
                    state.cause ??= failure("schema", error);
                    try { tx.abort(); } catch (_) { /* await terminal request */ }
                }
            };
            request.onerror = event => {
                if (event.target !== request) return;
                const error = state.cause ?? failure("open", request.error);
                cleanRequest(request);
                terminalRefusal(error);
            };
            request.onsuccess = () => {
                const db = request.result;
                cleanRequest(request);
                if (state.canceled || state.cause || db.version !== 2 || !db.objectStoreNames.contains(store)) {
                    db.close();
                    terminalRefusal(state.cause ?? failure("schema", "Invalid database version or store"));
                    return;
                }
                const owner = { db, store, active: new Set(), closing: false, released: false,
                    reason: null, unlock, closed: null, finishClose: null };
                owner.closed = new Promise(resolve => { owner.finishClose = resolve; });
                owner.closeLock = lockJob;
                db.onversionchange = () => {
                    owner.reason = failure("lost", "Database version changed; reopen required");
                    closeOwner(owner);
                };
                db.onclose = () => {
                    owner.reason = failure("lost", "Database connection closed; reopen required");
                    closeOwner(owner);
                };
                state.owner = owner;
                state.status = "opened";
                delivered = true;
                deliver(owner);
            };
            return held;
        },
    );
    lockJob.then(() => {
        if (state.owner) {
            state.owner.released = true;
            state.owner.finishClose();
        }
        if (!delivered) {
            state.status = "refused";
            const error = state.cause ?? failure("open", "Open ended without an owner");
            error.schemaUpgraded = state.schemaUpgraded;
            refuse(error);
        }
    }, error => {
        state.status = "refused";
        refuse(failure("unavailable", error, state.schemaUpgraded));
    });
    return state;
}

export function openResult(state) {
    return state.result.then(async owner => {
        if (state.canceled) {
            await closeOwner(owner);
            const error = state.cause ?? failure("canceled", "Opening canceled");
            error.schemaUpgraded = state.schemaUpgraded;
            state.status = "refused";
            throw error;
        }
        return owner;
    });
}
export function openStatus(state) { return state.status; }
export function cancelOpen(state) { state.cancel(); }
export function ownerActive(owner) { return !owner.closing && !owner.released; }
function releaseDrained(owner) {
    if (!owner.closing || owner.active.size !== 0 || owner.released) return;
    owner.db.onversionchange = owner.db.onclose = null;
    owner.db.close();
    owner.unlock();
}
export function closeOwner(owner) {
    owner.closing = true;
    // close prevents new transactions and drains existing transactions. Normal
    // close does not emit "close", so tracked terminal events decide release.
    owner.db.close();
    releaseDrained(owner);
    return owner.closed;
}

function transaction(owner, mode, submit) {
    if (!ownerActive(owner)) return Promise.reject(owner.reason ?? failure("lost", "Owner is closing; reopen required"));
    let tx;
    try { tx = owner.db.transaction(owner.store, mode); }
    catch (error) { return Promise.reject(failure("transaction", error)); }
    owner.active.add(tx);
    return new Promise((resolve, reject) => {
        let value, error;
        const end = committed => {
            tx.oncomplete = tx.onabort = tx.onerror = null;
            owner.active.delete(tx);
            if (committed) resolve(value);
            else reject(error ?? failure("transaction", tx.error ?? "Transaction aborted"));
            releaseDrained(owner);
        };
        tx.oncomplete = () => end(true);
        tx.onabort = () => end(false);
        // Request errors bubble; this is not a terminal event.
        tx.onerror = event => { error ??= failure("transaction", event.target.error); };
        try { submit(tx.objectStore(owner.store), result => { value = result; }); }
        catch (caught) {
            error = failure("transaction", caught);
            try { tx.abort(); } catch (_) { /* completion/abort still owns lease */ }
        }
    });
}
export function readKey(owner, key) {
    return transaction(owner, "readonly", (store, result) => {
        const request = store.get(key);
        request.onsuccess = () => result(request.result);
    });
}
export function readKeys(owner) {
    return transaction(owner, "readonly", (store, result) => {
        const request = store.getAllKeys();
        request.onsuccess = () => result(request.result);
    });
}
function writeOps(store, ops) {
    for (const [key, bytes] of ops) {
        if (bytes === null) store.delete(key);
        else store.put(bytes, key);
    }
}
export function applyOps(owner, ops) {
    return transaction(owner, "readwrite", store => writeOps(store, ops));
}
export function transactOps(owner, compute) {
    return transaction(owner, "readwrite", store => {
        // Submit both reads synchronously. The compute and writes occur inside
        // the final success event, where this transaction is still active.
        const keys = store.getAllKeys();
        const values = store.getAll();
        values.onsuccess = () => {
            try { writeOps(store, compute(keys.result, values.result)); }
            catch (_) { store.transaction.abort(); }
        };
    });
}
