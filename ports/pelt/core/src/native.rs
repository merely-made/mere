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
    let mut sessions = SessionRegistry::new();
    sessions.register(Box::new(genet_documents::LiverySessionEngine::new(
        resources,
    )));
    sessions.register(Box::new(ReaderSessionEngine::new(SmolwebTheme::System)));
    sessions.register(Box::new(DocumentSessionEngine));
    sessions.register(Box::new(
        mere_document_lanes::retained_clip::ClipSessionEngine,
    ));
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
