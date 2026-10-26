// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Private C ABI linked only into the signed custodian process. This is not
//! the IPC protocol: native confirmation/key handoff never reach dispatch.
use super::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Mutex,
};

static CORE: Mutex<Option<Custodian>> = Mutex::new(None);
const MAX_REPLY: usize = 64 * 1024;

unsafe fn input<'a>(pointer: *const u8, length: usize, limit: usize) -> Result<&'a [u8]> {
    if pointer.is_null() || length > limit {
        return Err(Refusal("invalid-native-buffer"));
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, length) })
}
unsafe fn key(pointer: *const u8, length: usize) -> Result<Zeroizing<[u8; 32]>> {
    if length != 32 {
        return Err(Refusal("invalid-native-key"));
    }
    let bytes = unsafe { input(pointer, length, 32)? };
    let mut key = Zeroizing::new([0; 32]);
    key.copy_from_slice(bytes);
    Ok(key)
}
fn reply(output: *mut u8, capacity: usize, operation: impl FnOnce() -> Result<Value>) -> usize {
    if output.is_null() || capacity > MAX_REPLY {
        return 0;
    }
    let result = match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(value)) => value,
        Ok(Err(Refusal(code))) => json!({"refused":code}),
        Err(_) => json!({"refused":"native-custody-panic"}),
    };
    let Ok(bytes) = serde_json::to_vec(&result) else {
        return 0;
    };
    if bytes.len() > capacity {
        return 0;
    }
    // The native caller owns a writable output allocation of capacity bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
    }
    bytes.len()
}

