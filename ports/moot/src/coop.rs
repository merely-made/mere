// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The coop lifecycle contract: one report shape every consumer can produce
//! from its own snapshot, plus a conformance harness that walks the I3
//! sequence against a consumer-supplied driver.
//!
//! This module owns no state. Membership, history, merge and authorization
//! stay with Gemot, Commons, Turnstone's place worker and the practice
//! fixture's store. A [`LifecycleReport`] is a view: what one consumer
//! believed about itself at one reading of its own clock. It is never peer
//! truth, never a claim about what another peer will decide, and never
//! durable — a consumer rebuilds it from its snapshot whenever asked.
//!
//! The two consumers it abstracts over differ in recorded ways (the
//! 2026-09-16 coop lifecycle parity plan's facts table): the fixture treats
//! one grant as all authority while Turnstone separates membership from
//! delegation, and only Turnstone cuts reading on revoke. Those differences
//! are [`Capabilities`] a consumer declares, not conformance failures.

use serde::{Deserialize, Serialize};

/// Current report version. Bump when a field's meaning changes.
pub const VERSION: u16 = 1;

/// Where this profile stands in the activity, as this consumer sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Participating now. The only verdict that carries no reason.
    Joined,
    /// This profile left. History is retained; participation stopped.
    Left,
    /// A grant this profile held ended at the clock reading below.
    Expired,
    /// An issuer withdrew this profile's authority.
    Revoked,
    /// Nothing is installed here: never joined, or the invitation is gone.
    NotJoined,
    /// Something is installed but does not admit this profile.
    NotAdmitted,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Self::Joined => "joined",
            Self::Left => "left",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
            Self::NotJoined => "not_joined",
            Self::NotAdmitted => "not_admitted",
        }
    }
}

/// What this profile may do, as the consumer's own authority evaluation read
/// it. Not a promise another peer will agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Manage,
    Write,
    Read,
    Pull,
}

/// Membership as this consumer currently sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership {
    /// Members in the consumer's own fold at this reading. Not a live roster
    /// and not a count of who is reachable.
    pub members: u32,
    /// This profile's access, not anyone else's.
    pub access: Access,
}

/// Grant facts. Absent means this consumer holds no grant to describe, which
/// is not the same as a grant that expired.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    /// When the grant ends, when the consumer knows a bound. `None` means
    /// unbounded as issued, never "unknown".
    pub expires_at_ms: Option<u64>,
    /// When it did end, set only once the consumer's clock passed it.
    pub expired_at_ms: Option<u64>,
}

/// Revocation facts the consumer retained. Present only when this profile's
/// own authority was withdrawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revocation {
    /// The revoking root, when the consumer recorded one. `None` says the
    /// consumer does not know who, not that nobody did.
    pub by: Option<[u8; 32]>,
    /// The consumer's clock reading on the revocation it retained.
    pub at_ms: Option<u64>,
}

/// Whether this profile can still read after a withdrawal. A claim about this
/// consumer's own decryption and projection, not about what peers send.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reading {
    /// Reading is unaffected by the withdrawal.
    Continues,
    /// Content authored after the withdrawal is unreadable here.
    Cut,
    /// Nothing has been withdrawn, or this consumer cannot tell.
    Unknown,
}

/// One consumer's own view of its lifecycle at one clock reading.
///
/// Data only. Building one must not mutate a store, and holding one says
/// nothing about any later reading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleReport {
    /// Report shape version; [`VERSION`] today.
    pub version: u16,
    pub verdict: Verdict,
    /// The consumer's own named reason, present unless [`Verdict::Joined`].
    /// Wording stays the consumer's; this contract does not normalize it.
    pub reason: Option<String>,
    pub membership: Option<Membership>,
    pub grant: Option<Grant>,
    pub revocation: Option<Revocation>,
    pub reading: Reading,
    /// The clock reading this whole report was taken at. Every field above
    /// describes that instant and no other.
    pub now_ms: u64,
}

impl LifecycleReport {
    /// A report at `now_ms` carrying nothing but a verdict and its reason.
    pub fn new(verdict: Verdict, reason: Option<String>, now_ms: u64) -> Self {
        Self {
            version: VERSION,
            verdict,
            reason,
            membership: None,
            grant: None,
            revocation: None,
            reading: Reading::Unknown,
            now_ms,
        }
    }

