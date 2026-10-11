// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::authority::PersonaeHost;
use crate::custody::{IdentityStorage, IdentityVault};
use crate::resident::CastellanResident;
use chatelaine::{
    Collection, CollectionId, Credential, CredentialId, CredentialKind, ItemId, ItemState, Link,
};

/// A failure with a receipt for every write completed before it. Persona
/// batches commit independently; an import never claims cross-persona atomicity.
#[derive(Debug)]
pub struct CommitFailure {
    /// Completed writes and notices.
    pub receipt: Receipt,
    /// A fixed diagnostic containing no imported credential values.
    pub reason: &'static str,
}
impl std::fmt::Display for CommitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason)
    }
}
impl std::error::Error for CommitFailure {}

impl Import {
    /// Commit into the resident's existing sealed stores and native SSH
    /// authority. Missing or extra choices are refused before any write.
    pub fn commit<S: IdentityStorage + 'static>(
        self,
        destinations: BTreeMap<String, Destination>,
        resident: &CastellanResident,
        host: &PersonaeHost<S>,
    ) -> Result<Receipt, CommitFailure> {
        let notices = self.preview.notices.clone();
        host.with_cxf_vault(|vault| self.commit_held(destinations, resident, vault))
            .unwrap_or_else(|_| {
                Err(CommitFailure {
                    receipt: Receipt {
                        items: 0,
                        ssh_keys: 0,
                        notices,
                        delete_plaintext_source: true,
                    },
                    reason: "vault is locked",
                })
            })
    }

    fn commit_held<S: IdentityStorage + 'static>(
        self,
        destinations: BTreeMap<String, Destination>,
        resident: &CastellanResident,
        vault: &mut IdentityVault<S>,
    ) -> Result<Receipt, CommitFailure> {
        let mut receipt = Receipt {
            items: 0,
            ssh_keys: 0,
            notices: self.preview.notices,
            delete_plaintext_source: true,
        };
        let fail = |receipt, reason| CommitFailure { receipt, reason };
        if destinations.len() != self.accounts.len()
            || self
                .accounts
                .iter()
                .any(|a| !destinations.contains_key(&a.id))
        {
            return Err(fail(
                receipt,
                "choose exactly one persona for each CXF account",
            ));
        }
        for destination in destinations.values() {
            if vault.storage().load_profile(&destination.profile).is_err() {
                return Err(fail(receipt, "destination profile is unavailable"));
            }
            if resident.items(destination.persona).list().is_err() {
                return Err(fail(receipt, "destination item store is unavailable"));
            }
        }
        // Source ids are scoped by account, even when accounts choose the
        // same persona. Only items with real stored/quarantined credentials
        // receive an index entry; reference-only inputs never mint an item.
        let mut targets = BTreeMap::new();
        for account in &self.accounts {
            for item in &account.items {
                if item.credentials.iter().any(|c| c.kind.is_some()) {
                    targets.insert(
                        (
                            account.id.clone(),
                            item.metadata.source_id.as_ref().unwrap().to_string(),
                        ),
                        (destinations[&account.id].persona, item.metadata.id),
                    );
                }
            }
        }
        // Resolve chains of reference-only aliases; cycles stay dangling.
        for _ in 0..self.accounts.iter().map(|a| a.items.len()).sum::<usize>() {
            let before = targets.len();
            for account in &self.accounts {
                for item in &account.items {
                    let key = (
                        account.id.clone(),
                        item.metadata.source_id.as_ref().unwrap().to_string(),
                    );
                    if targets.contains_key(&key) {
                        continue;
                    }
                    for reference in item.credentials.iter().filter_map(|c| c.reference.as_ref()) {
                        let target = (
                            reference.0.clone().unwrap_or_else(|| account.id.clone()),
                            reference.1.clone(),
                        );
                        if let Some(resolved) = targets.get(&target).copied() {
                            if resolved.0 == destinations[&account.id].persona {
                                targets.insert(key.clone(), resolved);
                                break;
                            }
                        }
                    }
                }
            }
            if before == targets.len() {
                break;
            }
        }
        type Batch = Vec<(Item, Vec<(CredentialId, Payload)>)>;
        let mut batches: BTreeMap<PersonaId, (Batch, Vec<Collection>)> = BTreeMap::new();
        let mut ssh_imports = Vec::new();
        for account in self.accounts {
            let destination = &destinations[&account.id];
            let (batch, collections) = batches.entry(destination.persona).or_default();
            for prepared in account.items {
                let source_id = prepared.metadata.source_id.as_ref().unwrap().to_string();
                let mut vault = prepared.metadata.clone();
                let mut quarantine = prepared.metadata;
                quarantine.id = ItemId::from_random(crate::items::random_id_bytes());
                quarantine.state = ItemState::Quarantined;
                let mut vault_payloads = Vec::new();
                let mut quarantine_payloads = Vec::new();
                for mut credential in prepared.credentials {
                    if let Some(private) = credential.ssh.take() {
                        ssh_imports.push((
                            destination.profile.clone(),
                            private,
                            destination.ssh_unlock_policy.clone(),
                        ));
                    }
                    if let Some(reference) = credential.reference.take() {
                        if let Some(link) = resolve(
                            &account.id,
                            &source_id,
                            destination.persona,
                            &reference,
                            &targets,
                            &mut receipt.notices,
                        ) {
                            vault.credentials.push(Credential {
                                id: CredentialId::from_random(crate::items::random_id_bytes()),
                                kind: CredentialKind::ItemReference(link),
                            });
                        }
                    }
                    if let Some(kind) = credential.kind.take() {
                        let id = CredentialId::from_random(crate::items::random_id_bytes());
                        let (item, payloads) = if credential.quarantine {
                            (&mut quarantine, &mut quarantine_payloads)
                        } else {
                            (&mut vault, &mut vault_payloads)
                        };
                        item.credentials.push(Credential { id, kind });
                        if let Some(payload) = credential.payload.take() {
                            payloads.push((id, payload));
                        }
                    }
                }
                let has_vault = !vault_payloads.is_empty();
                let has_quarantine = !quarantine_payloads.is_empty();
                if has_vault && has_quarantine {
                    vault.credentials.push(Credential {
                        id: CredentialId::from_random(crate::items::random_id_bytes()),
                        kind: CredentialKind::ItemReference(Link {
                            item: quarantine.id,
                        }),
                    });
                    quarantine.credentials.push(Credential {
                        id: CredentialId::from_random(crate::items::random_id_bytes()),
                        kind: CredentialKind::ItemReference(Link { item: vault.id }),
                    });
                } else if has_quarantine {
                    quarantine.id = vault.id;
                    quarantine.credentials.extend(
                        vault
                            .credentials
                            .into_iter()
                            .filter(|c| matches!(c.kind, CredentialKind::ItemReference(_))),
                    );
                    vault.credentials = Vec::new();
                }
                if has_vault {
                    batch.push((vault, vault_payloads));
                }
                if has_quarantine {
                    batch.push((quarantine, quarantine_payloads));
                }
            }
            for collection in account.collections.0.as_array().into_iter().flatten() {
                match collection_from(
                    collection,
                    &account.id,
                    destination.persona,
                    &targets,
                    &mut receipt.notices,
                ) {
                    Ok(collection) => collections.push(collection),
                    Err(_) => return Err(fail(receipt, "invalid collection metadata")),
                }
            }
        }
        // All destination choices, item shapes and links are resolved before
        // the first write. Native SSH slots remain their only key home.
        for (profile, private, unlock_policy) in ssh_imports {
            if PersonaeHost::<S>::import_ssh_private_for_held_vault(
                vault,
                &profile,
                private,
                dramatis::intents::ImportSshKeyNativeIntentV1 { unlock_policy },
            )
            .is_err()
            {
                return Err(fail(
                    receipt,
                    "native SSH import failed; see completed-write receipt",
                ));
            }
            receipt.ssh_keys += 1;
        }
        for (persona, (batch, collections)) in batches {
            match resident
                .items(persona)
                .begin()
                .import_batch(batch, collections)
            {
                Ok(count) => receipt.items += count,
                Err(_) => {
                    return Err(fail(
                        receipt,
                        "sealed persona batch failed; see completed-write receipt",
                    ));
                },
            }
        }
        Ok(receipt)
    }
}

