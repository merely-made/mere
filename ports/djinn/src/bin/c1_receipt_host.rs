// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit public synthetic resident for the Cleromancy C1 headed lane.
//! This executable never selects or opens an installed user identity.

use castellan::{
    authority::PersonaeHost,
    custody::{
        IdentityStorage, IdentityVault, PassphraseEncryptedStorage, Profile, SealedProfileStorage,
    },
};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, ContentHash, IntentInvocation, IntentResult,
    PortableCardV1, PresentationCodec, ProjectionRequest, ProjectionSnapshot,
};
use djinn::{
    embedded_reservoir::reservoir_applications, keeper::Keeper, resident_mere::MereRoutes,
    resident_reservoir::ResidentReservoir,
};
use graphshell::session_item::{self, MERE_ARCHIVE, MERE_GRAPH, MERE_SESSIONS};
use graphshell::{
    identity::VaultProtectionView,
    native::{
        app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId},
        app_broker::{AppEndpointCatalog, serve_app_broker},
        app_client::AppBrokerClient,
        endpoint_catalog::ResidentEndpointCatalog,
        tasks::ResidentTasks,
    },
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

const MARKER: &[u8] = b"cleromancy-c1 public synthetic fixture/v1\n";

const PUBLIC_PASSPHRASE: &[u8] = b"C1 public synthetic identity only";
const PUBLIC_SEED: [u8; 32] = [0xc1; 32];

fn fixture_persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0xc1_2026_1013))
}

/// Explicit fixture provisioning, separate from owner startup and read-only
/// inspection. Uses the normal portable vault and typed wallet bootstrap so
/// ordinary Cleromancy can unlock the same public identity and own the same
/// disk after the synthetic helper has stopped. No reservoir is opened and
/// no graph, session, view or archive is fabricated here.
fn provision_ordinary(root: &std::path::Path) -> Result<serde_json::Value, String> {
    use castellan::custody::wallet::{
        ensure_wallet_state, load_identity_seed_read_only, save_identity_seed,
    };
    let persona = fixture_persona();
    let shared = root.join("shared");
    let vault = root.join("ordinary-vault");
    let known = pandect::wallet_store::list_personas(&shared).map_err(|e| e.to_string())?;
    if known.iter().any(|id| *id != persona) {
        return Err("public fixture provisioning refuses another persona wallet".into());
    }
    let previous_seed = load_identity_seed_read_only(&shared).map_err(|e| e.to_string())?;
    if previous_seed.is_some_and(|seed| seed != PUBLIC_SEED) {
        return Err("public fixture provisioning refuses another identity seed".into());
    }
    if previous_seed.is_none() && pandect::wallet_store::identity_seed_path(&shared).exists() {
        return Err("public fixture provisioning refuses an unreadable existing seed".into());
    }
    let chain = pandect::wallet_store::derive_persona_chain_root(PUBLIC_SEED, persona)
        .map_err(|e| e.to_string())?;
    if pandect::wallet_store::load_persona_wallet(&shared, persona)
        .map_err(|e| e.to_string())?
        .is_some_and(|wallet| wallet.chain_root != chain)
    {
        return Err("public fixture provisioning refuses another persona chain root".into());
    }
    std::fs::create_dir_all(&vault).map_err(|e| e.to_string())?;
    let storage = PassphraseEncryptedStorage::open(
        vault.join(castellan::custody::bootstrap::PASSPHRASE_VAULT_FILE),
        PUBLIC_PASSPHRASE,
    )
    .map_err(|e| e.to_string())?;
    let profile_id = ProfileId("c1-public-synthetic".into());
    let profiles = storage.list_profiles().map_err(|e| e.to_string())?;
    if profiles.iter().any(|summary| summary.id != profile_id) {
        return Err("public fixture provisioning refuses another ordinary-vault profile".into());
    }
    if profiles.is_empty() {
        storage
            .save_profile(&Profile::new(
                profile_id.clone(),
                "C1 public synthetic fixture",
                Ed25519Keypair::from_seed(PUBLIC_SEED),
            ))
            .map_err(|e| e.to_string())?;
    } else if storage
        .load_profile(&profile_id)
        .map_err(|e| e.to_string())?
        .master
        .to_seed()
        != PUBLIC_SEED
    {
        return Err("public fixture provisioning refuses another ordinary-vault key".into());
    }
    if previous_seed.is_none() {
        save_identity_seed(&shared, PUBLIC_SEED).map_err(|e| e.to_string())?;
    }
    if ensure_wallet_state(&shared, persona, "C1 public synthetic device")
        .map_err(|e| e.to_string())?
        != PUBLIC_SEED
    {
        return Err("typed fixture wallet bootstrap returned another identity seed".into());
    }
    if pandect::wallet_store::resolve_persona(&shared, None).map_err(|e| e.to_string())? != persona
    {
        return Err("fixture wallet did not resolve the exact public persona".into());
    }
    Ok(serde_json::json!({
        "schema":"cleromancy.c1-ordinary-provisioning/v1","persona":persona.as_uuid().to_string(),
        "profile":profile_id.0,"shared_root":shared,"vault_dir":vault,
        "public_fixture":true,"standard_passphrase_vault":true,"typed_wallet_bootstrap":true,
        "graph_seeded":false,"reservoir_opened":false,"owner_startup_unchanged":true,
        "public_passphrase":"C1 public synthetic identity only",
    }))
}