    /// Joined carries no reason; every other verdict names one.
    pub fn check_invariants(&self) -> Result<(), String> {
        match (self.verdict, self.reason.as_deref()) {
            (Verdict::Joined, None) => Ok(()),
            (Verdict::Joined, Some(_)) => Err("joined carries a refusal reason".into()),
            (verdict, None) => Err(format!("{} carries no reason", verdict.name())),
            (_, Some(reason)) if reason.trim().is_empty() => Err("reason is empty".into()),
            _ => Ok(()),
        }
    }
}

/// Which clock the driver reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Clock {
    /// Deterministic and moved by an explicit command, as the practice
    /// fixture's is.
    Fixed,
    /// Wall clock, as Turnstone's place worker reads.
    System,
}

/// What a consumer declares about itself before the harness walks it. These
/// steer expectations; they never skip a verdict check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// One grant is all this consumer's authority, so its expiry also ends
    /// reconnect and reading. The fixture; not Turnstone, whose membership
    /// survives its grant.
    pub grant_is_all_authority: bool,
    /// Revocation stops the revoked profile reading later content.
    pub cuts_reading_on_revoke: bool,
    /// The consumer separately receipts that a duplicate replay changes no
    /// activity state.
    pub receipts_duplicate_replay: bool,
    pub clock: Clock,
}

/// One step of the I3 sequence, named so a failure says where.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Fresh,
    InviteWrongTarget,
    Join,
    Leave,
    Rejoin,
    Reconnect,
    Expire,
    ReconnectAfterExpiry,
    Revoke,
    ReplayDuplicate,
}

impl Step {
    pub fn name(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::InviteWrongTarget => "invite_wrong_target",
            Self::Join => "join",
            Self::Leave => "leave",
            Self::Rejoin => "rejoin",
            Self::Reconnect => "reconnect",
            Self::Expire => "expire",
            Self::ReconnectAfterExpiry => "reconnect_after_expiry",
            Self::Revoke => "revoke",
            Self::ReplayDuplicate => "replay_duplicate",
        }
    }
}

/// A consumer driving its own lifecycle for the harness.
///
/// Sync on purpose. Both consumers are async underneath — the fixture awaits
/// its replica, Turnstone awaits its place worker — but each already has a
/// blocking test seam (the fixture's self-check, Turnstone's render-free
/// tests, both of which drive a runtime). A sync trait lets both implement it
/// with a small state machine over their own commands, keeps this port free
/// of an async runtime or `async_trait`, and keeps the harness a plain
/// function any `#[test]` can call. Drivers that need to await block inside
/// the method.
///
/// Every step returns the consumer's own named refusal as `Err`. A refusal is
/// a fact, not a harness error.
pub trait LifecycleDriver {
    /// Invite a peer this activity must refuse. Must fail.
    fn invite_wrong_target(&mut self) -> Result<(), String>;
    /// Invite the peer under test with bounded authority.
    fn invite(&mut self) -> Result<(), String>;
    /// Install the invitation.
    fn join(&mut self) -> Result<(), String>;
    /// This profile's view right now. Must not mutate anything.
    fn report(&mut self) -> LifecycleReport;
    /// Stop participating, keeping retained history.
    fn leave(&mut self) -> Result<(), String>;
    /// Install an invitation again after leaving.
    fn rejoin(&mut self) -> Result<(), String>;
    /// Restart hook: drop live session state so the next reconnect really
    /// rechecks. Defaulted to nothing for a consumer with no live session.
    fn restart(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// Recheck admission and authority, then resume.
    fn reconnect(&mut self) -> Result<(), String>;
    /// Reach the grant's expiry: advance a fixed clock, or wait.
    fn expire(&mut self) -> Result<(), String>;
    /// The issuer withdraws this profile's authority.
    fn revoke(&mut self) -> Result<(), String>;
    /// Deliver an operation this consumer already holds.
    fn replay_duplicate(&mut self) -> Result<(), String>;
}

/// Every step's report, in walk order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conformance {
    pub capabilities: Capabilities,
    pub steps: Vec<(Step, LifecycleReport)>,
}