fn resolve(
    account: &str,
    item: &str,
    persona: PersonaId,
    reference: &(Option<String>, String),
    targets: &BTreeMap<(String, String), (PersonaId, ItemId)>,
    notices: &mut Vec<Notice>,
) -> Option<Link> {
    let key = (
        reference.0.clone().unwrap_or_else(|| account.into()),
        reference.1.clone(),
    );
    match targets.get(&key) {
        Some((target, id)) if *target == persona => Some(Link { item: *id }),
        target => {
            notices.push(Notice {
                account: account.into(),
                item: Some(item.into()),
                credential: None,
                reason: if target.is_some() {
                    "cross-persona link omitted"
                } else {
                    "dangling item link omitted"
                }
                .into(),
            });
            None
        },
    }
}
fn collection_from(
    value: &serde_json::Value,
    account: &str,
    persona: PersonaId,
    targets: &BTreeMap<(String, String), (PersonaId, ItemId)>,
    notices: &mut Vec<Notice>,
) -> Result<Collection, ImportError> {
    let source_id = parse::text(value, "id").ok_or(ImportError("missing collection id"))?;
    let mut items = Vec::new();
    for reference in value["items"]
        .as_array()
        .ok_or(ImportError("missing collection members"))?
    {
        let target =
            parse::text(reference, "item").ok_or(ImportError("missing collection item id"))?;
        if let Some(link) = resolve(
            account,
            &source_id,
            persona,
            &(parse::text(reference, "account"), target),
            targets,
            notices,
        ) {
            items.push(link);
        }
    }
    let mut sub_collections = Vec::new();
    for child in value["subCollections"].as_array().into_iter().flatten() {
        sub_collections.push(collection_from(child, account, persona, targets, notices)?);
    }
    Ok(Collection {
        id: CollectionId::from_random(crate::items::random_id_bytes()),
        source_id: Some(parse::source(&source_id)?),
        title: parse::text(value, "title").ok_or(ImportError("missing collection title"))?,
        subtitle: parse::text(value, "subtitle"),
        created_at: value["creationAt"].as_u64(),
        modified_at: value["modifiedAt"].as_u64(),
        items,
        sub_collections,
    })
}