/// Caller buffers must be valid for the stated lengths and may not alias the
/// output. Initialization receives the native provider's new-item result;
/// existing keys may only reopen, never regenerate a missing profile.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn djinn_custodian_initialize(
    config: *const u8,
    config_len: usize,
    root: *const u8,
    root_len: usize,
    newly_created_key: bool,
    output: *mut u8,
    capacity: usize,
) -> usize {
    reply(output, capacity, || {
        let config: SyntheticConfig =
            serde_json::from_slice(unsafe { input(config, config_len, MAX_REQUEST_BYTES)? })
                .map_err(|_| Refusal("invalid-native-config"))?;
        let key = unsafe { key(root, root_len)? };
        let mut guard = CORE.lock().map_err(|_| Refusal("custodian-unavailable"))?;
        if guard.is_some() {
            return Err(Refusal("already-initialized"));
        }
        let core = if newly_created_key {
            Custodian::create_synthetic(config, key)?
        } else {
            Custodian::open_synthetic(config, key)?
        };
        let status = core.status();
        *guard = Some(core);
        Ok(status)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn djinn_custodian_dispatch(
    principal: *const u8,
    principal_len: usize,
    request: *const u8,
    request_len: usize,
    output: *mut u8,
    capacity: usize,
) -> usize {
    reply(output, capacity, || {
        let principal = std::str::from_utf8(unsafe { input(principal, principal_len, 256)? })
            .map_err(|_| Refusal("invalid-native-principal"))?;
        let request = unsafe { input(request, request_len, MAX_REQUEST_BYTES)? };
        let mut guard = CORE.lock().map_err(|_| Refusal("custodian-unavailable"))?;
        let core = guard.as_mut().ok_or(Refusal("not-initialized"))?;
        match core.dispatch(principal, request)? {
            Dispatch::Reply(value) => Ok(value),
            Dispatch::Confirm(facts) => {
                Ok(json!({"nativeConfirmationRequired":true,"facts":facts}))
            },
        }
    })
}

/// Only the resident-owned native dialog calls this. No IPC operation routes
/// client-supplied tickets or acceptance booleans here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn djinn_custodian_confirm(
    ticket: *const u8,
    ticket_len: usize,
    accepted: bool,
    output: *mut u8,
    capacity: usize,
) -> usize {
    reply(output, capacity, || {
        let ticket = std::str::from_utf8(unsafe { input(ticket, ticket_len, 64)? })
            .map_err(|_| Refusal("invalid-native-ticket"))?;
        CORE.lock()
            .map_err(|_| Refusal("custodian-unavailable"))?
            .as_mut()
            .ok_or(Refusal("not-initialized"))?
            .complete_confirmation(ticket, accepted)
    })
}

/// The native host performs OS user verification and retrieves a fresh key
/// directly from its own data-protection Keychain group before calling here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn djinn_custodian_unlock(
    root: *const u8,
    root_len: usize,
    output: *mut u8,
    capacity: usize,
) -> usize {
    reply(output, capacity, || {
        let key = unsafe { key(root, root_len)? };
        let mut guard = CORE.lock().map_err(|_| Refusal("custodian-unavailable"))?;
        let core = guard.as_mut().ok_or(Refusal("not-initialized"))?;
        core.unlock_after_native_confirmation(key)?;
        Ok(core.status())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn answer(f: impl FnOnce(*mut u8, usize) -> usize) -> Value {
        let mut bytes = vec![0; MAX_REPLY];
        let count = f(bytes.as_mut_ptr(), bytes.len());
        assert!(count > 0 && count <= bytes.len());
        serde_json::from_slice(&bytes[..count]).unwrap()
    }
    #[test]
    fn the_native_abi_uses_real_sealed_profiles_but_client_json_cannot_complete_confirmation() {
        let directory = tempfile::tempdir().unwrap();
        let run = "abcdef0123456789abcdef0123456789";
        std::fs::write(directory.path().join(".djinn-synthetic-custody"), run).unwrap();
        let config=serde_json::to_vec(&json!({"run_id":run,"directory":directory.path(),"profile":"h1-synthetic","principal":"signed-client"})).unwrap();
        let root = [0x64; 32];
        let created = answer(|out, cap| unsafe {
            djinn_custodian_initialize(
                config.as_ptr(),
                config.len(),
                root.as_ptr(),
                root.len(),
                true,
                out,
                cap,
            )
        });
        assert_eq!(created["backend"], "castellan-sealed-profile-v1");
        assert_eq!(created["realConsumersAllowed"], false);
        let who = b"signed-client";
        let dispatch = |value: Value| {
            let request = serde_json::to_vec(&value).unwrap();
            answer(|out, cap| unsafe {
                djinn_custodian_dispatch(
                    who.as_ptr(),
                    who.len(),
                    request.as_ptr(),
                    request.len(),
                    out,
                    cap,
                )
            })
        };
        let request = dispatch(
            json!({"op":"sign","namespace":ROOT_NAMESPACE,"challenge":"native ABI synthetic","instance":created["instance"]}),
        );
        assert_eq!(request["nativeConfirmationRequired"], true);
        let ticket = request["facts"]["ticket"].as_str().unwrap();
        assert_eq!(
            dispatch(json!({"op":"approve","ticket":ticket,"accepted":true}))["refused"],
            "invalid-request"
        );
        let signature = answer(|out, cap| unsafe {
            djinn_custodian_confirm(ticket.as_ptr(), ticket.len(), true, out, cap)
        });
        let public: [u8; 32] = STANDARD
            .decode(signature["publicKey"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let sig: [u8; 64] = STANDARD
            .decode(signature["signature"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        assert!(
            personae::Ed25519PublicKey::from_bytes(&public)
                .unwrap()
                .verify(
                    b"native ABI synthetic",
                    &personae::Ed25519Signature::from_bytes(&sig)
                )
        );
        let invalid = answer(|out, cap| unsafe {
            djinn_custodian_dispatch(
                std::ptr::null(),
                0,
                std::ptr::null(),
                MAX_REQUEST_BYTES + 1,
                out,
                cap,
            )
        });
        assert_eq!(invalid["refused"], "invalid-native-buffer");
        assert_eq!(
            dispatch(json!({"op":"lock","instance":created["instance"]}))["locked"],
            true
        );
        let unlocked = answer(|out, cap| unsafe {
            djinn_custodian_unlock(root.as_ptr(), root.len(), out, cap)
        });
        assert_eq!(unlocked["locked"], false);
    }
}
