// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The no-residue instrument (vault lock plan, ruling 6).
//!
//! A tracking global allocator, test-only, owns this binary's heap. Canary
//! keys are planted in a profile; while armed, every freed block is scanned
//! before it reaches the system allocator, and at each scenario's end every
//! block still live is scanned too. A hit is a canary (any 16-byte window of
//! it, raw or as serde_json's decimal array) left in memory nobody owns.
//!
//! The positive control runs first in the same process: an uncleared Vec
//! must be found, a `Zeroizing` one must not. Stack copies and memory the OS
//! allocates (DPAPI's `LocalAlloc`) are outside what this can see.
//!
//! No libtest harness: the allocator is process-wide, and one thread keeps
//! the scan free of races.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use personae::{
    CredentialLineage, Ed25519Keypair, IdentitySlot, IdentityStorage, IdentityVault,
    PassphraseEncryptedStorage, Profile, ProfileId, ProtocolKey, SealedProfileStorage,
    SecretBytes, UnlockTier,
};
use zeroize::Zeroizing;

// ─── The tracker ──────────────────────────────────────────────────────────

const WINDOW: usize = 16;
const MAX_CANARY: usize = 64;
const MAX_CANARIES: usize = 4;
const TABLE: usize = 1 << 20;
const TOMBSTONE: usize = usize::MAX;
const MAX_HITS: usize = 64;

struct Canary {
    bytes: [u8; MAX_CANARY],
    len: usize,
    /// Byte positions by value (raw match anchors).
    at: [[u8; 4]; 256],
    at_count: [u8; 256],
    /// Unique position per value when the canary's bytes are distinct
    /// (needed for the decimal-array match), else -1.
    unique: [i16; 256],
    json: bool,
}

#[derive(Clone, Copy)]
struct Hit {
    phase: usize,
    live: bool,
    canary: usize,
    json: bool,
    size: usize,
}

struct State {
    canaries: [Option<Canary>; MAX_CANARIES],
    hits: [Option<Hit>; MAX_HITS],
    hit_count: usize,
    live_ptr: [usize; TABLE],
    live_size: [usize; TABLE],
    tracked: usize,
    overflow: bool,
}

struct Shared(UnsafeCell<State>);
// Every access holds `LOCK`.
unsafe impl Sync for Shared {}

static STATE: Shared = Shared(UnsafeCell::new(State {
    canaries: [const { None }; MAX_CANARIES],
    hits: [None; MAX_HITS],
    hit_count: 0,
    live_ptr: [0; TABLE],
    live_size: [0; TABLE],
    tracked: 0,
    overflow: false,
}));
static LOCK: AtomicBool = AtomicBool::new(false);
static ARMED: AtomicBool = AtomicBool::new(false);
static PHASE: AtomicUsize = AtomicUsize::new(0);

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    while LOCK
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        std::hint::spin_loop();
    }
    let out = f(unsafe { &mut *STATE.0.get() });
    LOCK.store(false, Ordering::Release);
    out
}

fn slot_of(ptr: usize) -> usize {
    (ptr >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15) % TABLE
}

impl State {
    fn insert(&mut self, ptr: usize, size: usize) {
        let mut i = slot_of(ptr);
        for _ in 0..TABLE {
            if self.live_ptr[i] == 0 || self.live_ptr[i] == TOMBSTONE {
                self.live_ptr[i] = ptr;
                self.live_size[i] = size;
                self.tracked += 1;
                return;
            }
            i = (i + 1) % TABLE;
        }
        self.overflow = true;
    }

    fn remove(&mut self, ptr: usize) {
        if self.tracked == 0 {
            return;
        }
        let mut i = slot_of(ptr);
        for _ in 0..TABLE {
            match self.live_ptr[i] {
                0 => return,
                p if p == ptr => {
                    self.live_ptr[i] = TOMBSTONE;
                    self.tracked -= 1;
                    return;
                },
                _ => i = (i + 1) % TABLE,
            }
        }
    }