#[derive(Clone, Copy)]
enum InspectApplication {
    Graphshell,
    Cleromancy,
}

impl InspectApplication {
    fn id(self) -> &'static str {
        match self {
            Self::Graphshell => "graphshell",
            Self::Cleromancy => "cleromancy",
        }
    }

    fn view(self) -> &'static str {
        match self {
            Self::Graphshell => "graphshell.mere",
            Self::Cleromancy => "reading-workspace",
        }
    }
}

/// Inspect an established, running public fixture, reading no reservoir files.
/// Default inspection sends no intents. A requested session may use only its
/// advertised connection-local Attach, recorded in JSON; inspection never
/// ensures, mints, forks, trashes, edits a graph, or saves a view. Explicit
/// route opening retains the owner's normal startup behavior: if no live
/// session exists, the owner may mint one. Qualification uses this mode only
/// after the actual product has established live sessions, and checks that
/// durable bytes/counts do not change across inspection.
async fn inspect(
    endpoint: &str,
    application: InspectApplication,
    target_session: Option<Uuid>,
) -> Result<serde_json::Value, String> {
    let mut client = AppBrokerClient::open_route_at(
        endpoint,
        AppId::new(application.id()),
        AppRouteId::new("mere/divination").map_err(|e| e.to_string())?,
    )
    .await
    .map_err(|e| e.to_string())?;
    let result = inspect_connected(&mut client, application, target_session).await;
    let close = client.close().await.map_err(|e| e.to_string());
    let value = result?;
    close?;
    Ok(value)
}

async fn read_projection(
    client: &mut AppBrokerClient,
    request: ProjectionRequest,
    cards: &mut HashMap<ContentHash, PortableCardV1>,
) -> Result<ProjectionSnapshot, String> {
    let expected = request.session.clone();
    let snapshot = client.snapshot(request).await.map_err(|e| e.to_string())?;
    if snapshot.session != expected {
        return Err("inspect snapshot has another projection namespace".into());
    }
    for binding in &snapshot.presentation.bindings {
        let offer = snapshot
            .presentation
            .offers
            .get(&binding.key)
            .and_then(|offers| offers.first())
            .ok_or("inspect projection has no bound card offer")?;
        if offer.codec != PresentationCodec::PortableCardV1 {
            return Err("inspect projection has another presentation role".into());
        }
        // AppBrokerClient verifies response namespace, requested hash, and
        // the actual byte address before these bytes enter the card cache.
        let bytes = client
            .resource(snapshot.session.clone(), offer.resource)
            .await
            .map_err(|e| e.to_string())?;
        if ContentHash::of(&bytes) != offer.resource {
            return Err("inspect card has another content address".into());
        }
        let card: PortableCardV1 = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        cards.insert(offer.resource, card);
    }
    Ok(snapshot)
}

fn source_cards<'a>(
    snapshot: &'a ProjectionSnapshot,
    cards: &'a HashMap<ContentHash, PortableCardV1>,
    adapter: &str,
) -> Result<Vec<(String, &'a PortableCardV1)>, String> {
    let mut result = Vec::new();
    for (instance, item) in snapshot.scene.active_items_in_order() {
        let source = snapshot.scene.tables.sources[item.source.0 as usize]
            .as_ref()
            .ok_or("inspect item has no source")?;
        if source.adapter != adapter {
            continue;
        }
        let offer = snapshot
            .presentation
            .offers_for(instance)
            .and_then(|offers| offers.first())
            .ok_or("inspect item has no portable card")?;
        let card = cards
            .get(&offer.resource)
            .ok_or("inspect card was not fetched")?;
        result.push((source.id.clone(), card));
    }
    Ok(result)
}

