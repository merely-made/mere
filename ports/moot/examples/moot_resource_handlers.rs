// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Local acceptance host over production handlers, signed Moot facts and redb.
//! `moot_resource_handlers read|read-offline|dispatch|serve|export <fixture-state>`.
//! Identity seeds and ceilings are explicit fixture settings, not user credentials.
#[path = "moot_resource_handlers/support.rs"]
mod support;
use gemot::moot::{MootError, MootFile, MootGovernanceError};
use moot::resource_coop::handlers::Owner;
use moot::resource_coop::*;
use personae::IdentityProvider;
use std::{
    io::{Read, Write},
    path::PathBuf,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).ok_or("Expected mode")?;
    let directory = PathBuf::from(
        std::env::args()
            .nth(2)
            .ok_or("Expected fixture-state path")?,
    );
    let marker = directory.join("MERE_BUILD_OWNER.json");
    let owner_marker: serde_json::Value = serde_json::from_slice(&std::fs::read(marker)?)?;
    if owner_marker["owner"] != "Moot resource handler acceptance" {
        return Err("Fixture state owner marker mismatch".into());
    }
    let founder = support::founder();
    let identity = support::reader();
    let service = MootFile::open(
        directory.join("moot"),
        support::ID,
        founder.master_public_key().to_bytes(),
        support::retention_settings(),
    )
    .await?;
    match service.snapshot().await {
        Err(MootError::Governance(MootGovernanceError::NotFounded)) => {
            support::initialize(&service).await?
        },
        Err(error) => return Err(error.into()),
        Ok(_) => {},
    }
    if mode == "export" {
        let mut drop = Vec::new();
        service
            .export_plain_drop(
                &mut drop,
                stickleback::DropExportProfile::default(),
                stickleback::DropLimits::default(),
            )
            .await?;
        std::io::stdout().write_all(&drop)?;
        return Ok(());
    }
    let mut encounter = muniment::MemoryBackend::default();
    if mode != "read-offline" {
        eidetic::manifest::save_manifest(&mut encounter, &support::manifest()).await?;
    }
    let mut custody = muniment::RedbBackend::open(directory.join("copies.redb"))?;
    let config = support::config(&service).await?;
    let mut owner = Owner {
        moot: &service,
        identity: &identity,
        encounter: &mut encounter,
        custody: &mut custody,
        sealer: None,
        config,
        selected_collection: None,
    };
    let at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    if mode == "read" || mode == "read-offline" {
        let snapshot = owner.read(at_ms).await?;
        println!(
            "{}",
            serde_json::json!({"snapshots":[{"reading":1,"snapshot":snapshot},{"reading":2,"snapshot":snapshot}]})
        );
    } else {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let command: serde_json::Value = serde_json::from_str(&input)?;
        let reference: ResourceRef = serde_json::from_value(command["target"]["resource"].clone())?;
        if mode == "serve" {
            let body = owner
                .serve(
                    reference,
                    command["audience"].as_str().ok_or("Audience missing")?,
                    at_ms,
                )
                .await?;
            println!("{}", serde_json::json!({"text":String::from_utf8(body)?}));
        } else if mode == "dispatch" {
            let supplied_reader: [u8; 32] = serde_json::from_value(command["reader"].clone())?;
            let result = async {
                if supplied_reader != identity.master_public_key().to_bytes() {
                    return Err("Foreign signing reader".to_string());
                }
                let mut pane = Pane::new(owner.read(at_ms).await?).map_err(|e| e.to_string())?;
                pane.select_resource(reference).map_err(|e| e.to_string())?;
                let action: ResourceAction =
                    serde_json::from_value(command["target"]["action"].clone())
                        .map_err(|e| e.to_string())?;
                let supplied_terms: Option<ActionTerms> =
                    serde_json::from_value(command["target"]["observed_terms"].clone())
                        .map_err(|e| e.to_string())?;
                let request = pane.prepare_resource(action).map_err(|e| e.to_string())?;
                if let Target::Resource { observed_terms, .. } = &request.target {
                    if observed_terms != &supplied_terms {
                        return Err("Changed action terms; refresh and prepare again".into());
                    }
                }
                Ok::<_, String>(owner.dispatch(&request, at_ms).await)
            }
            .await;
            let (outcome, body) =
                result.unwrap_or_else(|reason| (Outcome::Refused { reason }, None));
            println!(
                "{}",
                serde_json::json!({"outcome":outcome,"text":body.map(String::from_utf8).transpose()?})
            );
        } else {
            return Err("Unknown mode".into());
        }
    }
    Ok(())
}
