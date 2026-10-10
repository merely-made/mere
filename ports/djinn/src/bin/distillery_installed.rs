// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Installed Distillery configuration, answered by the running resident.
//!
//! This binary intentionally stops before starting a resident. The remaining
//! construction inputs are a mesh-owned store/retention policy and a
//! device-owned `HostConfig`/`ResidentSettings`; accepting defaults here would
//! make Distillery the unchosen scheduler and device-policy authority.
//!
//! It opens no vault (dramatis repo plan, D5): `inspect` asks djinn's custody
//! route for the roster and the protection, and says the identity is pending
//! when djinn is absent or Locked (D12). It lives in djinn's package since
//! DR-C because the client is graphshell's, and graphshell already depends on
//! distillery, so distillery cannot depend back.

use std::path::PathBuf;

use distillery::{InstalledAuthority, InstalledSettings};
use graphshell::native::app_admission::{AppId, configured_app_endpoint};
use graphshell::native::custody_client::BlockingCustodyClient;
use personae::ProfileId;

/// The application name this tool connects to djinn as.
const APP: &str = "distillery";

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("distillery-installed: {error}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage());
    };
    let options = Options::parse(rest)?;
    match command.as_str() {
        "configure" => {
            let profile = options.profile.ok_or_else(usage)?;
            let settings = InstalledAuthority::configure(&options.data_root, ProfileId(profile))
                .map_err(|error| error.to_string())?;
            println!(
                "configured Distillery at {} to use Personae profile `{}`",
                options.data_root.display(),
                settings.profile
            );
        },
        "inspect" => {
            if options.profile.is_some() {
                return Err("--profile only belongs to configure".into());
            }
            let settings = InstalledSettings::load(&options.data_root)
                .map_err(|error| error.to_string())?
                .ok_or("Distillery is not configured: run `configure` first")?;
            let endpoint = options.app_endpoint.unwrap_or_else(configured_app_endpoint);
            let answer = BlockingCustodyClient::open_at(&endpoint, AppId::new(APP))
                .and_then(|mut djinn| Ok((djinn.status()?, djinn.roster()?)));
            let (status, roster) = match answer {
                Ok(answer) => answer,
                Err(error) if error.is_pending() => {
                    println!(
                        "Distillery profile: {}\nPersonae identity: pending ({error})\nProduct root: {}",
                        settings.profile,
                        options.data_root.display()
                    );
                    return Ok(());
                },
                Err(error) => return Err(error.to_string()),
            };
            if status.lock == graphshell::identity::VaultLockView::Locked {
                println!("Distillery profile: {}\nPersonae identity: pending (the vault is locked)\nProduct root: {}",
                    settings.profile, options.data_root.display());
                return Ok(());
            }
            let known = roster
                .entries
                .iter()
                .any(|entry| entry.id == settings.profile_id());
            if !known {
                return Err(format!(
                    "the configured profile `{}` is not in djinn's vault",
                    settings.profile
                ));
            }
            println!(
                "Distillery profile: {}\nPersonae protection: {:?} ({:?})\nProduct root: {}\n\
                 Resident start remains gated on caller-supplied mesh retention and device host settings.",
                settings.profile,
                status.protection,
                status.lock,
                options.data_root.display()
            );
        },
        _ => return Err(usage()),
    }
    Ok(())
}

struct Options {
    data_root: PathBuf,
    app_endpoint: Option<String>,
    profile: Option<String>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut data_root = None;
        let mut app_endpoint = None;
        let mut profile = None;
        let mut values = args.iter();
        while let Some(option) = values.next() {
            match option.as_str() {
                "--data-root" => data_root = Some(PathBuf::from(next(&mut values, option)?)),
                "--app-endpoint" => app_endpoint = Some(next(&mut values, option)?.to_string()),
                "--profile" => profile = Some(next(&mut values, option)?.to_string()),
                _ => return Err(usage()),
            }
        }
        Ok(Self {
            data_root: data_root.ok_or_else(usage)?,
            app_endpoint,
            profile,
        })
    }
}

fn next<'a>(
    values: &mut impl Iterator<Item = &'a String>,
    option: &str,
) -> Result<&'a str, String> {
    values
        .next()
        .map(String::as_str)
        .ok_or_else(|| format!("{option} needs a value"))
}

fn usage() -> String {
    "usage:\n  distillery-installed configure --data-root <path> --profile <personae-profile>\n  distillery-installed inspect --data-root <path> [--app-endpoint <endpoint>]".into()
}