    fn scan(&mut self, block: &[u8], live: bool) {
        for c in 0..MAX_CANARIES {
            let Some(canary) = &self.canaries[c] else { continue };
            let raw = raw_hit(canary, block);
            let json = !raw && canary.json && json_hit(canary, block);
            if (raw || json) && self.hit_count < MAX_HITS {
                self.hits[self.hit_count] = Some(Hit {
                    phase: PHASE.load(Ordering::Relaxed),
                    live,
                    canary: c,
                    json,
                    size: block.len(),
                });
                self.hit_count += 1;
            } else if raw || json {
                self.hit_count = MAX_HITS;
            }
        }
    }
}

fn raw_hit(c: &Canary, block: &[u8]) -> bool {
    if block.len() < WINDOW {
        return false;
    }
    for i in 0..=block.len() - WINDOW {
        let b = block[i] as usize;
        for k in 0..c.at_count[b] as usize {
            let j = c.at[b][k] as usize;
            if j + WINDOW <= c.len && block[i..i + WINDOW] == c.bytes[j..j + WINDOW] {
                return true;
            }
        }
    }
    false
}

/// Any 16 consecutive canary bytes written as `a,b,c,...` decimals.
fn json_hit(c: &Canary, block: &[u8]) -> bool {
    let mut run = 0usize;
    let mut expect: i32 = -1;
    let mut num: u32 = 0;
    let mut in_num = false;
    let finish = |num: u32, run: &mut usize, expect: &mut i32| -> bool {
        let j = if num <= 255 { c.unique[num as usize] as i32 } else { -1 };
        if j < 0 {
            *run = 0;
            *expect = -1;
            return false;
        }
        *run = if *run > 0 && j == *expect { *run + 1 } else { 1 };
        *expect = j + 1;
        *run >= WINDOW
    };
    for &ch in block {
        if ch.is_ascii_digit() {
            num = (num * 10 + u32::from(ch - b'0')).min(1000);
            in_num = true;
            continue;
        }
        if in_num && finish(num, &mut run, &mut expect) {
            return true;
        }
        in_num = false;
        num = 0;
        if ch != b',' {
            run = 0;
        }
    }
    in_num && finish(num, &mut run, &mut expect)
}

struct Tracker;

