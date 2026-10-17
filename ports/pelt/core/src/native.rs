// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The stack's held-content engines, shared by embedding applications.
use crate::PeltRegistries;
pub use genet_host_api::ResourceFetcher;
use inker::{
    EngineDocument, EngineProfileBinding, SessionEngine, SessionError, SessionRegistry,
    SessionSpawnRequest, SurfaceEngineRegistry,
    routing::{EngineRoutePolicy, EngineRouteRule, SurfaceContractMode},
};
use mere_document_lanes::{ReaderSessionEngine, SmolwebTheme};
use netrender::Scene;

/// Subresources supplied by the content owner. Embedded data URLs use Genet's
/// existing decoder; other addresses resolve only from this held inventory.
#[derive(Clone, Default)]
pub struct HeldResources(pub std::collections::BTreeMap<String, Vec<u8>>);
impl genet_host_api::ResourceFetcher for HeldResources {
    fn fetch(&self, address: &str) -> Option<Vec<u8>> {
        if address.starts_with("data:") {
            genet_host_api::ResourceFetcher::fetch(&genet_documents::LocalFetcher, address)
        } else {
            self.0.get(address).cloned()
        }
    }

    fn fetch_request(
        &self,
        request: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        if request.url.starts_with("data:") {
            genet_documents::LocalFetcher.fetch_request(request)
        } else {
            // The held inventory records bytes, not their transport status or
            // response headers, even when a key happens to look like HTTPS.
            None
        }
    }
}

#[cfg(test)]
mod held_resource_contract {
    use super::*;

    #[test]
    fn held_https_bytes_do_not_claim_http_response_metadata() {
        let address = "https://example.test/held";
        let mut held = HeldResources::default();
        held.0.insert(address.into(), b"cached".to_vec());
        assert_eq!(held.fetch(address), Some(b"cached".to_vec()));
        assert!(
            held.fetch_request(&genet_host_api::ResourceRequest::plain_get(address))
                .is_none()
        );
    }
}

pub const DOCUMENT_ENGINE_ID: &str = "mere.document";

/// A normalized Inker document. Acquisition, interpretation and custody belong
/// to the supplied content handler; this engine operates its retained view.
pub struct DocumentSessionEngine;
impl SessionEngine<Scene> for DocumentSessionEngine {
    fn engine_id(&self) -> &str {
        DOCUMENT_ENGINE_ID
    }
    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn inker::DocumentSession<Scene>>, SessionError> {
        let document: EngineDocument = serde_document(request)?;
        let doc = mere_document_lanes::SmolwebDocument::from_document_with_theme(
            document,
            SmolwebTheme::System,
        );
        Ok(Box::new(mere_document_lanes::SmolwebDocumentSession::new(
            doc,
            request.viewport,
        )))
    }
}
fn serde_document(request: &SessionSpawnRequest) -> Result<EngineDocument, SessionError> {
    // The shared clip engine already validates this address/source binding.
    // The normalized-document handler uses the same serialization contract.
    let body = request.body.as_deref().ok_or_else(|| {
        SessionError::Unsupported("A retained document requires held content".into())
    })?;
    let document: EngineDocument =
        serde_json::from_str(body).map_err(|e| SessionError::SpawnFailed(e.to_string()))?;
    if document.address != request.address
        || document.provenance.source_kind.as_deref() != Some(DOCUMENT_ENGINE_ID)
    {
        return Err(SessionError::SpawnFailed(
            "Retained document presentation identity differs".into(),
        ));
    }
    Ok(document)
}

