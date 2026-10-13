// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Host commands around one independent reading. No storage or transport here.
use super::*;

#[derive(Clone, Debug)]
pub enum Intent {
    Refresh,
    InspectResource(ResourceRef),
    InspectActivity(ActivityRef),
    Resource(ResourceAction),
    Activity(ActivityAction),
    ForkCollection,
}

/// A concrete application implements this beside its existing domain owners.
/// `dispatch` must recheck keys, authority, quotas and revision at the write/read.
pub trait Owner {
    fn read(&mut self) -> Result<Snapshot, String>;
    fn dispatch(&mut self, request: &Request) -> Outcome;
}

#[derive(Debug)]
pub struct Controller {
    pub pane: Pane,
    pub notice: Option<String>,
}
impl Controller {
    pub fn new(snapshot: Snapshot) -> Result<Self, Refusal> {
        Ok(Self {
            pane: Pane::new(snapshot)?,
            notice: None,
        })
    }
    pub fn refresh(&mut self, snapshot: Snapshot) -> Result<(), Refusal> {
        self.pane.refresh(snapshot)?;
        self.notice = None;
        Ok(())
    }
    /// Async hosts keep the returned non-serializable binding locally while
    /// sending only the addressed intention to their actual domain owner.
    pub fn begin(&mut self, intent: Intent) -> Option<Request> {
        self.notice = None;
        let prepared = match intent {
            Intent::Refresh => Ok(Some(self.pane.prepare_refresh())),
            Intent::InspectResource(reference) => {
                self.pane.select_resource(reference).map(|_| None)
            },
            Intent::InspectActivity(reference) => {
                self.pane.select_activity(&reference).map(|_| None)
            },
            Intent::Resource(action) => self.pane.prepare_resource(action).map(Some),
            Intent::Activity(action) => self.pane.prepare_activity(action).map(Some),
            Intent::ForkCollection => self.pane.prepare_collection_fork().map(Some),
        };
        match prepared {
            Ok(request) => request,
            Err(error) => {
                self.notice = Some(error.to_string());
                None
            },
        }
    }
    pub fn perform(&mut self, owner: &mut impl Owner, intent: Intent) {
        let Some(request) = self.begin(intent) else {
            return;
        };
        let outcome = match owner.read() {
            Ok(current) => match validate_request(&request, &current) {
                Ok(()) if matches!(request.target, Target::Refresh { .. }) => {
                    if let Err(error) = self.pane.complete_refresh(&request, current) {
                        self.notice = Some(error.to_string());
                    }
                    return;
                },
                Ok(()) => owner.dispatch(&request),
                Err(error) => Outcome::Refused {
                    reason: error.to_string(),
                },
            },
            Err(reason) => Outcome::Refused { reason },
        };
        self.pane.complete(&request, outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> Snapshot {
        Snapshot {
            moot: [1; 32],
            reader: [2; 32],
            collection: None,
            activities: vec![],
            resources: vec![ResourceRead {
                shared_by: None,
                reference: ResourceRef {
                    moot: [1; 32],
                    contribution: [3; 32],
                    manifest: [4; 32],
                },
                title: "Note".into(),
                schema: "text/plain".into(),
                content: ContentRead::Resolved,
                retention: RetentionRead::Unknown,
                terms: BTreeMap::new(),
                hosting: HostingRead::Unknown,
                offers: [(ResourceAction::Open, Offer::Offered)].into(),
            }],
        }
    }
    struct Domain {
        current: Snapshot,
        calls: usize,
    }
    impl Owner for Domain {
        fn read(&mut self) -> Result<Snapshot, String> {
            Ok(self.current.clone())
        }
        fn dispatch(&mut self, _: &Request) -> Outcome {
            self.calls += 1;
            Outcome::Refused {
                reason: "storage quota exceeded".into(),
            }
        }
    }
    #[test]
    fn withdrawal_before_dispatch_never_calls_owner_command() {
        let read = snapshot();
        let reference = read.resources[0].reference;
        let mut controller = Controller::new(read.clone()).unwrap();
        controller.begin(Intent::InspectResource(reference));
        let mut owner = Domain {
            current: read,
            calls: 0,
        };
        owner.current.resources.clear();
        controller.perform(&mut owner, Intent::Resource(ResourceAction::Open));
        assert_eq!(owner.calls, 0);
        assert!(matches!(
            controller.pane.outcome(),
            Some(Outcome::Refused { .. })
        ));
    }
    #[test]
    fn independent_readings_and_owner_refusal_preserve_their_reads() {
        let read = snapshot();
        let reference = read.resources[0].reference;
        let mut first = Controller::new(read.clone()).unwrap();
        let mut second = Controller::new(read.clone()).unwrap();
        first.begin(Intent::InspectResource(reference));
        second.begin(Intent::InspectResource(reference));
        let request = first.begin(Intent::Resource(ResourceAction::Open)).unwrap();
        assert!(!second.pane.complete(
            &request,
            Outcome::Accepted {
                owner_receipt: "foreign completion".into()
            }
        ));
        let mut owner = Domain {
            current: read.clone(),
            calls: 0,
        };
        first.perform(&mut owner, Intent::Resource(ResourceAction::Open));
        assert_eq!(owner.calls, 1);
        assert_eq!(first.pane.snapshot(), &read);
        assert_eq!(second.pane.snapshot(), &read);
        assert!(
            matches!(first.pane.outcome(),Some(Outcome::Refused{reason}) if reason=="storage quota exceeded")
        );
        assert!(second.pane.outcome().is_none());
    }
    #[test]
    fn a_changed_collection_version_refuses_before_dispatch() {
        let mut read = snapshot();
        read.collection = Some(CollectionRead {
            fork_destination: Some(ForkDestination { id: [19;32], name: "Reviewed fixture copy".into() }),
            id: [5; 32],
            name: "Notes".into(),
            heads: vec![[6; 32]],
            membership_commitment: [7; 32],
            fork: Some(Offer::Offered),
        });
        let mut controller = Controller::new(read.clone()).unwrap();
        let request = controller.begin(Intent::ForkCollection).unwrap();
        read.collection.as_mut().unwrap().heads = vec![[8; 32]];
        assert_eq!(
            validate_request(&request, &read),
            Err(Refusal::ChangedCollection)
        );
        controller.refresh(read).unwrap();
        assert!(!controller.pane.complete(
            &request,
            Outcome::Accepted {
                owner_receipt: "late".into()
            }
        ));
    }
}
