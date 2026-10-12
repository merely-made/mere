// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Synthetic identities, actual signed governance/membership/contributions.
#[cfg(test)]
use crate as moot;
use eidetic::*;
use gemot::moot::constitution::{CapabilityGrant, ConstitutionRules};
use gemot::moot::hosting::{DeviceCeiling, HostingPolicy, hosting_cap};
use gemot::moot::*;
use muniment::Backend;
use personae::{IdentityProvider, InMemoryProvider};
use servitor::{Cap, cap_path};
pub const ID: MootId = MootId([41; 32]);
pub const BODY:&[u8]=b"Shared resource handler notes\n\nA verified owner copy survives source absence and restart. Hosting is a separate signed, bounded promise.\n";
pub fn founder() -> InMemoryProvider {
    InMemoryProvider::from_seed([1; 32])
}
pub fn author() -> InMemoryProvider {
    InMemoryProvider::from_seed([2; 32])
}
pub fn reader() -> InMemoryProvider {
    InMemoryProvider::from_seed([7; 32])
}
pub fn read_cap() -> Cap {
    Cap::scope("moot/resources/read").unwrap()
}
pub fn copy_cap() -> Cap {
    Cap::scope("moot/resources/retain").unwrap()
}
pub fn retention_settings() -> MootRetentionSettings {
    MootRetentionSettings {
        revision: PolicyRevision(proofs::Digest::blake3(b"resource handler fixture history")),
        availability: AvailabilityPolicy {
            promised_floor: KeepBound::Forever,
        },
        erasure: ErasurePolicy {
            history_ceiling: KeepBound::UntilCheckpoint,
        },
    }
}
pub fn manifest() -> BlobManifest {
    BlobManifest {
        id: ManifestId::of_blob(BODY),
        content_hash: Hash::of(BODY),
        byte_size: BODY.len() as u64,
        schema: SchemaRef::from_id(ManifestId::of_blob(b"text/plain")),
        created_at: Timestamp(1),
        last_accessed: None,
        sources: vec![BlobSource::Embedded {
            bytes: BODY.to_vec(),
            media_type: "text/plain".into(),
        }],
        privacy: PrivacyClass::PublicPortable,
        provenance: ProvenanceRecord::self_imported("resource handler fixture"),
        trust: TrustEnvelope::self_asserted(),
        schema_metadata: serde_json::Value::Null,
        manifest_version: BlobManifest::CURRENT_VERSION,
    }
}
pub async fn initialize<B: Backend + Clone>(
    service: &Moot<B>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let founder = founder();
    let author = author();
    let reader = reader();
    let mut rules = ConstitutionRules::founder_only(founder.master_public_key().to_bytes());
    rules.admission = mien::Policy::MembersOnly {
        rate_limit: 100,
        rate_window_ms: 60_000,
    };
    for (n, subject, cap) in [
        (
            1,
            author.master_public_key().to_bytes(),
            gemot::moot::records::fauna_cap(),
        ),
        (2, reader.master_public_key().to_bytes(), read_cap()),
        (3, reader.master_public_key().to_bytes(), copy_cap()),
        (
            4,
            reader.master_public_key().to_bytes(),
            hosting_cap("public")?,
        ),
    ] {
        rules.grant(CapabilityGrant {
            id: [n; 32],
            subject,
            path_prefix: cap_path(&cap),
            not_before_ms: 0,
            expires_at_ms: None,
            delegation_depth: 0,
        });
    }
    service.found([1; 32], None, None, rules, 0).await?;
    service
        .update_membership_for_identity(
            &founder,
            MootMembershipAction::Create {
                initial_members: vec![
                    MootMember {
                        member: founder.master_public_key().to_bytes(),
                        access: MootAccessLevel::Manage,
                    },
                    MootMember {
                        member: author.master_public_key().to_bytes(),
                        access: MootAccessLevel::Write,
                    },
                    MootMember {
                        member: reader.master_public_key().to_bytes(),
                        access: MootAccessLevel::Write,
                    },
                ],
            },
        )
        .await?;
    for n in 1..=2 {
        service
            .share_authorized_for_identity(
                &author,
                *manifest().id.0.as_bytes(),
                "text/plain".into(),
                format!("Handler notes contribution {n}"),
                n,
            )
            .await?;
    }
    Ok(())
}
pub async fn config<B: Backend + Clone>(
    service: &Moot<B>,
) -> std::result::Result<moot::resource_coop::handlers::Config, Box<dyn std::error::Error>> {
    Ok(moot::resource_coop::handlers::Config {
        public_export_manifests: [*manifest().id.0.as_bytes()].into(),
        read_cap: read_cap(),
        retain_cap: copy_cap(),
        retention_revision: [51; 32],
        retention_limits: eidetic::retention::Limits {
            max_payload_bytes: 4096,
            max_stored_bytes: 8192,
        },
        hosting: HostingPolicy {
            constitution_revision: service.snapshot().await?.governance.revision,
            revision: [52; 32],
            audience: "public".into(),
            max_bytes: 4096,
            max_duration_ms: 3_600_000,
        },
        device_ceiling: DeviceCeiling {
            max_bytes: 2048,
            max_duration_ms: 1_800_000,
        },
        device: [53; 32],
        requested_host_bytes: 2048,
        requested_duration_ms: 600_000,
    })
}
