// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit local resident endpoint. One JSON command per stdin line; responses
//! contain status or refusal, never identity keys. EOF joins resident work while
//! retaining the configured restart intent. `stop` durably disables that intent.

use djinn::{
    moot_site::{MootSiteOwnersV1, MootSiteTransport, open_owned},
    resident_site::GeminiListenerPolicyV1,
};
use serde::Deserialize;
use site_model::{MAX_ENCODED_SNAPSHOT_BYTES, PublishedSnapshotV1, publication::SiteBindingV1};
use std::{
    io::{BufRead, Read, Write},
    path::PathBuf,
};

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Configure {
        binding: SiteBindingV1,
        policy: GeminiListenerPolicyV1,
        #[serde(default)]
        transport: MootSiteTransport,
    },
    Start,
    Status,
    Stop,
    Import {
        records: PathBuf,
        snapshot: Option<PathBuf>,
    },
    Shutdown,
}

fn read_bounded(
    path: &std::path::Path,
    limit: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("Input exceeds its byte limit".into());
    }
    Ok(bytes)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: djinn-moot-site ENDPOINT_ROOT OWNERS_JSON".into());
    }
    let owners: MootSiteOwnersV1 =
        serde_json::from_slice(&read_bounded(std::path::Path::new(&args[1]), 16 * 1024)?)?;
    let mut service = open_owned(std::path::Path::new(&args[0]), &owners).await?;
    let emit = |value: serde_json::Value| -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", serde_json::to_string(&value)?);
        std::io::stdout().flush()?;
        Ok(())
    };
    emit(serde_json::json!({"ok":true,"status":service.status()}))?;
    let mut input = std::io::stdin().lock();
    loop {
        let mut line = Vec::new();
        let count = input.by_ref().take(65537).read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        if count > 65536 {
            return Err("Resident command exceeds 64 KiB".into());
        }
        let command = match serde_json::from_slice::<Command>(&line) {
            Ok(command) => command,
            Err(error) => {
                emit(serde_json::json!({"ok":false,"error":error.to_string()}))?;
                continue;
            },
        };
        let result: Result<(), String> = match command {
            Command::Configure {
                binding,
                policy,
                transport,
            } => {
                service
                    .configure_transport(binding, policy, transport)
                    .await
            },
            Command::Start => service.start().await.map(|_| ()),
            Command::Stop => service.stop(),
            Command::Status => Ok(()),
            Command::Import { records, snapshot } => {
                let prepared = (|| -> Result<_, Box<dyn std::error::Error>> {
                    let snapshot = snapshot
                        .map(|path| -> Result<_, Box<dyn std::error::Error>> {
                            Ok(PublishedSnapshotV1::decode(&read_bounded(
                                &path,
                                MAX_ENCODED_SNAPSHOT_BYTES,
                            )?)?)
                        })
                        .transpose()?;
                    Ok((std::fs::File::open(records)?, snapshot))
                })();
                match prepared {
                    Ok((records, snapshot)) => service.import_public(records, snapshot).await,
                    Err(error) => Err(error.to_string()),
                }
            },
            Command::Shutdown => {
                let report = service.shutdown()?;
                emit(
                    serde_json::json!({"ok":true,"tasks":{"joined":report.joined(),"cancelled":report.cancelled,"panicked":report.panicked,"rejected":report.rejected}}),
                )?;
                break;
            },
        };
        match result {
            Ok(()) => emit(serde_json::json!({"ok":true,"status":service.status()}))?,
            Err(error) => {
                emit(serde_json::json!({"ok":false,"error":error,"status":service.status()}))?
            },
        }
    }
    service.shutdown()?;
    Ok(())
}