async fn inspect_connected(
    client: &mut AppBrokerClient,
    application: InspectApplication,
    target_session: Option<Uuid>,
) -> Result<serde_json::Value, String> {
    let descriptor = client
        .open_session()
        .await
        .map_err(|e| e.to_string())?
        .descriptor;
    let find = |namespace: &str| -> Result<ProjectionRequest, String> {
        let mut found = descriptor
            .projections
            .iter()
            .filter(|offer| offer.request.session.0 == namespace);
        let request = found
            .next()
            .ok_or_else(|| format!("inspect missing {namespace} projection"))?
            .request
            .clone();
        if found.next().is_some() {
            return Err(format!("inspect duplicate {namespace} projection"));
        }
        Ok(request)
    };
    let mut cards = HashMap::new();
    let mut sessions = read_projection(client, find(MERE_SESSIONS)?, &mut cards).await?;
    let mut graph = read_projection(client, find(MERE_GRAPH)?, &mut cards).await?;
    let mut attachment_action = None;
    if let Some(target) = target_session {
        let current = {
            let attached = source_cards(&graph, &cards, session_item::SESSION_ITEM_SOURCE)?;
            if attached.len() != 1 {
                return Err("inspect graph must disclose one attached session card".into());
            }
            Uuid::parse_str(&attached[0].0).map_err(|e| e.to_string())?
        };
        // Find the target by its stable source UUID in this exact admitted
        // session catalog. Unknown, trashed or no-longer-offered targets do
        // not buy an unchecked attachment path.
        let (instance, _) = sessions
            .scene
            .active_items_in_order()
            .into_iter()
            .find(|(_, item)| {
                sessions.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .is_some_and(|source| {
                        source.adapter == session_item::SESSIONS_SOURCE
                            && Uuid::parse_str(&source.id).ok() == Some(target)
                    })
            })
            .ok_or("inspect target session is absent from the admitted catalog")?;
        if target != current {
            let advertised = sessions
                .presentation
                .offers_for(instance)
                .is_some_and(|offers| {
                    offers.iter().any(|offer| {
                        offer.semantics.actions.iter().any(|action| {
                            action.intent.0 == session_item::ATTACH_SESSION_INTENT
                                && action.payload_schema == session_item::SESSIONS_SCHEMA
                        })
                    })
                });
            if !advertised {
                return Err("inspect target does not advertise connection-local attachment".into());
            }
            let invocation = IntentInvocation {
                session: sessions.session.clone(),
                target: instance,
                observed_epoch: sessions.scene.epoch,
                observed_revision: sessions.scene.revision,
                intent: session_item::ATTACH_SESSION_INTENT.into(),
                payload: serde_json::to_vec(&session_item::SessionsActionV1::on(
                    pandect::SessionId::from_uuid(target),
                ))
                .map_err(|e| e.to_string())?,
            };
            match client
                .request_body(CarrierRequestBody::Intent(invocation))
                .await
                .map_err(|e| e.to_string())?
            {
                CarrierResponseBody::Intent(IntentResult::Accepted) => {},
                other => {
                    return Err(format!(
                        "inspect attachment was not acknowledged: {other:?}"
                    ));
                },
            }
            attachment_action = Some(serde_json::json!({
                "intent":session_item::ATTACH_SESSION_INTENT,"from_session":current.to_string(),
                "to_session":target.to_string(),"scope":"connection-local","acknowledged":true
            }));
            cards.clear();
            sessions = read_projection(client, find(MERE_SESSIONS)?, &mut cards).await?;
            graph = read_projection(client, find(MERE_GRAPH)?, &mut cards).await?;
        }
    }
    let archive = read_projection(client, find(MERE_ARCHIVE)?, &mut cards).await?;
    let session_cards = source_cards(&graph, &cards, session_item::SESSION_ITEM_SOURCE)?;
    if session_cards.len() != 1 {
        return Err("inspect graph must disclose one attached session card".into());
    }
    let (attached_id, session_card) = &session_cards[0];
    let attached = Uuid::parse_str(attached_id).map_err(|e| e.to_string())?;
    if target_session.is_some_and(|target| target != attached) {
        return Err("inspect did not attach the requested exact session UUID".into());
    }
    let media = *session_card
        .media
        .first()
        .ok_or("inspect session card has no material media")?;
    let bytes = client
        .resource(graph.session.clone(), media)
        .await
        .map_err(|e| e.to_string())?;
    if ContentHash::of(&bytes) != media {
        return Err("inspect material has another content address".into());
    }
    let material: session_item::SessionGraphV1 =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if material.schema != session_item::SESSION_GRAPH_SCHEMA
        || material.application != application.id()
        || *material.session.as_uuid() != attached
        || material.epoch != graph.scene.epoch
        || material.revision != graph.scene.revision
    {
        return Err("inspect material has another schema/application/session/position".into());
    }
    // Replay validation is a read-only check, using the same product-owned
    // validator as the resident; it never installs or persists a graph.
    cleromancy_domain::host::validate_divination_graph(&material.clone().into_graph()?)
        .map_err(|e| e.to_string())?;
    let ui_sessions = source_cards(&sessions, &cards, session_item::SESSIONS_SOURCE)?
        .into_iter()
        .filter_map(|(id, card)| {
            Uuid::parse_str(&id).ok().map(|_| {
                serde_json::json!({
                    "id":id,"name":card.title,"attached":id==*attached_id,"badges":card.badges,
                })
            })
        })
        .collect::<Vec<_>>();
    if !ui_sessions
        .iter()
        .any(|session| session["id"].as_str() == Some(attached_id.as_str()))
    {
        return Err("inspect attached session is absent from the UI catalog".into());
    }
    let archive_entries = source_cards(&archive, &cards, session_item::ARCHIVE_SOURCE)?
        .into_iter()
        .filter(|(id, _)| id != "archive")
        .map(|(id, card)| serde_json::json!({"id":id,"name":card.title,"badges":card.badges}))
        .collect::<Vec<_>>();
    let records = material
        .graph
        .nodes
        .iter()
        .map(|node| {
            serde_json::json!({
                "id":node.node_id,"title":node.title,"address":node.address.as_url_str()
            })
        })
        .collect::<Vec<_>>();
    let mut records_by_facet: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, facets) in material.facets.iter() {
        for (facet, _) in facets.iter() {
            records_by_facet
                .entry(facet.as_str().into())
                .or_default()
                .push(id.to_string());
        }
    }
    Ok(serde_json::json!({
        "schema":"cleromancy.c1-resident-inspection/v1","application":application.id(),"route":"mere/divination",
        "sessions":ui_sessions,"session_ids":ui_sessions.iter().map(|session| session["id"].clone()).collect::<Vec<_>>(),
        "session_count":ui_sessions.len(),"attached_id":attached_id,
        "read_only":material.read_only,"own_views":material.views,"own_view_name":application.view(),"own_view":material.views.get(application.view()),
        "epoch":material.epoch.0,"revision":material.revision.0,"material_hash":media,
        "domain_records":records,"domain_record_ids":records.iter().map(|record| record["id"].clone()).collect::<Vec<_>>(),
        "domain_record_count":records.len(),"domain_records_by_facet":records_by_facet,
        "domain_edge_count":material.graph.edges.len(),"archive":archive_entries,
        "archive_ids":archive_entries.iter().map(|entry| entry["id"].clone()).collect::<Vec<_>>(),"archive_count":archive_entries.len(),
        "requested_session_id":target_session.map(|id| id.to_string()),"attachment_action":attachment_action,
        "inspection_sends_no_intents":attachment_action.is_none(),"inspection_intent_count":usize::from(attachment_action.is_some()),
        "requires_established_live_sessions":true
    }))
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(
        matches!(args.len(), 3 | 4),
        "usage: c1-receipt-host <endpoint> <isolated-root> <new|reopen|inspect|inspect-cleromancy|provision-ordinary> [target-session-uuid]"
    );
    let endpoint = args[0].clone();
    let root = PathBuf::from(&args[1]);
    // This is deliberately a fixed mode enum, not an arbitrary application
    // argument. Refuse unknown modes before any connection or file mutation.
    let inspection = match args[2].as_str() {
        "inspect" => Some(InspectApplication::Graphshell),
        "inspect-cleromancy" => Some(InspectApplication::Cleromancy),
        "new" | "reopen" | "provision-ordinary" => None,
        _ => panic!("mode must be new, reopen, inspect, inspect-cleromancy or provision-ordinary"),
    };
    let target_session = match (inspection, args.get(3)) {
        (Some(_), Some(value)) => {
            Some(Uuid::parse_str(value).expect("inspect target must be a valid session UUID"))
        },
        (_, None) => None,
        (None, Some(_)) => panic!("target session is supported only by the fixed inspect modes"),
    };
    if args[2] == "provision-ordinary" {
        assert_eq!(
            std::fs::read(root.join(".c1-public-synthetic")).unwrap(),
            MARKER,
            "only an explicitly marked public fixture may be provisioned"
        );
        let value = provision_ordinary(&root).expect("ordinary public fixture provisioning failed");
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
        return;
    }
    if let Some(application) = inspection {
        assert_eq!(
            std::fs::read(root.join(".c1-public-synthetic")).unwrap(),
            MARKER,
            "only an explicitly marked public fixture may be inspected"
        );
        let value = tokio::time::timeout(
            Duration::from_secs(15),
            inspect(&endpoint, application, target_session),
        )
        .await
        .expect("fixture inspection timed out")
        .expect("fixture inspection failed");
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
        return;
    }
    match args[2].as_str() {
        "new" => {
            assert!(!root.exists(), "new synthetic root must not exist");
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join(".c1-public-synthetic"), MARKER).unwrap();
        },
        "reopen" => assert_eq!(
            std::fs::read(root.join(".c1-public-synthetic")).unwrap(),
            MARKER,
            "only an explicitly marked public fixture may reopen"
        ),
        _ => unreachable!("inspection modes returned before owner startup"),
    }
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0xc1; 32]);
    let profile_id = ProfileId("c1-public-synthetic".into());
    if args[2] == "new" {
        storage
            .enroll_passphrase(b"C1 public synthetic identity only")
            .unwrap();
        storage
            .save_profile(&Profile::new(
                profile_id.clone(),
                "C1 public synthetic fixture",
                Ed25519Keypair::from_seed([0xc1; 32]),
            ))
            .unwrap();
    }
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &profile_id).unwrap(),
        None,
        VaultProtectionView::Passphrase,
    ));
    let persona = fixture_persona();
    let reservoir = ResidentReservoir::open(&root.join("shared"), Some(persona))
        .await
        .unwrap();
    let apps = reservoir_applications();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new(
        apps.iter()
            .cloned()
            .map(|app| (app, ResidentReservoir::route())),
    ));
    djinn::custody::grant(&grants).unwrap();
    let routes = MereRoutes::new(
        root.join("shared"),
        persona,
        catalog.clone(),
        grants.clone(),
        apps,
    )
    .with_divination_authority();
    routes.load_access(&reservoir).await.unwrap();
    for mere in reservoir.meres().await {
        routes.serve(&mere).await.unwrap();
    }
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes.clone())))
        .await
        .unwrap();
    // Clients ensure through the same owner route after attachment. No
    // product graph or session is fabricated by this startup fixture.
    let tasks = ResidentTasks::new();
    let (scope, door, address) = (tasks.clone(), catalog.clone(), endpoint.clone());
    let mut server = tokio::spawn(async move {
        scope
            .scope(serve_app_broker(
                &address,
                Arc::new(Keeper::new(host)),
                grants,
                600_000,
                None,
                door,
            ))
            .await
    });
    let mut ready = false;
    for _ in 0..200 {
        if graphshell::native::local_endpoint::connect_local(&endpoint)
            .await
            .is_ok()
        {
            ready = true;
            break;
        }
        assert!(!server.is_finished(), "synthetic resident failed to bind");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(ready, "synthetic resident did not bind");
    std::fs::write(root.join("ready.json"), serde_json::to_vec(&serde_json::json!({"schema":"cleromancy.c1-resident-fixture/v1","pid":std::process::id(),"persona":persona.as_uuid().to_string(),"endpoint":endpoint,"mode":args[2],"applications":["cleromancy","graphshell","turnstone","knot-editor"]})).unwrap()).unwrap();
    println!(
        "C1_READY pid={} persona={} mode={}",
        std::process::id(),
        persona.as_uuid(),
        args[2]
    );
    let stop = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    });
    tokio::select! { result = &mut server => { result.unwrap().unwrap(); }, _ = stop => { server.abort(); let _ = server.await; } }
    tasks.cancel_and_join().await;
    catalog
        .update(|catalog| *catalog = ResidentEndpointCatalog::new())
        .await;
    drop(routes);
    drop(reservoir);
    // Prove the exclusive reservoir handle was released before reporting.
    ResidentReservoir::open(&root.join("shared"), Some(persona))
        .await
        .unwrap();
    println!("C1_RELEASED pid={}", std::process::id());
}
