// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Terminal vault operations executed inside djinn custody.

use std::io::Write;
use std::path::PathBuf;

use crate::custody::bootstrap;
use crate::custody::ssh_slot;
use crate::custody::vault::{
    IdentitySlot, IdentityStorage, Profile, ProfileId, ProtocolKey, UnlockTier,
};
use ssh_key::HashAlg;
use ssh_key::private::PrivateKey;
use ssh_key::public::PublicKey;

mod certs;
use certs::{cmd_ca, cmd_enroll_host, cmd_face, cmd_krl, cmd_mint, cmd_revoke};

/// A public CA installation script. Executed by the terminal after custody
/// returns, so SSH can call the live agent without waiting on the vault mutex.
pub(crate) struct Enrollment {
    pub target: String,
    pub script: String,
    pub confirmation: String,
}

/// Execute a known vault command against the resident's already-open storage.
/// Only public text and certificates are written to `out`.
pub(crate) fn execute(
    storage: &dyn IdentityStorage,
    profile: &ProfileId,
    description: &str,
    command: &str,
    rest: &[String],
    out: &mut dyn Write,
) -> Result<Option<Enrollment>, String> {
    if command == "enroll-host" && !rest.iter().any(|arg| arg == "--system") {
        return certs::prepare_enrollment(&load(storage, profile)?, rest).map(Some);
    }
    match command {
        "profiles" => cmd_profiles(out, storage, description),
        "new-profile" => cmd_new_profile(out, storage, rest),
        "list" => cmd_list(out, storage, profile, description),
        "show" => cmd_show(out, storage, profile, rest),
        "add-ssh" => cmd_add_ssh(out, storage, profile, rest),
        "pub" => cmd_pub(out, storage, profile, rest),
        "remove" => cmd_remove(out, storage, profile, rest),
        "ca" => cmd_ca(out, &load(storage, profile)?, rest),
        "mint" => cmd_mint(out, &load(storage, profile)?, rest),
        "enroll-host" => cmd_enroll_host(out, &load(storage, profile)?, rest),
        "face" => cmd_face(out, storage, profile, rest),
        "revoke" => cmd_revoke(out, storage, profile, rest),
        "krl" => cmd_krl(out, &load(storage, profile)?, rest),
        other => Err(format!("unknown command {other:?}\n\n")),
    }?;
    Ok(None)
}
fn load(storage: &dyn IdentityStorage, id: &ProfileId) -> Result<Profile, String> {
    storage.load_profile(id).map_err(|err| {
        let known = storage.list_profiles().unwrap_or_default();
        if known.iter().any(|summary| &summary.id == id) {
            return format!("load profile {:?}: {err}", id.0);
        }
        let names: Vec<&str> = known.iter().map(|summary| summary.id.0.as_str()).collect();
        let existing = if names.is_empty() {
            "this vault has no profiles yet".to_string()
        } else {
            format!("this vault has: {}", names.join(", "))
        };
        format!(
            "no profile {:?} ({existing})\ncreate it with: personae-vault new-profile {}",
            id.0, id.0
        )
    })
}

// ─── commands ─────────────────────────────────────────────────────────────

