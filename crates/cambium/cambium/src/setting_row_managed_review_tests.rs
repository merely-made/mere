/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

#[test]
fn reviewed_draft_equal_to_original_baseline_survives_external_observation() {
    let session = SettingRowSession::new(text_spec("baseline"));
    session.record_edit(SettingValue::Text("mine".into()));
    assert_eq!(
        session.observe(text_spec("external")),
        SettingRowObservation::PreservedDraft
    );
    session.fail(SettingRowFailureKind::Conflict, "revision changed");
    assert!(session.review_mine());
    session.record_edit(SettingValue::Text("baseline".into()));

    assert_eq!(
        session.observe(text_spec("external")),
        SettingRowObservation::PreservedDraft
    );
    let snapshot = session.snapshot();
    assert!(snapshot.dirty);
    assert_eq!(snapshot.phase, SettingRowPhase::ReviewingMine);
    assert_eq!(snapshot.draft_value, SettingValue::Text("baseline".into()));
}
