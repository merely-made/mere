// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn a_request_reads_back_as_itself_and_names_its_route() {
    for request in [
        RemoteMereRequest::explicit("divination", "turnstone"),
        RemoteMereRequest::ambient("divination", "turnstone"),
    ] {
        let read = RemoteMereRequest::parse(REMOTE_DOMAIN, &request.path(), request.action());
        assert_eq!(read.as_ref(), Some(&request));
    }
    assert_eq!(
        RemoteMereRequest::explicit("divination", "turnstone").route_id(),
        "mere/divination"
    );
    assert_eq!(
        RemoteMereRequest::ambient("divination", "turnstone").route_id(),
        "ambient/mere/divination"
    );
}

#[test]
fn a_route_does_not_take_the_other_routes_action_or_a_loose_spelling() {
    let explicit = RemoteMereRequest::explicit("divination", "turnstone");
    assert!(RemoteMereRequest::parse(REMOTE_DOMAIN, &explicit.path(), "ambient").is_none());
    let ambient = RemoteMereRequest::ambient("divination", "turnstone");
    assert!(RemoteMereRequest::parse(REMOTE_DOMAIN, &ambient.path(), "connect").is_none());
    for path in [
        "/services/projection",
        "/services/projection/mere/divination",
        "/services/projection/mere/divination/app",
        "/services/projection/mere/divination/app/",
        "/services/projection/mere//app/turnstone",
        "/services/projection/mere/divination/app/turnstone/extra",
        "/services/projection/mere/divination/apps/turnstone",
        "/services/projection/merely/divination/app/turnstone",
    ] {
        assert!(
            RemoteMereRequest::parse(REMOTE_DOMAIN, path, "connect").is_none(),
            "{path} is not a remote mere request"
        );
    }
    assert!(RemoteMereRequest::parse("mere.other", &explicit.path(), "connect").is_none());
}

#[test]
fn only_a_grant_at_or_under_the_application_names_it() {
    let request = RemoteMereRequest::explicit("divination", "turnstone");
    assert!(request.named_by("/services/projection/mere/divination/app/turnstone"));
    assert!(request.named_by("/services/projection/mere/divination/app/turnstone/x"));
    assert!(!request.named_by("/services/projection/mere/divination"));
    assert!(!request.named_by("/services/projection/mere"));
    assert!(!request.named_by("/services/projection/mere/divination/app/turnstone-x"));
    assert!(!request.named_by("/services/projection/mere/divination/app/knot"));
}