impl Conformance {
    /// The report recorded at `step`, if the walk reached it.
    pub fn at(&self, step: Step) -> Option<&LifecycleReport> {
        self.steps
            .iter()
            .find(|(recorded, _)| *recorded == step)
            .map(|(_, report)| report)
    }
}

/// The step that disagreed, and how.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub step: Step,
    pub mismatch: String,
}

impl core::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}: {}", self.step.name(), self.mismatch)
    }
}

impl std::error::Error for Failure {}

fn fail(step: Step, mismatch: impl Into<String>) -> Failure {
    Failure {
        step,
        mismatch: mismatch.into(),
    }
}

/// Take a report, check the invariants every verdict must hold, and record it.
fn record(
    driver: &mut impl LifecycleDriver,
    steps: &mut Vec<(Step, LifecycleReport)>,
    step: Step,
) -> Result<LifecycleReport, Failure> {
    let report = driver.report();
    report
        .check_invariants()
        .map_err(|mismatch| fail(step, mismatch))?;
    if report.version != VERSION {
        return Err(fail(
            step,
            format!("report version {} is not {VERSION}", report.version),
        ));
    }
    if let Some((_, previous)) = steps.last()
        && report.now_ms < previous.now_ms
    {
        return Err(fail(step, "the clock went backwards"));
    }
    steps.push((step, report.clone()));
    Ok(report)
}

fn expect_verdict(step: Step, report: &LifecycleReport, want: Verdict) -> Result<(), Failure> {
    if report.verdict == want {
        return Ok(());
    }
    Err(fail(
        step,
        format!(
            "verdict {} where {} was due",
            report.verdict.name(),
            want.name()
        ),
    ))
}

fn expect_refused(step: Step, outcome: Result<(), String>) -> Result<(), Failure> {
    match outcome {
        Ok(()) => Err(fail(step, "admitted where a refusal was due")),
        Err(reason) if reason.trim().is_empty() => Err(fail(step, "refused without a reason")),
        Err(_) => Ok(()),
    }
}

fn expect_admitted(step: Step, outcome: Result<(), String>) -> Result<(), Failure> {
    outcome.map_err(|reason| fail(step, format!("refused: {reason}")))
}

