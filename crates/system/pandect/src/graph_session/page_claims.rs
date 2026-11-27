// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::time::SystemTime;

use mere::model::author::Author;
use mere::model::{PersistedResourceFacet, PersistedResourceRecord, ResourceNode};
use mere::acquire::page_claims::{KEPT_PAGE_CLAIMS_FACET, KeptPageClaims, PageClaims};
use mere::session::SessionGraph;
use muniment::Backend;
use oxrdf::NamedNode;

use super::{ChangeKind, GraphSession, LegacyTransactionBackend, PinnedView, SessionError};

impl<B: Backend, G: SessionGraph> GraphSession<B, G> {
    /// Read a capture's kept assertions and block provenance from this session.
    pub fn kept_page_claims(
        &self,
        source: &NamedNode,
    ) -> Result<Option<KeptPageClaims>, SessionError> {
        read_kept(self.graph(), source)
    }
}

impl<G: SessionGraph> PinnedView<G> {
    /// Read the same evidence as it was at this journal cursor.
    pub fn kept_page_claims(
        &self,
        source: &NamedNode,
    ) -> Result<Option<KeptPageClaims>, SessionError> {
        read_kept(self.graph(), source)
    }
}

impl<B: LegacyTransactionBackend + Clone, G: SessionGraph> GraphSession<B, G> {
    /// Admit one kept capture through the existing session journal. The caller
    /// owns the capture and policy and supplies an IRI for its immutable edition.
    /// Use a custody-backed session for claims from a sealed capture.
    /// An identical retry returns the existing evidence without another edit.
    pub async fn admit_page_claims(
        &mut self,
        author: Author,
        source: &NamedNode,
        claims: &PageClaims,
        at: SystemTime,
    ) -> Result<KeptPageClaims, SessionError> {
        let kept = KeptPageClaims::new(source.clone(), claims);
        let value = kept.facet_value();
        self.transact_change(at, false, |candidate| {
            candidate.try_edit_as(author.clone(), ChangeKind::Edit, |graph| {
                store_kept(graph, &author, &kept, value)
            })?;
            Ok(())
        })
        .await?;
        Ok(kept)
    }
}

fn read_kept<G: SessionGraph>(
    graph: &G,
    source: &NamedNode,
) -> Result<Option<KeptPageClaims>, SessionError> {
    let resource = ResourceNode::for_term(source.as_str());
    let Some(record) = graph.resources().resource_record(resource.id()) else {
        return Ok(None);
    };
    let Some(facet) = record
        .facets
        .iter()
        .find(|facet| facet.facet == KEPT_PAGE_CLAIMS_FACET)
    else {
        return Ok(None);
    };
    let value = serde_json::from_str(&facet.value_json)
        .map_err(|_| SessionError::Corrupt("Malformed page claim evidence facet".into()))?;
    KeptPageClaims::from_facet(source.clone(), &value)
        .map(Some)
        .map_err(|_| SessionError::Corrupt("Malformed page claim evidence".into()))
}

fn store_kept<G: SessionGraph>(
    graph: &mut G,
    author: &Author,
    kept: &KeptPageClaims,
    value: serde_json::Value,
) -> Result<(), SessionError> {
    if let Some(old) = read_kept(graph, &kept.source)? {
        return if &old == kept {
            Ok(())
        } else {
            Err(SessionError::Validation(
                "Kept source already identifies different claims".into(),
            ))
        };
    }
    let resource = ResourceNode::for_term(kept.source.as_str());
    let mut record = graph
        .resources()
        .resource_record(resource.id())
        .unwrap_or_else(|| PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_owned(),
            facets: Vec::new(),
        });
    record.facets.push(PersistedResourceFacet {
        facet: KEPT_PAGE_CLAIMS_FACET.to_owned(),
        value_json: value.to_string(),
    });
    // The engine's one recorded resource write (engine split D257). A
    // refusal shows in the read-back below.
    let _ = graph.write_resource_record(author, resource.id(), Some(record));
    if read_kept(graph, &kept.source)?.as_ref() != Some(kept) {
        return Err(SessionError::Validation(
            "Kept source claims could not be admitted".into(),
        ));
    }
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use mere::ContextCache;
    use mere::acquire::page_claims::interpret;
    use mere::dataset::View;
    use muniment::{Backend, MemoryBackend};

    use super::*;
    use crate::MereSessions;
    use crate::test_surface::Surface;

    #[test]
    fn kept_source_reopens_at_its_original_cursor_through_a_sealed_journal() {
        pollster::block_on(async {
            let raw = MemoryBackend::new();
            let epoch = sporran::wallet_store::KeyEpochId(uuid::Uuid::from_bytes([72; 16]));
            let sealed = sporran::WalletSealedBackend::new(
                raw.clone(),
                sporran::WalletEpochSealer::from_epoch(
                    crate::PersonaId::new(),
                    epoch,
                    b"fixture secret",
                ),
            );
            let sessions = MereSessions::<_, Surface>::new_with(sealed.clone());
            let author = Author::user();
            let manifest = sessions.mint(author.clone(), None).await.unwrap();
            let mut session = sessions.open_with(manifest.session_id).await.unwrap();
            let source = NamedNode::new_unchecked("urn:fixture:kept-capture");
            let page = "https://example.test/field-notes";
            let block =
                r##"{"@id":"#river","http://example.test/source":"fixture private claim"}"##;
            let claims = interpret(&ContextCache::new(), page, [(5, block), (6, "{")]);
            let before = session.journal().live_cursor();
            assert!(session.kept_page_claims(&source).unwrap().is_none());

            let now = SystemTime::now();
            let kept = session
                .admit_page_claims(author.clone(), &source, &claims, now)
                .await
                .unwrap();
            let cursor = session.journal().live_cursor();
            assert!(cursor > before);
            assert!(
                session
                    .pin(before)
                    .unwrap()
                    .kept_page_claims(&source)
                    .unwrap()
                    .is_none()
            );

            let retry = session
                .admit_page_claims(author.clone(), &source, &claims, now)
                .await
                .unwrap();
            assert_eq!(retry, kept);
            assert_eq!(session.journal().live_cursor(), cursor);
            assert!(
                session
                    .admit_page_claims(author, &source, &PageClaims::default(), now)
                    .await
                    .is_err()
            );
            assert_eq!(session.journal().live_cursor(), cursor);
            drop(session);

            let reopen = GraphSession::<_, Surface>::open_with(sealed, manifest.session_id)
                .await
                .unwrap();
            let reopened = reopen
                .pin(cursor)
                .unwrap()
                .kept_page_claims(&source)
                .unwrap()
                .unwrap();
            assert_eq!(reopened, kept);
            let graph = source.into();
            let index = reopened.dataset();
            let scoped: Vec<_> = index
                .quads_matching(View::Live, None, None, None, Some(&graph))
                .collect();
            assert_eq!(scoped.len(), 1);
            assert_eq!(reopened.claims.blocks[0].order, 5);
            assert_eq!(reopened.claims.diagnostics[0].order, 6);
            assert_eq!(
                reopened.claims.blocks[0].quads[0].graph_name,
                oxrdf::GraphName::DefaultGraph
            );
            for key in raw.list("").await.unwrap() {
                let bytes = raw.get(&key).await.unwrap().unwrap();
                assert!(
                    !bytes
                        .windows(b"fixture private claim".len())
                        .any(|window| window == b"fixture private claim")
                );
            }
        });
    }
}