unsafe impl GlobalAlloc for Tracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && ARMED.load(Ordering::Relaxed) {
            with_state(|s| s.insert(ptr as usize, layout.size()));
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let armed = ARMED.load(Ordering::Relaxed);
        with_state(|s| {
            s.remove(ptr as usize);
            if armed {
                let block = unsafe { std::slice::from_raw_parts(ptr, layout.size()) };
                s.scan(block, false);
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
    // `realloc` keeps the default: alloc, copy, dealloc, so the old block is
    // scanned like any other free.
}

#[global_allocator]
static GLOBAL: Tracker = Tracker;

// ─── Harness ──────────────────────────────────────────────────────────────

fn plant(index: usize, bytes: &[u8], json: bool) {
    assert!(bytes.len() <= MAX_CANARY && bytes.len() >= WINDOW);
    let mut canary = Canary {
        bytes: [0; MAX_CANARY],
        len: bytes.len(),
        at: [[0; 4]; 256],
        at_count: [0; 256],
        unique: [-1; 256],
        json,
    };
    canary.bytes[..bytes.len()].copy_from_slice(bytes);
    for (j, &b) in bytes.iter().enumerate() {
        let n = &mut canary.at_count[b as usize];
        assert!(*n < 4, "canary repeats a byte too often");
        canary.at[b as usize][*n as usize] = j as u8;
        *n += 1;
        if json {
            assert_eq!(canary.unique[b as usize], -1, "decimal canary needs distinct bytes");
            canary.unique[b as usize] = j as i16;
        }
    }
    with_state(|s| s.canaries[index] = Some(canary));
}

fn clear_canaries() {
    with_state(|s| s.canaries = [const { None }; MAX_CANARIES]);
}

fn arm() {
    with_state(|s| {
        s.hits = [None; MAX_HITS];
        s.hit_count = 0;
    });
    ARMED.store(true, Ordering::SeqCst);
}

/// Scan what is still live, disarm, and return every hit.
fn disarm() -> (Vec<Hit>, bool) {
    PHASE.store(99, Ordering::SeqCst);
    let (hits, overflow) = with_state(|s| {
        for i in 0..TABLE {
            let p = s.live_ptr[i];
            if p != 0 && p != TOMBSTONE {
                let block = unsafe { std::slice::from_raw_parts(p as *const u8, s.live_size[i]) };
                s.scan(block, true);
            }
        }
        let mut out = [None; MAX_HITS];
        out[..s.hit_count.min(MAX_HITS)].copy_from_slice(&s.hits[..s.hit_count.min(MAX_HITS)]);
        let overflow = s.overflow;
        s.live_ptr = [0; TABLE];
        s.tracked = 0;
        (out, overflow)
    });
    ARMED.store(false, Ordering::SeqCst);
    (hits.into_iter().flatten().collect(), overflow)
}

fn phase(n: usize) {
    PHASE.store(n, Ordering::SeqCst);
}

/// Distinct bytes with no visible pattern: an odd-step walk of 0..=255.
fn canary_bytes<const N: usize>(offset: usize) -> [u8; N] {
    let mut out = [0u8; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = ((offset + i) * 167 + 59) as u8;
    }
    out
}

const NAMES: [&str; MAX_CANARIES] = ["master seed", "slot payload", "storage root", "-"];

struct Report {
    failures: usize,
}

impl Report {
    fn check(&mut self, scenario: &str, phases: &[&str], hits: &[Hit], overflow: bool) {
        assert!(!overflow, "{scenario}: allocation table overflowed");
        if hits.is_empty() {
            println!("residue {scenario}: clean");
            return;
        }
        self.failures += 1;
        println!("residue {scenario}: {} hit(s)", hits.len());
        for h in hits {
            println!(
                "  {} {} ({}) in a {}-byte block, phase {}",
                if h.live { "live" } else { "freed uncleared" },
                NAMES[h.canary],
                if h.json { "decimal JSON" } else { "raw" },
                h.size,
                phases.get(h.phase).copied().unwrap_or(if h.phase == 99 {
                    "after drop"
                } else {
                    "?"
                }),
            );
        }
    }
}

// ─── Scenarios ────────────────────────────────────────────────────────────

fn canary_profile(seed: [u8; 32], payload: &[u8]) -> Profile {
    let mut profile = Profile::new(
        ProfileId("work".into()),
        "Work",
        Ed25519Keypair::from_seed(seed),
    );
    profile.slots.insert(
        ProtocolKey::new("canary", None),
        IdentitySlot::Direct {
            kind: "canary".into(),
            payload: SecretBytes::new(payload.to_vec()),
            lineage: CredentialLineage::LocallyDerived,
            unlock_tier: UnlockTier::Session,
        },
    );
    profile
}

const STORAGE_PHASES: [&str; 6] = ["setup", "save", "open", "switch", "list", "drop"];

/// Save, load, switch and list through one backend, then drop everything.
fn storage_round_trip<S: IdentityStorage>(storage: S, seed: [u8; 32], payload: &[u8]) {
    let profile = canary_profile(seed, payload);
    phase(1);
    storage.save_profile(&profile).unwrap();
    drop(profile);
    phase(2);
    let mut vault = IdentityVault::open(storage, &ProfileId("work".into())).unwrap();
    assert_eq!(vault.current_profile().master.to_seed(), seed);
    phase(3);
    vault.switch_profile(&ProfileId("work".into())).unwrap();
    phase(4);
    let listed = vault.storage().list_profiles().unwrap();
    assert_eq!(listed.len(), 1);
    phase(5);
    drop(listed);
    drop(vault);
}

fn main() {
    let mut report = Report { failures: 0 };
    let seed: [u8; 32] = canary_bytes(0);
    let payload: [u8; 48] = canary_bytes(32);

    // Positive control: the instrument must see what it claims to see.
    plant(0, &seed, true);
    arm();
    phase(1);
    drop(std::hint::black_box(seed.to_vec()));
    phase(2);
    drop(std::hint::black_box(serde_json::to_vec(&seed).unwrap()));
    phase(3);
    drop(std::hint::black_box(Zeroizing::new(seed.to_vec())));
    phase(4);
    let kept = std::hint::black_box(seed.to_vec());
    let (hits, _) = disarm();
    drop(kept);
    let saw = |p: usize, live: bool, json: bool| {
        hits.iter()
            .any(|h| h.phase == p && h.live == live && h.json == json)
    };
    let control = saw(1, false, false)
        && saw(2, false, true)
        && !hits.iter().any(|h| h.phase == 3)
        && saw(99, true, false);
    println!(
        "positive control: uncleared raw {} / uncleared decimal {} / zeroized {} / live {}",
        if saw(1, false, false) { "found" } else { "MISSED" },
        if saw(2, false, true) { "found" } else { "MISSED" },
        if hits.iter().any(|h| h.phase == 3) { "FLAGGED" } else { "clean" },
        if saw(99, true, false) { "found" } else { "MISSED" },
    );
    assert!(control, "the instrument failed its positive control");

    plant(1, &payload, true);

    // SealedProfileStorage, the AutoOs desktop backend (fixed root here).
    let dir = tempfile::tempdir().unwrap();
    let sealed = SealedProfileStorage::open_with_key(dir.path().join("sealed"), [0x51; 32]);
    arm();
    storage_round_trip(sealed, seed, &payload);
    let (hits, overflow) = disarm();
    report.check("sealed profile storage", &STORAGE_PHASES, &hits, overflow);

    // PassphraseEncryptedStorage, the portable passphrase vault.
    let pass = PassphraseEncryptedStorage::open(dir.path().join("vault.json"), b"canary").unwrap();
    arm();
    storage_round_trip(pass, seed, &payload);
    let (hits, overflow) = disarm();
    report.check("passphrase storage", &STORAGE_PHASES, &hits, overflow);

    // The DPAPI-held AutoOs root, read back from disk.
    #[cfg(windows)]
    {
        let path = dir.path().join("auto-unlock-root.json");
        let root = Zeroizing::new(
            personae::load_or_create_auto_unlock_root(&path)
                .unwrap()
                .unwrap(),
        );
        plant(2, root.as_ref(), false);
        arm();
        phase(1);
        let again = Zeroizing::new(
            personae::load_existing_auto_unlock_root(&path)
                .unwrap()
                .unwrap(),
        );
        assert_eq!(*again, *root);
        drop(again);
        let (hits, overflow) = disarm();
        report.check("DPAPI root", &["setup", "load"], &hits, overflow);
        with_state(|s| s.canaries[2] = None);
    }

    // The agent's listing, which mints a certificate from the master.
    #[cfg(feature = "agent")]
    agent_listing(&mut report, dir.path(), seed, &payload);

    clear_canaries();
    if report.failures > 0 {
        println!("no-residue: FAILED ({} scenario(s) left key residue)", report.failures);
        std::process::exit(1);
    }
    println!("no-residue: ok");
}

#[cfg(feature = "agent")]
fn agent_listing(report: &mut Report, root: &std::path::Path, seed: [u8; 32], payload: &[u8]) {
    use ssh_agent_lib::agent::Session;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    let storage = SealedProfileStorage::open_with_key(root.join("agent"), [0x52; 32]);
    let mut profile = canary_profile(seed, payload);
    let ssh = ssh_key::private::PrivateKey::from(ssh_key::private::Ed25519Keypair::from_seed(
        &[0x33; 32],
    ));
    profile.slots.insert(
        personae::agent::protocol_key_for(&ssh),
        personae::ssh_slot::slot_for(&ssh, UnlockTier::Session).unwrap(),
    );
    drop(ssh);
    let mut agent =
        personae::agent::VaultAgent::new(IdentityVault::with_profile(storage, profile));
    arm();
    phase(1);
    {
        let mut listing = std::pin::pin!(agent.request_identities());
        let Poll::Ready(identities) = listing
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        else {
            panic!("the listing awaits nothing");
        };
        // A certified key is listed twice: certificate, then bare key.
        assert_eq!(identities.unwrap().len(), 2);
    }
    phase(2);
    drop(agent);
    let (hits, overflow) = disarm();
    report.check("agent listing", &["setup", "list", "drop"], &hits, overflow);
}