fn cmd_profiles(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    description: &str,
) -> Result<(), String> {
    writeln!(out, "storage: {description}").map_err(|error| error.to_string())?;
    let mut summaries = storage
        .list_profiles()
        .map_err(|err| format!("list profiles: {err}"))?;
    if summaries.is_empty() {
        writeln!(
            out,
            "\nno profiles yet (`personae-vault new-profile default` creates one)"
        )
        .map_err(|error| error.to_string())?;
        return Ok(());
    }
    summaries.sort_by(|a, b| a.id.cmp(&b.id));
    writeln!(out, "").map_err(|error| error.to_string())?;
    for summary in summaries {
        let plural = if summary.slot_count == 1 { "" } else { "s" };
        writeln!(
            out,
            "{}  \"{}\", {} slot{plural}",
            summary.id.0, summary.display_name, summary.slot_count
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn cmd_new_profile(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    rest: &[String],
) -> Result<(), String> {
    let id = ProfileId(rest.first().ok_or("new-profile needs an id")?.clone());
    let (profile, created) = bootstrap::load_or_create_profile(storage, &id)
        .map_err(|err| format!("create profile: {err}"))?;
    if created {
        writeln!(out, "created profile {:?}", profile.id.0).map_err(|error| error.to_string())?;
    } else {
        writeln!(out, "profile {:?} already exists", profile.id.0)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn cmd_list(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    description: &str,
) -> Result<(), String> {
    let profile = load(storage, id)?;
    let plural = if profile.slots.len() == 1 { "" } else { "s" };
    writeln!(
        out,
        "profile {:?} ({}), {} slot{plural}",
        profile.id.0,
        profile.display_name,
        profile.slots.len()
    )
    .map_err(|error| error.to_string())?;
    writeln!(out, "storage: {description}").map_err(|error| error.to_string())?;
    if profile.slots.is_empty() {
        writeln!(
            out,
            "\nno slots yet (`personae-vault add-ssh <file>` imports an SSH key)"
        )
        .map_err(|error| error.to_string())?;
        return Ok(());
    }

    let mut keys: Vec<&ProtocolKey> = profile.slots.keys().collect();
    keys.sort();
    writeln!(out, "").map_err(|error| error.to_string())?;
    for key in keys {
        let slot = &profile.slots[key];
        writeln!(out, "{}", format_key(key)).map_err(|error| error.to_string())?;
        writeln!(
            out,
            "  {}, {}, {} unlock",
            slot.kind(),
            category(slot),
            tier_label(slot.unlock_tier())
        )
        .map_err(|error| error.to_string())?;
        if let Some(line) = ssh_summary(slot) {
            writeln!(out, "  {line}").map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn cmd_show(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let profile = load(storage, id)?;
    let key = resolve_key(&profile, rest.first().ok_or("show needs a slot key")?)?;
    let slot = &profile.slots[&key];

    writeln!(out, "{}", format_key(&key)).map_err(|error| error.to_string())?;
    writeln!(out, "  kind:      {}", slot.kind()).map_err(|error| error.to_string())?;
    writeln!(out, "  category:  {}", category(slot)).map_err(|error| error.to_string())?;
    writeln!(out, "  unlock:    {}", tier_label(slot.unlock_tier()))
        .map_err(|error| error.to_string())?;
    writeln!(out, "  payload:   {} bytes (not shown)", payload_len(slot))
        .map_err(|error| error.to_string())?;
    if let IdentitySlot::Bootstrap { state_dir, .. } = slot {
        writeln!(out, "  state dir: {}", state_dir.display()).map_err(|error| error.to_string())?;
    }
    if let Some(line) = ssh_summary(slot) {
        writeln!(out, "  ssh:       {line}").map_err(|error| error.to_string())?;
    }
    writeln!(out, "  lineage:   {:?}", slot.lineage()).map_err(|error| error.to_string())?;
    writeln!(
        out,
        "  losing this device: {}",
        slot.lineage().device_loss_note()
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn cmd_add_ssh(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let mut path = None;
    let mut tier = UnlockTier::Session;
    for arg in rest {
        match arg.as_str() {
            "--per-use" => tier = UnlockTier::PerUse,
            _ if path.is_none() => path = Some(PathBuf::from(arg)),
            other => return Err(format!("unexpected argument {other:?}")),
        }
    }
    let path = path.ok_or("add-ssh needs a private-key file")?;
    let bytes = zeroize::Zeroizing::new(
        std::fs::read(&path).map_err(|err| format!("read {}: {err}", path.display()))?
    );
    let private = PrivateKey::from_openssh(&bytes)
        .map_err(|err| format!("parse {} as an OpenSSH private key: {err}", path.display()))?;

    let key = ssh_slot::protocol_key_for(&private);
    if let AddSsh::AlreadyHeld(held) = add_ssh_key(storage, id, &private, tier)? {
        writeln!(out, "already held {}", format_key(&key)).map_err(|error| error.to_string())?;
        writeln!(out, "  {}", describe_ssh(&private)).map_err(|error| error.to_string())?;
        writeln!(out, "  unlock: {} (left untouched)", tier_label(held))
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    writeln!(out, "imported {}", format_key(&key)).map_err(|error| error.to_string())?;
    writeln!(out, "  {}", describe_ssh(&private)).map_err(|error| error.to_string())?;
    writeln!(out, "  unlock: {}", tier_label(tier)).map_err(|error| error.to_string())?;
    if tier == UnlockTier::PerUse {
        writeln!(
            out,
            "  note: the agent refuses to sign with per-use slots until a confirmation UI exists"
        )
        .map_err(|error| error.to_string())?;
    }
    writeln!(
        out,
        "\nthe vault now holds this key. Deleting the original file leaves the vault as its \
         only holder: {}",
        path.display()
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// What `add-ssh` did with a key.
#[derive(Debug, PartialEq)]
enum AddSsh {
    Imported,
    /// The fingerprint was held; nothing was written. Carries its tier.
    AlreadyHeld(UnlockTier),
}

/// The import door, as castellan's and `ssh-add`'s (ruling 60).
fn add_ssh_key(
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    private: &PrivateKey,
    tier: UnlockTier,
) -> Result<AddSsh, String> {
    let key = ssh_slot::protocol_key_for(private);
    let profile = load(storage, id)?;
    // Ruling 54: a held key is never rewritten, its tier included.
    if let Some(held) = profile.slots.get(&key) {
        return Ok(AddSsh::AlreadyHeld(held.unlock_tier()));
    }
    // Ruling 56: only keys the agent can sign are taken in.
    crate::custody::ssh_sign::check_signable(private.key_data())
        .map_err(|refused| format!("refused: {refused}"))?;
    let slot = ssh_slot::slot_for(private, tier).map_err(|err| err.to_string())?;
    let mut vault = crate::custody::IdentityVault::with_profile(storage, profile);
    vault
        .add_slot(key, slot)
        .map_err(|err| format!("store slot: {err}"))?;
    Ok(AddSsh::Imported)
}

fn cmd_pub(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let profile = load(storage, id)?;
    let key = resolve_key(&profile, rest.first().ok_or("pub needs a slot key")?)?;
    let private = ssh_slot::private_key_from_slot(&profile.slots[&key])
        .map_err(|err| format!("decode ssh slot: {err}"))?;
    let public = PublicKey::from(&private);
    writeln!(
        out,
        "{}",
        public
            .to_openssh()
            .map_err(|err| format!("encode public key: {err}"))?
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn cmd_remove(
    out: &mut dyn Write,
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let profile = load(storage, id)?;
    let key = resolve_key(&profile, rest.first().ok_or("remove needs a slot key")?)?;
    let slot = &profile.slots[&key];
    let note = slot.lineage().device_loss_note().to_string();
    let is_ssh = slot.kind() == ssh_slot::SSH_MOD_ID;

    let mut vault = crate::custody::IdentityVault::with_profile(storage, profile);
    let removed = vault
        .remove_slot(&key)
        .map_err(|err| format!("remove slot: {err}"))?;
    if !removed {
        return Err(format!("slot {} was not present", format_key(&key)));
    }
    writeln!(out, "removed {}", format_key(&key)).map_err(|error| error.to_string())?;
    writeln!(out, "  {note}").map_err(|error| error.to_string())?;
    if is_ssh {
        writeln!(
            out,
            "  the vault no longer holds this key; re-import from a file to restore it"
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

// ─── formatting helpers ───────────────────────────────────────────────────

pub(crate) fn format_key(key: &ProtocolKey) -> String {
    match &key.instance {
        Some(instance) => format!("{}:{instance}", key.mod_id),
        None => key.mod_id.clone(),
    }
}

fn category(slot: &IdentitySlot) -> &'static str {
    match slot {
        IdentitySlot::Direct { .. } => "direct",
        IdentitySlot::Bootstrap { .. } => "bootstrap",
    }
}

fn payload_len(slot: &IdentitySlot) -> usize {
    match slot {
        IdentitySlot::Direct { payload, .. } => payload.len(),
        IdentitySlot::Bootstrap { bootstrap, .. } => bootstrap.len(),
    }
}

fn tier_label(tier: UnlockTier) -> String {
    match tier {
        UnlockTier::Session => "session".to_string(),
        UnlockTier::ShortTtl { idle_seconds } => {
            format!("short-ttl ({idle_seconds}s idle; not enforced yet)")
        },
        UnlockTier::PerUse => "per-use (agent refuses to sign; no confirmation UI yet)".to_string(),
    }
}

fn describe_ssh(private: &PrivateKey) -> String {
    let public = PublicKey::from(private);
    let comment = private.comment();
    let comment = if comment.is_empty() {
        String::new()
    } else {
        format!(", comment {comment:?}")
    };
    format!(
        "{}, {}{comment}",
        private.algorithm(),
        public.fingerprint(HashAlg::Sha256)
    )
}

fn ssh_summary(slot: &IdentitySlot) -> Option<String> {
    if slot.kind() != ssh_slot::SSH_MOD_ID {
        return None;
    }
    match ssh_slot::private_key_from_slot(slot) {
        Ok(private) => Some(describe_ssh(&private)),
        Err(err) => Some(format!("unreadable ssh payload: {err}")),
    }
}

/// Resolve a user-typed slot key against the profile, accepting a unique
/// prefix so full fingerprints need not be typed.
fn resolve_key(profile: &Profile, typed: &str) -> Result<ProtocolKey, String> {
    let exact = match typed.split_once(':') {
        Some((mod_id, instance)) => ProtocolKey::new(mod_id, Some(instance.to_string())),
        None => ProtocolKey::new(typed, None),
    };
    if profile.slots.contains_key(&exact) {
        return Ok(exact);
    }

    let mut matches: Vec<&ProtocolKey> = profile
        .slots
        .keys()
        .filter(|key| format_key(key).starts_with(typed))
        .collect();
    match matches.len() {
        1 => Ok(matches.remove(0).clone()),
        0 => Err(format!(
            "no slot matches {typed:?} (`personae-vault list` shows what is there)"
        )),
        _ => {
            matches.sort();
            let candidates: Vec<String> = matches.iter().map(|key| format_key(key)).collect();
            Err(format!(
                "{typed:?} is ambiguous; it matches:\n  {}",
                candidates.join("\n  ")
            ))
        },
    }
}

// The unit-testable core here is `resolve_key`; everything else is I/O
// formatting exercised by the end-to-end CLI run recorded in the plan.
#[cfg(test)]
mod tests;