/// Native engines without acquiring a window, GPU, remote transport or store.
pub fn sessions() -> SessionRegistry<Scene> {
    sessions_with_resources(HeldResources::default())
}
pub fn sessions_with_resources(resources: HeldResources) -> SessionRegistry<Scene> {
    sessions_with_fetcher(resources)
}
pub fn sessions_with_fetcher<Fetch: ResourceFetcher + Send + Sync + 'static>(
    resources: Fetch,
) -> SessionRegistry<Scene> {
    sessions_with_fetcher_and_fonts(resources, Vec::new())
}
/// Explicit host fonts are applied to each session, without ambient acquisition.
pub fn sessions_with_fetcher_and_fonts<Fetch: ResourceFetcher + Send + Sync + 'static>(
    resources: Fetch,
    fonts: Vec<Vec<u8>>,
) -> SessionRegistry<Scene> {
    sessions_with_presentation(resources, fonts, None)
}
/// A reading host may select its own display face. Canonical source bytes stay
/// with the owner; this is an explicit presentation choice, not acquisition.
pub fn sessions_with_presentation<Fetch: ResourceFetcher + Send + Sync + 'static>(
    resources: Fetch,
    fonts: Vec<Vec<u8>>,
    family: Option<String>,
) -> SessionRegistry<Scene> {
    let sheets = family
        .as_ref()
        .map(|family| {
            format!(
                "* {{font-family:{} !important;}}",
                serde_json::to_string(family).unwrap()
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    let mut sessions = SessionRegistry::new();
    let engines: Vec<Box<dyn SessionEngine<Scene>>> = vec![
        Box::new(genet_documents::LiverySessionEngine::with_author_css(
            resources, sheets,
        )),
        Box::new(ReaderSessionEngine::new(SmolwebTheme::System)),
        Box::new(DocumentSessionEngine),
        Box::new(mere_document_lanes::retained_clip::ClipSessionEngine),
    ];
    for inner in engines {
        sessions.register(Box::new(FontEngine {
            inner,
            fonts: fonts.clone(),
            family: family.clone(),
        }));
    }

    sessions
}

pub fn route_policy() -> EngineRoutePolicy {
    let mut policy = EngineRoutePolicy::default();
    for rule in &mut policy.rules {
        if rule.engine_id == inker::routing::ENGINE_GENET_WEB {
            rule.engine_id = inker::routing::ENGINE_GENET_LIVERY.into();
        }
    }
    policy.fallback.engine_id = inker::routing::ENGINE_GENET_LIVERY.into();
    policy.rules.insert(
        0,
        EngineRouteRule::content_type(
            ["application/vnd.mere.clip+json"],
            mere_document_lanes::retained_clip::ENGINE_ID,
            SurfaceContractMode::CompositedTexture,
        ),
    );
    policy.rules.insert(
        0,
        EngineRouteRule::content_type(
            ["application/vnd.mere.document+json"],
            DOCUMENT_ENGINE_ID,
            SurfaceContractMode::CompositedTexture,
        ),
    );
    policy
}

/// Applications can extend the session registry and install their external
/// producers before using the ordinary Pelt registry constructor instead.
pub fn registries(
    workspace_id: impl Into<String>,
    policy: EngineRoutePolicy,
    profile: EngineProfileBinding,
) -> PeltRegistries<Scene> {
    PeltRegistries::new(
        sessions(),
        SurfaceEngineRegistry::new(),
        policy,
        workspace_id,
        inker::routing::ENGINE_GENET_LIVERY,
        profile,
    )
    .with_host_history()
    .with_host_loading()
}

struct FontEngine {
    inner: Box<dyn SessionEngine<Scene>>,
    fonts: Vec<Vec<u8>>,
    family: Option<String>,
}
impl SessionEngine<Scene> for FontEngine {
    fn engine_id(&self) -> &str {
        self.inner.engine_id()
    }
    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn inker::DocumentSession<Scene>>, SessionError> {
        let mut session = self.inner.spawn(request)?;
        if self.fonts.is_empty() {
            return Ok(session);
        }
        if let Some(doc) = session
            .as_any()
            .downcast_mut::<mere_document_lanes::SmolwebDocumentSession>()
        {
            doc.document_mut().set_fonts(self.fonts.clone());
            if let Some(family) = &self.family {
                doc.document_mut().set_font_family(family);
            }
        } else if let Some(doc) = session
            .as_any()
            .downcast_mut::<mere_document_lanes::ReaderDocumentSession>()
        {
            doc.set_fonts(self.fonts.clone());
            if let Some(family) = &self.family {
                doc.set_font_family(family);
            }
        } else if let Some(doc) = session
            .as_any()
            .downcast_mut::<genet_documents::LiveryDocumentSession>()
        {
            use genet_document_resources::{ResolvedResource, ResourceKind};
            let mut held = doc.resource_set().clone();
            for (index, bytes) in self.fonts.iter().enumerate() {
                let address = format!("mere:pelt/host-font/{index}");
                held.resources.push(ResolvedResource {
                    kind: ResourceKind::Font,
                    authored_url: address.clone(),
                    resolved_url: address.clone(),
                    final_url: address,
                    content_type: Some("font/ttf".into()),
                    bytes: bytes.clone(),
                });
            }
            doc.replace_resource_bytes(held)
                .map_err(SessionError::SpawnFailed)?;
        }
        Ok(session)
    }
}
