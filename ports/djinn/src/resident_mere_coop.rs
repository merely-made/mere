// SPDX-License-Identifier: MPL-2.0
//! Co-op reuses the one resident host of an existing session.
use super::*;
use graphshell::coop::{self, GraphAuthority, GraphReading, Owner, control, network};
use personae::IdentityProvider;
use std::sync::Mutex as SyncMutex;

struct Attached(SharedHost);
impl GraphAuthority for Attached {
    fn read(&self) -> GraphReading {
        run(async {
            let host = self.0.lock().await;
            let session = host.graph_session();
            GraphReading {
                graph: session.graph().clone(),
                revision: Revision(session.journal().live_cursor().0),
                authors: session
                    .changes()
                    .iter()
                    .rev()
                    .take(32)
                    .rev()
                    .map(|c| c.author.clone())
                    .collect(),
            }
        })
    }
    fn transact(
        &self,
        expected: Revision,
        author: Author,
        deltas: Vec<kernel::graph::apply::GraphDelta>,
    ) -> Result<(), String> {
        run(async {
            self.0
                .lock()
                .await
                .transact_coop(expected, author, deltas)
                .await
                .map_err(|e| e.to_string())
        })
    }
}
struct Activity {
    owner: coop::SharedOwner<DirectoryBackend>,
    identity: Arc<graphshell_endpoint::door::custody_identity::CustodyIdentity>,
    contact: network::Card,
    ledger: Arc<std::sync::RwLock<notochord::RevocationLedger>>,
    _server: tokio::task::JoinHandle<()>,
}
// Lifetime follows the resident, not the window; reconnect never opens another writer.
#[derive(Default)]
pub(super) struct Activities(SyncMutex<HashMap<SessionId, Activity>>);
impl Activities {
    fn call(
        &self,
        shared: &Arc<MereShared>,
        request: control::Request,
    ) -> Result<control::Reply, String> {
        let graph = request.graph();
        let mut activities = self.0.lock().map_err(|_| "co-op activities lock")?;
        if let std::collections::hash_map::Entry::Vacant(entry) = activities.entry(graph) {
            let manifest = run(shared.sessions().list())
                .map_err(|e| e.to_string())?
                .into_iter()
                .find(|m| m.session_id == graph)
                .ok_or("choose an existing graph in this mere")?;
            if manifest.trashed.is_some() || manifest.codicil_read_only {
                return Err("choose a live writable graph".into());
            }
            let endpoint = shared
                .custody_endpoint
                .as_deref()
                .ok_or("co-op custody is not composed")?;
            let identity = Arc::new(
                graphshell_endpoint::door::custody_identity::CustodyIdentity::connect_at(
                    endpoint,
                    "graphshell",
                )
                .map_err(|e| e.to_string())?,
            );
            let host = run(shared.host(graph))?;
            let mut owner = run(Owner::attach(
                shared.backend.clone(),
                graph,
                identity.master_public_key().to_bytes(),
                Arc::new(Attached(host)),
            ))?;
            let live_identity = identity.clone();
            let live_shared = Arc::downgrade(shared);
            owner.set_access_guard(Arc::new(move || {
                if live_identity.is_locked() {
                    return false;
                }
                let Some(shared) = live_shared.upgrade() else {
                    return false;
                };
                run(shared.sessions().list()).is_ok_and(|sessions| {
                    sessions.iter().any(|m| {
                        m.session_id == graph && m.trashed.is_none() && !m.codicil_read_only
                    })
                })
            }));
            let owner = Arc::new(SyncMutex::new(owner));
            let transport = run(network::bind(&*identity))?;
            let contact = run(network::card(&*identity, graph, &transport))?;
            let mut host = network::host(owner.clone())?;
            let ledger = host.revocations().clone();
            let server = tokio::spawn(async move {
                loop {
                    match host.accept_one(&*transport, coop::now_ms).await {
                        Ok(Ok(session)) => {
                            tokio::spawn(async move {
                                let _ = session.finished().await;
                            });
                        },
                        Ok(Err(_)) => {},
                        Err(_) => break,
                    }
                }
            });
            entry.insert(Activity {
                owner,
                identity,
                contact,
                ledger,
                _server: server,
            });
        }
        let activity = activities.get(&graph).unwrap();
        if activity.identity.is_locked() {
            return Err("identity locked; unlock before using co-op".into());
        }
        let manifest = run(shared.sessions().list())
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|m| m.session_id == graph)
            .ok_or("graph is no longer available")?;
        if manifest.trashed.is_some() || manifest.codicil_read_only {
            return Err("graph is no longer live and writable".into());
        }
        let mut owner = activity.owner.lock().map_err(|_| "co-op owner lock")?;
        match request {
            control::Request::Read { .. } => {},
            control::Request::Invite { guest, write, .. } => {
                run(owner.invite(
                    &*activity.identity,
                    guest,
                    write,
                    coop::now_ms() + 24 * 60 * 60 * 1000,
                ))?;
            },
            control::Request::Revoke { .. } => {
                run(owner.revoke(&*activity.identity))?;
                if let Some(revoked) = owner.revocation() {
                    activity
                        .ledger
                        .write()
                        .map_err(|_| "revocation lock")?
                        .fold(revoked.check().map_err(|e| format!("{e:?}"))?);
                }
            },
            control::Request::Edit(edit) => {
                run(owner.edit_owner(edit))?;
            },
        }
        Ok(control::Reply {
            data: owner.owner_data(),
            contact: activity.contact.clone(),
            invitation: owner.invitation().cloned(),
        })
    }
}

pub(super) fn register(
    catalog: &mut graphshell_host::endpoint_catalog::ResidentEndpointCatalog,
    shared: Arc<MereShared>,
    grants: AppRouteGrants,
) -> Result<(), graphshell_host::endpoint_catalog::ResidentEndpointCatalogError> {
    let route = format!("coop/{}", shared.domain);
    let route_id = route.clone();
    catalog.register(&route, "Graph co-op owner controls", move |context| {
        if context.application() != Some("graphshell") {
            return Err("co-op controls belong to the owner Graphshell application".into());
        }
        let shared = shared.clone();
        let grants = grants.clone();
        let route_id = route_id.clone();
        let domain = shared.domain.clone();
        let authorize = Arc::new(move || {
            let app = AppId::new("graphshell");
            for route in [mere_route_id(&domain), route_id.clone()] {
                grants
                    .current()
                    .admit(&AppRequest {
                        app: app.clone(),
                        route: AppRouteId::new(route).map_err(|e| e.to_string())?,
                    })
                    .map_err(|_| "owner application access ended".to_string())?;
            }
            Ok(())
        });
        Ok(control::Endpoint::new(
            context.session().clone(),
            authorize,
            Arc::new(move |request| shared.coop.call(&shared, request)),
        ))
    })
}
