// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The certificate half of `personae-vault`: the authority, minting, and
//! the one-command enrollment that replaces per-pair `authorized_keys`
//! setup.
//!
//! Split from `main.rs` to stay under the 600-line ceiling.

use personae::delegation::Issue;
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::custody::vault::Profile;
use crate::custody::{ssh_face, ssh_krl, ssh_slot};
use insigne::delegation::{DelegationRevocation, SignedDelegationRevocation};
use personae::enroll::{self, device_id_for_host};
use personae::ssh_ca::{SshCertAuthority, UserCertRequest};
use personae::ssh_face::FacePolicy;
use personae::{IdentityProvider, InMemoryProvider};
use ssh_key::public::PublicKey;

use super::format_key;

// ─── the certificate authority ────────────────────────────────────────────

/// The authority for this profile, derived from its master key.
pub(crate) fn authority(profile: &Profile) -> Result<SshCertAuthority, String> {
    let provider = InMemoryProvider::from_seed(profile.master.to_seed());
    SshCertAuthority::derive(&provider).map_err(|err| format!("derive the CA: {err}"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or_default()
}

pub(crate) fn cmd_ca(
    output: &mut dyn Write,
    profile: &Profile,
    rest: &[String],
) -> Result<(), String> {
    let ca = authority(profile)?;
    let patterns = flag(rest, "--patterns").unwrap_or_else(|| "*".to_string());
    writeln!(output, "fingerprint: {}", ca.fingerprint()).map_err(|error| error.to_string())?;
    writeln!(output, "\n# TrustedUserCAKeys / cert-authority key")
        .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "{}",
        ca.trusted_user_ca_line().map_err(str_err)?.trim_end()
    )
    .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "\n# ~/.ssh/known_hosts line for hosts serving a host certificate"
    )
    .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "{}",
        enroll::known_hosts_line(&ca, &patterns).map_err(str_err)?
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn cmd_mint(
    output: &mut dyn Write,
    profile: &Profile,
    rest: &[String],
) -> Result<(), String> {
    let slot_key = rest
        .first()
        .ok_or("mint needs a slot, e.g. `mint ssh:SHA256:d3tQ --host q-pc.local`")?;
    // The device is this machine unless told otherwise: a grant is held by
    // the machine carrying the credential, which is what makes revoking a
    // lost machine meaningful. See ssh_ca::self_grant.
    let device_name = flag(rest, "--device").unwrap_or_else(enroll::local_host_name);
    let principal = flag(rest, "--principal").unwrap_or_else(default_principal);
    let hours = flag(rest, "--hours")
        .map(|value| value.parse::<u64>().map_err(|_| "--hours wants a number"))
        .transpose()?
        .unwrap_or(12);

    let private = resolve_ssh_key(profile, slot_key)?;

    let policy = ssh_face::effective_policy(profile).map_err(str_err)?;
    if !policy.principals.contains(&principal) {
        return Err(format!(
            "face {:?} may not name principal {principal:?} (it carries: {})",
            profile.id.0,
            policy.principals.join(", ")
        ));
    }
    let ledger = ssh_krl::load_ledger(profile).map_err(str_err)?;
    let provider = InMemoryProvider::from_seed(profile.master.to_seed());
    let grant = personae::ssh_ca::self_grant(
        &provider,
        device_id_for_host(&device_name),
        &policy.action_refs(),
        hours * 3_600_000,
        now_ms(),
    )
    .map_err(|err| format!("issue the ssh grant: {err}"))?;
    let ca = authority(profile)?;
    let cert = ca
        .mint_user_cert(
            &UserCertRequest {
                grant: &grant,
                subject: &PublicKey::from(&private),
                principals: vec![principal.clone()],
                // The face's own limits outrank the command line: a flag may
                // narrow a face further, never widen it.
                force_command: flag(rest, "--force-command").or(policy.force_command.clone()),
                source_address: flag(rest, "--source-address").or(policy.source_address.clone()),
                ledger: &ledger,
            },
            now_ms(),
        )
        .map_err(str_err)?;

    let encoded = cert.to_openssh().map_err(|err| format!("{err}"))?;
    match flag(rest, "--out") {
        Some(path) => {
            std::fs::write(&path, format!("{encoded}\n"))
                .map_err(|err| format!("write {path}: {err}"))?;
            writeln!(output, "wrote {path}").map_err(|error| error.to_string())?;
        },
        None => writeln!(output, "{encoded}").map_err(|error| error.to_string())?,
    }
    writeln!(
        output,
        "minted for {principal}, held by {device_name}, valid {hours}h, grant {}",
        &personae::ssh_ca::key_id_for(&grant.certificate.id())[..16]
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn cmd_enroll_host(
    output: &mut dyn Write,
    profile: &Profile,
    rest: &[String],
) -> Result<(), String> {
    rest.first().ok_or("enroll-host needs a target")?;
    let ca = authority(profile)?;
    writeln!(
        output,
        "{}",
        enroll::system_sshd_snippet(
            &ca,
            "/etc/ssh/personae_ca.pub",
            Some("/etc/ssh/ssh_host_ed25519_key-cert.pub"),
        )
        .map_err(str_err)?
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn prepare_enrollment(
    profile: &Profile,
    rest: &[String],
) -> Result<super::Enrollment, String> {
    let target = rest
        .first()
        .ok_or("enroll-host needs a target, e.g. `enroll-host markik@q-pc.local`")?;
    let (user, host) = enroll::split_target(target);
    let policy = ssh_face::effective_policy(profile).map_err(str_err)?;
    let principal = flag(rest, "--principal")
        .or_else(|| user.map(str::to_string))
        .unwrap_or_else(|| {
            policy
                .principals
                .first()
                .cloned()
                .unwrap_or_else(default_principal)
        });
    let ca = authority(profile)?;

    let line = enroll::user_trust_line(&ca, &[principal.clone()]).map_err(str_err)?;
    let script = enroll::user_install_script(&line);
    Ok(super::Enrollment {
        target: target.clone(),
        script,
        confirmation: format!(
            "enrolled {host}: certificates for principal {principal:?} are now accepted\n\
            re-running replaces the authorized_keys CA line\n\
            host-key prompts still apply; `enroll-host {target} --system` prints the root half\n"
        ),
    })
}

/// Resolve a slot name among this profile's SSH *keys* only.
///
/// The generic resolver matches any mod_id by prefix, and `ssh` is a prefix
/// of `ssh-face`, so `mint ssh` became ambiguous the moment face policies
/// existed. Minting needs a key, and a policy is not one.
fn resolve_ssh_key(profile: &Profile, typed: &str) -> Result<ssh_key::PrivateKey, String> {
    let mut matches: Vec<_> = ssh_slot::ssh_slots(profile)
        .into_iter()
        .filter(|slot| format_key(&slot.key).starts_with(typed))
        .collect();
    match matches.len() {
        1 => Ok(matches.remove(0).private),
        0 => Err(format!(
            "no ssh key matches {typed:?} (`personae-vault list` shows what is there)"
        )),
        _ => {
            let mut names: Vec<String> = matches.iter().map(|slot| format_key(&slot.key)).collect();
            names.sort();
            Err(format!(
                "{typed:?} is ambiguous; it matches:\n  {}",
                names.join("\n  ")
            ))
        },
    }
}

/// Revoke a device: this vault stops certifying it, immediately.
pub(crate) fn cmd_revoke(
    output: &mut dyn Write,
    storage: &dyn crate::custody::IdentityStorage,
    id: &personae::vault::ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let name = rest
        .first()
        .filter(|arg| !arg.starts_with("--"))
        .cloned()
        .ok_or("revoke needs a device, e.g. `revoke thinkpad.local`")?;
    let mut profile = storage
        .load_profile(id)
        .map_err(|err| format!("load profile: {err}"))?;
    let device = device_id_for_host(&name);
    let provider = InMemoryProvider::from_seed(profile.master.to_seed());
    let policy = ssh_face::effective_policy(&profile).map_err(str_err)?;
    let now = now_ms();

    // Revocation names a grant, and the grant it names is one issued for
    // this device: what closes the device is the serial they share.
    let grant = personae::ssh_ca::self_grant(&provider, device, &policy.action_refs(), 1, now)
        .map_err(|err| format!("name the device's grant: {err}"))?;
    let revocation = SignedDelegationRevocation::issue(
        &provider,
        DelegationRevocation::new(
            grant.certificate.id(),
            provider.master_public_key().to_bytes(),
            grant.certificate.scope.clone(),
            now,
            *blake3::hash(&now.to_le_bytes()).as_bytes(),
        ),
    )
    .map_err(|err| format!("sign the revocation: {err}"))?;

    let mut ledger = ssh_krl::load_ledger(&profile).map_err(str_err)?;
    if !ledger.fold(&revocation, &name) {
        return Err("the revocation did not verify".to_string());
    }
    ssh_krl::store_ledger(&mut profile, &ledger).map_err(str_err)?;
    storage
        .save_profile(&profile)
        .map_err(|err| format!("save profile: {err}"))?;

    let serial = personae::ssh_ca::serial_for_device(device);
    writeln!(output, "revoked {name} (certificate serial {serial})")
        .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "  this vault will not certify it again, so its access ends when its"
    )
    .map_err(|error| error.to_string())?;
    writeln!(output, "  last certificate expires (at most 12h)")
        .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "  to close it on a host now: personae-vault krl --out <file>, then"
    )
    .map_err(|error| error.to_string())?;
    writeln!(output, "  RevokedKeys in that host's sshd_config")
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// Render the revocation list, and compile it when ssh-keygen is present.
pub(crate) fn cmd_krl(
    output: &mut dyn Write,
    profile: &Profile,
    rest: &[String],
) -> Result<(), String> {
    let ledger = ssh_krl::load_ledger(profile).map_err(str_err)?;
    if ledger.is_empty() {
        writeln!(output, "nothing is revoked").map_err(|error| error.to_string())?;
        return Ok(());
    }
    let spec = ledger.krl_spec();
    let Some(out) = flag(rest, "--out") else {
        write!(output, "{spec}").map_err(|error| error.to_string())?;
        writeln!(
            output,
            "# compile with: ssh-keygen -k -s <ca.pub> -f <krl> <this file>"
        )
        .map_err(|error| error.to_string())?;
        return Ok(());
    };

    // ssh-keygen needs the CA key on disk to bind serial and id records to
    // the authority they revoke under, so it goes to a sibling file the
    // caller can keep: it is public material, and a host deploying the KRL
    // wants it anyway.
    let ca = authority(profile)?;
    let spec_path = format!("{out}.spec");
    let ca_path = format!("{out}.ca.pub");
    std::fs::write(&spec_path, &spec).map_err(|err| format!("write {spec_path}: {err}"))?;
    std::fs::write(&ca_path, ca.trusted_user_ca_line().map_err(str_err)?)
        .map_err(|err| format!("write {ca_path}: {err}"))?;

    let status = Command::new("ssh-keygen")
        .args(["-k", "-s", &ca_path, "-f", &out, &spec_path])
        .status()
        .map_err(|err| format!("run ssh-keygen: {err}"))?;
    if !status.success() {
        return Err(format!(
            "ssh-keygen could not compile the KRL (spec left at {spec_path})"
        ));
    }
    writeln!(
        output,
        "wrote {out} ({} device(s) revoked)",
        ledger.devices().count()
    )
    .map_err(|error| error.to_string())?;
    writeln!(
        output,
        "  deploy: copy to the host and name it in sshd_config's RevokedKeys"
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Show or set this face's SSH policy.
pub(crate) fn cmd_face(
    output: &mut dyn Write,
    storage: &dyn crate::custody::IdentityStorage,
    id: &personae::vault::ProfileId,
    rest: &[String],
) -> Result<(), String> {
    let mut profile = storage
        .load_profile(id)
        .map_err(|err| format!("load profile: {err}"))?;

    if let Some(shape) = rest.first().filter(|arg| !arg.starts_with("--")) {
        let principal = flag(rest, "--principal").unwrap_or_else(|| id.0.clone());
        let policy = match shape.as_str() {
            "work" => FacePolicy::work(principal),
            "research" => FacePolicy::research(principal),
            "burner" => FacePolicy::burner(
                principal,
                flag(rest, "--command").unwrap_or_else(|| "true".into()),
            ),
            other => {
                return Err(format!(
                    "unknown face shape {other:?} (work, research, burner)"
                ));
            },
        };
        ssh_face::store_policy(&mut profile, &policy).map_err(str_err)?;
        storage
            .save_profile(&profile)
            .map_err(|err| format!("save profile: {err}"))?;
        writeln!(output, "face {:?} is now a {shape} face", id.0)
            .map_err(|error| error.to_string())?;
    }

    let policy = ssh_face::effective_policy(&profile).map_err(str_err)?;
    let stored = ssh_face::load_policy(&profile).map_err(str_err)?.is_some();
    writeln!(
        output,
        "face: {:?}{}",
        id.0,
        if stored {
            ""
        } else {
            " (no stored policy; showing the default)"
        }
    )
    .map_err(|error| error.to_string())?;
    writeln!(output, "  principals: {}", policy.principals.join(", "))
        .map_err(|error| error.to_string())?;
    writeln!(output, "  actions:    {}", policy.action_refs().join(", "))
        .map_err(|error| error.to_string())?;
    if let Some(command) = &policy.force_command {
        writeln!(output, "  forced:     {command}").map_err(|error| error.to_string())?;
    }
    if let Some(addresses) = &policy.source_address {
        writeln!(output, "  from:       {addresses}").map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn default_principal() -> String {
    std::env::var("USER").unwrap_or_else(|_| "root".into())
}

/// Read `--name value` out of the trailing arguments.
fn flag(rest: &[String], name: &str) -> Option<String> {
    rest.iter()
        .position(|arg| arg == name)
        .and_then(|at| rest.get(at + 1))
        .cloned()
}

fn str_err<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}