/// Walk the I3 sequence against `driver`, asserting the same verdicts and
/// reasons at each step. `caps` steers where the two consumers are recorded to
/// differ; it never skips a verdict check.
pub fn conform(
    driver: &mut impl LifecycleDriver,
    caps: &Capabilities,
) -> Result<Conformance, Failure> {
    let mut steps = Vec::new();

    // Nothing installed yet.
    let report = record(driver, &mut steps, Step::Fresh)?;
    expect_verdict(Step::Fresh, &report, Verdict::NotJoined)?;

    // A wrong target is refused by name, and refusing changes nothing.
    expect_refused(Step::InviteWrongTarget, driver.invite_wrong_target())?;
    let report = record(driver, &mut steps, Step::InviteWrongTarget)?;
    expect_verdict(Step::InviteWrongTarget, &report, Verdict::NotJoined)?;

    // Invite, then join.
    expect_admitted(Step::Join, driver.invite())?;
    expect_admitted(Step::Join, driver.join())?;
    let report = record(driver, &mut steps, Step::Join)?;
    expect_verdict(Step::Join, &report, Verdict::Joined)?;
    if report.membership.is_none() {
        return Err(fail(Step::Join, "joined without membership"));
    }
    if report.grant.is_none() {
        return Err(fail(Step::Join, "joined without grant facts"));
    }

    // Leave keeps history and names why participation stopped.
    expect_admitted(Step::Leave, driver.leave())?;
    let report = record(driver, &mut steps, Step::Leave)?;
    expect_verdict(Step::Leave, &report, Verdict::Left)?;

    expect_admitted(Step::Rejoin, driver.rejoin())?;
    let report = record(driver, &mut steps, Step::Rejoin)?;
    expect_verdict(Step::Rejoin, &report, Verdict::Joined)?;

    // Reconnect after a restart rechecks rather than restoring.
    expect_admitted(Step::Reconnect, driver.restart())?;
    expect_admitted(Step::Reconnect, driver.reconnect())?;
    let report = record(driver, &mut steps, Step::Reconnect)?;
    expect_verdict(Step::Reconnect, &report, Verdict::Joined)?;
    let before_expiry = report.now_ms;

    // Expiry.
    expect_admitted(Step::Expire, driver.expire())?;
    let report = record(driver, &mut steps, Step::Expire)?;
    expect_verdict(Step::Expire, &report, Verdict::Expired)?;
    match report.grant {
        Some(Grant {
            expired_at_ms: Some(_),
            ..
        }) => {},
        _ => return Err(fail(Step::Expire, "expired without an expired_at_ms")),
    }
    if caps.clock == Clock::Fixed && report.now_ms == before_expiry {
        return Err(fail(Step::Expire, "a fixed clock did not advance"));
    }

    // Declared capability: whether the grant was all the authority there was.
    let step = Step::ReconnectAfterExpiry;
    let outcome = driver.reconnect();
    if caps.grant_is_all_authority {
        expect_refused(step, outcome)?;
    } else {
        expect_admitted(step, outcome)?;
    }
    let report = record(driver, &mut steps, step)?;
    expect_verdict(step, &report, Verdict::Expired)?;

    // Revocation.
    expect_admitted(Step::Revoke, driver.revoke())?;
    let report = record(driver, &mut steps, Step::Revoke)?;
    expect_verdict(Step::Revoke, &report, Verdict::Revoked)?;
    if report.revocation.is_none() {
        return Err(fail(Step::Revoke, "revoked without revocation facts"));
    }
    let due = if caps.cuts_reading_on_revoke {
        Reading::Cut
    } else {
        Reading::Continues
    };
    if report.reading != due {
        return Err(fail(
            Step::Revoke,
            format!("reading {:?} where {due:?} was declared", report.reading),
        ));
    }

    // A duplicate changes nothing, for the consumer that receipts it.
    if caps.receipts_duplicate_replay {
        let before = report;
        expect_admitted(Step::ReplayDuplicate, driver.replay_duplicate())?;
        let after = record(driver, &mut steps, Step::ReplayDuplicate)?;
        if after != before {
            return Err(fail(
                Step::ReplayDuplicate,
                "a duplicate changed the report",
            ));
        }
    }

    Ok(Conformance {
        capabilities: *caps,
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPIRES_AT_MS: u64 = 1_000;
    const FOUNDER: [u8; 32] = [7; 32];

    /// A store-free driver: the state machine both consumers implement over
    /// their own commands, with nothing behind it.
    struct Reference {
        caps: Capabilities,
        now_ms: u64,
        installed: bool,
        left: bool,
        revoked: bool,
        /// Reports Joined whatever happened, to prove the harness catches it.
        lies_after_revoke: bool,
    }

    impl Reference {
        fn new(caps: Capabilities) -> Self {
            Self {
                caps,
                now_ms: 50,
                installed: false,
                left: false,
                revoked: false,
                lies_after_revoke: false,
            }
        }

        fn expired(&self) -> bool {
            self.now_ms > EXPIRES_AT_MS
        }

        fn verdict(&self) -> Verdict {
            if !self.installed {
                return if self.left {
                    Verdict::Left
                } else {
                    Verdict::NotJoined
                };
            }
            if self.revoked {
                Verdict::Revoked
            } else if self.expired() {
                Verdict::Expired
            } else {
                Verdict::Joined
            }
        }
    }

    impl LifecycleDriver for Reference {
        fn invite_wrong_target(&mut self) -> Result<(), String> {
            Err("this activity's membership does not contain the invited root".into())
        }

        fn invite(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn join(&mut self) -> Result<(), String> {
            self.installed = true;
            self.left = false;
            Ok(())
        }

        fn report(&mut self) -> LifecycleReport {
            let verdict = if self.lies_after_revoke && self.revoked {
                Verdict::Joined
            } else {
                self.verdict()
            };
            let reason = match verdict {
                Verdict::Joined => None,
                Verdict::Left => Some("not joined: this peer left the activity".into()),
                Verdict::NotJoined => Some("not joined: no invitation is installed".into()),
                Verdict::Expired => Some(format!(
                    "expired: this peer's grant ended at {EXPIRES_AT_MS} ms; clock is {} ms",
                    self.now_ms
                )),
                Verdict::Revoked => Some("revoked: the issuer revoked this peer's grant".into()),
                Verdict::NotAdmitted => Some("not admitted here".into()),
            };
            let mut report = LifecycleReport::new(verdict, reason, self.now_ms);
            if self.installed {
                report.membership = Some(Membership {
                    members: 2,
                    access: Access::Write,
                });
                report.grant = Some(Grant {
                    expires_at_ms: Some(EXPIRES_AT_MS),
                    expired_at_ms: self.expired().then_some(EXPIRES_AT_MS),
                });
            }
            if self.revoked {
                report.revocation = Some(Revocation {
                    by: Some(FOUNDER),
                    at_ms: Some(self.now_ms),
                });
                report.reading = if self.caps.cuts_reading_on_revoke {
                    Reading::Cut
                } else {
                    Reading::Continues
                };
            }
            report
        }

        fn leave(&mut self) -> Result<(), String> {
            self.installed = false;
            self.left = true;
            Ok(())
        }

        fn rejoin(&mut self) -> Result<(), String> {
            self.join()
        }

        fn restart(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn reconnect(&mut self) -> Result<(), String> {
            if !self.installed {
                return Err("not joined: nothing is installed here".into());
            }
            if self.revoked {
                return Err("revoked: the issuer revoked this peer's grant".into());
            }
            if self.expired() && self.caps.grant_is_all_authority {
                return Err(format!(
                    "expired: this peer's grant ended at {EXPIRES_AT_MS} ms"
                ));
            }
            Ok(())
        }

        fn expire(&mut self) -> Result<(), String> {
            self.now_ms = EXPIRES_AT_MS + 1;
            Ok(())
        }

        fn revoke(&mut self) -> Result<(), String> {
            self.revoked = true;
            Ok(())
        }

        fn replay_duplicate(&mut self) -> Result<(), String> {
            Ok(())
        }
    }

    /// The practice fixture's profile: one grant is all authority, reading
    /// survives revocation, duplicate replay is receipted, clock is fixed.
    fn fixture_profile() -> Capabilities {
        Capabilities {
            grant_is_all_authority: true,
            cuts_reading_on_revoke: false,
            receipts_duplicate_replay: true,
            clock: Clock::Fixed,
        }
    }

    /// Turnstone's profile: membership outlives the grant, revocation rotates
    /// the group so reading is cut, duplicate replay is not separately
    /// receipted. Its clock is the system's in product; this store-free
    /// driver advances deterministically, so it declares Fixed.
    fn turnstone_profile() -> Capabilities {
        Capabilities {
            grant_is_all_authority: false,
            cuts_reading_on_revoke: true,
            receipts_duplicate_replay: false,
            clock: Clock::Fixed,
        }
    }

    #[test]
    fn the_reference_driver_conforms_under_the_fixture_profile() {
        let caps = fixture_profile();
        let conformance = conform(&mut Reference::new(caps), &caps).expect("fixture profile");
        assert_eq!(
            conformance.at(Step::Join).map(|report| report.verdict),
            Some(Verdict::Joined)
        );
        assert_eq!(
            conformance.at(Step::Revoke).map(|report| report.reading),
            Some(Reading::Continues)
        );
        assert!(conformance.at(Step::ReplayDuplicate).is_some());
    }

    #[test]
    fn the_reference_driver_conforms_under_the_turnstone_profile() {
        let caps = turnstone_profile();
        let conformance = conform(&mut Reference::new(caps), &caps).expect("turnstone profile");
        assert_eq!(
            conformance.at(Step::Revoke).map(|report| report.reading),
            Some(Reading::Cut)
        );
        // Not declared, so the walk does not reach it.
        assert!(conformance.at(Step::ReplayDuplicate).is_none());
    }

    #[test]
    fn every_step_is_walked_in_order() {
        let caps = fixture_profile();
        let conformance = conform(&mut Reference::new(caps), &caps).expect("fixture profile");
        let walked: Vec<_> = conformance
            .steps
            .iter()
            .map(|(step, _)| step.name())
            .collect();
        assert_eq!(
            walked,
            [
                "fresh",
                "invite_wrong_target",
                "join",
                "leave",
                "rejoin",
                "reconnect",
                "expire",
                "reconnect_after_expiry",
                "revoke",
                "replay_duplicate",
            ]
        );
    }

    #[test]
    fn a_driver_that_reports_joined_after_revoke_fails_at_the_revoke_step() {
        let caps = turnstone_profile();
        let mut driver = Reference::new(caps);
        driver.lies_after_revoke = true;
        let failure = conform(&mut driver, &caps).expect_err("the lie must be caught");
        assert_eq!(failure.step, Step::Revoke);
        assert!(failure.mismatch.contains("joined"), "{failure}");
    }

    #[test]
    fn a_declared_capability_does_not_skip_the_verdict_check() {
        // Declaring "reading is cut" while the driver says it continues is a
        // failure, not a waiver.
        let caps = Capabilities {
            cuts_reading_on_revoke: true,
            ..fixture_profile()
        };
        let failure = conform(&mut Reference::new(fixture_profile()), &caps)
            .expect_err("the declaration must be checked");
        assert_eq!(failure.step, Step::Revoke);
        assert!(failure.mismatch.contains("Cut"), "{failure}");
    }

    #[test]
    fn a_grant_that_is_all_authority_must_refuse_reconnect_after_expiry() {
        // The driver behaves like Turnstone; the declaration says fixture.
        let caps = fixture_profile();
        let failure = conform(&mut Reference::new(turnstone_profile()), &caps)
            .expect_err("an admitted reconnect must fail here");
        assert_eq!(failure.step, Step::ReconnectAfterExpiry);
        assert!(failure.mismatch.contains("refusal was due"), "{failure}");
    }

    #[test]
    fn joined_has_no_reason_and_every_other_verdict_has_one() {
        assert!(
            LifecycleReport::new(Verdict::Joined, None, 50)
                .check_invariants()
                .is_ok()
        );
        assert!(
            LifecycleReport::new(Verdict::Joined, Some("because".into()), 50)
                .check_invariants()
                .is_err()
        );
        for verdict in [
            Verdict::Left,
            Verdict::Expired,
            Verdict::Revoked,
            Verdict::NotJoined,
            Verdict::NotAdmitted,
        ] {
            assert!(
                LifecycleReport::new(verdict, None, 50)
                    .check_invariants()
                    .is_err(),
                "{} must name a reason",
                verdict.name()
            );
            assert!(
                LifecycleReport::new(verdict, Some("  ".into()), 50)
                    .check_invariants()
                    .is_err(),
                "{} must name a non-empty reason",
                verdict.name()
            );
            assert!(
                LifecycleReport::new(verdict, Some("named".into()), 50)
                    .check_invariants()
                    .is_ok()
            );
        }
    }

    fn pinned_report() -> LifecycleReport {
        LifecycleReport {
            version: VERSION,
            verdict: Verdict::Revoked,
            reason: Some("revoked: the issuer revoked this peer's delegation".into()),
            membership: Some(Membership {
                members: 2,
                access: Access::Write,
            }),
            grant: Some(Grant {
                expires_at_ms: Some(1_000),
                expired_at_ms: None,
            }),
            revocation: Some(Revocation {
                by: None,
                at_ms: Some(1_001),
            }),
            reading: Reading::Continues,
            now_ms: 1_001,
        }
    }

    #[test]
    fn a_report_round_trips_through_serde() {
        let report = pinned_report();
        let json = serde_json::to_string(&report).expect("serialize");
        let back: LifecycleReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(report, back);
    }

    /// Pinned wire shape. Changing this string is a version bump.
    const PINNED_JSON: &str = r#"{"version":1,"verdict":"revoked","reason":"revoked: the issuer revoked this peer's delegation","membership":{"members":2,"access":"write"},"grant":{"expires_at_ms":1000,"expired_at_ms":null},"revocation":{"by":null,"at_ms":1001},"reading":"continues","now_ms":1001}"#;

    #[test]
    fn the_wire_shape_is_pinned() {
        assert_eq!(
            serde_json::to_string(&pinned_report()).unwrap(),
            PINNED_JSON
        );
        assert_eq!(
            serde_json::from_str::<LifecycleReport>(PINNED_JSON).unwrap(),
            pinned_report()
        );
    }

    #[test]
    fn a_conformance_walk_round_trips_through_serde() {
        let caps = fixture_profile();
        let conformance = conform(&mut Reference::new(caps), &caps).expect("fixture profile");
        let json = serde_json::to_string(&conformance).expect("serialize");
        let back: Conformance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(conformance, back);
    }
}
