// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Map's physics composition form: role, role groups, two-law mix,
//! grouping, two-stage schedule and speed, as one model both hosts drive.
//!
//! A host renders [`PhysicsForm::fields`] (ids, labels, valid choices and the
//! current values), feeds each edit back through [`PhysicsForm::choose`],
//! [`PhysicsForm::enter`] or [`PhysicsForm::tick`], and turns a press into a
//! [`FormAction`] with [`PhysicsForm::action`]. The model owns what the form
//! offers, the defaults, the read-back of the canvas's authored spec
//! ([`Readback`]), the validation, and the refusal wording; the
//! [`physics_controls`] functions it calls own the spec edit. Graphshell's web
//! tree and Turnstone's Arrange pane differ only in how they draw it.

use super::composition::CHARGE_BETWEEN_WEIGHT;
use super::physics_catalog::PhysicsLaw;
use super::physics_controls::{
    self, SPEED_PRESETS, StageDraft, TwoLawGrouping, TwoLawMix, TwoStageSchedule,
};
use super::schedule::StageStop;
use super::{
    CANVAS_PHYSICS_KIND_SOURCES, CANVAS_PHYSICS_LAWS, Canvas, PhysicsChoice, PhysicsKindSource,
    PhysicsOverlay, Role,
};

/// The frame count a stage form starts with.
pub const DEFAULT_FRAMES: &str = "60";
/// The capture choice that names no role.
pub const NO_CAPTURE: &str = "none";

/// Which of a pair a field belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    First,
    Second,
}

impl Slot {
    pub const BOTH: [Slot; 2] = [Slot::First, Slot::Second];

    /// The slot as a word: `first`, `second`.
    pub fn id(self) -> &'static str {
        match self {
            Slot::First => "first",
            Slot::Second => "second",
        }
    }

    /// The heading over a schedule stage.
    pub fn stage_title(self) -> &'static str {
        match self {
            Slot::First => "Stage 1",
            Slot::Second => "Stage 2",
        }
    }
}

/// One field of the form. Hosts key their widgets, hooks and scripts by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldId {
    Role,
    RoleGroups,
    Speed,
    MixLaw(Slot),
    MixShare(Slot),
    GroupPartition,
    GroupOuter,
    GroupOuterShare,
    GroupInner,
    StageLaw(Slot),
    StageStop(Slot),
    StageFrames(Slot),
    StageCapture(Slot),
    ScheduleOverlay(PhysicsOverlay),
}

impl FieldId {
    /// The field's stable id: `mix.first`, `schedule.second.stop`,
    /// `schedule.overlay.gravity-locus`.
    pub fn key(self) -> String {
        match self {
            FieldId::Role => "physics.role".into(),
            FieldId::RoleGroups => "physics.role-groups".into(),
            FieldId::Speed => "physics.speed".into(),
            FieldId::MixLaw(slot) => format!("mix.{}", slot.id()),
            FieldId::MixShare(slot) => format!("mix.{}-share", slot.id()),
            FieldId::GroupPartition => "grouping.partition".into(),
            FieldId::GroupOuter => "grouping.outer".into(),
            FieldId::GroupOuterShare => "grouping.outer-share".into(),
            FieldId::GroupInner => "grouping.inner".into(),
            FieldId::StageLaw(slot) => format!("schedule.{}.law", slot.id()),
            FieldId::StageStop(slot) => format!("schedule.{}.stop", slot.id()),
            FieldId::StageFrames(slot) => format!("schedule.{}.frames", slot.id()),
            FieldId::StageCapture(slot) => format!("schedule.{}.capture", slot.id()),
            FieldId::ScheduleOverlay(overlay) => format!("schedule.overlay.{}", overlay.id()),
        }
    }

    /// The field's name, as a control is labelled.
    pub fn label(self) -> String {
        let ordinal = |slot: Slot| match slot {
            Slot::First => "First",
            Slot::Second => "Second",
        };
        match self {
            FieldId::Role => "Role".into(),
            FieldId::RoleGroups => "Role groups".into(),
            FieldId::Speed => "Speed".into(),
            FieldId::MixLaw(slot) => format!("{} law", ordinal(slot)),
            FieldId::MixShare(slot) => format!("{} share", ordinal(slot)),
            FieldId::GroupPartition => "Grouping partition".into(),
            FieldId::GroupOuter => "Outer law".into(),
            FieldId::GroupOuterShare => "Outer share".into(),
            FieldId::GroupInner => "Inner law".into(),
            FieldId::StageLaw(slot) => format!("{} stage law", ordinal(slot)),
            FieldId::StageStop(slot) => format!("{} stage stop", ordinal(slot)),
            FieldId::StageFrames(slot) => format!("{} stage frames", ordinal(slot)),
            FieldId::StageCapture(slot) => format!("{} stage capture", ordinal(slot)),
            FieldId::ScheduleOverlay(overlay) => format!("Schedule {}", overlay.label()),
        }
    }
}

/// How a field is edited, with what it holds now.
#[derive(Clone, Debug, PartialEq)]
pub enum Control {
    /// One of `options` (`(id, label)`); `current` is an id from them.
    Choice {
        options: Vec<(&'static str, &'static str)>,
        current: &'static str,
    },
    /// Text that parses as a number, kept as typed so an authored value
    /// reads back digit for digit. `integer` asks for a whole count.
    Number {
        text: String,
        min: f64,
        integer: bool,
    },
    Toggle(bool),
}

/// A field as a host draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub id: FieldId,
    pub label: String,
    pub control: Control,
    /// False while the field changes nothing (frames, with a stop that does
    /// not count them). A host may hide it or leave it; the value is kept.
    pub relevant: bool,
}

/// The editors the form is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Mix,
    Grouping,
    Schedule,
}

/// Fields drawn together, under one heading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub title: Option<&'static str>,
    pub note: Option<&'static str>,
    pub fields: Vec<FieldId>,
}

/// What a button in the form does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    ApplyRole,
    ApplyRoleGroups,
    ApplyMix,
    UseSingleLaw,
    ApplyGrouping,
    ApplySchedule,
}

impl Command {
    pub fn label(self) -> &'static str {
        match self {
            Command::ApplyRole => "Apply role",
            Command::ApplyRoleGroups => "Apply role groups",
            Command::ApplyMix => "Apply mix",
            Command::UseSingleLaw => "Use single law",
            Command::ApplyGrouping => "Apply grouping",
            Command::ApplySchedule => "Apply and run schedule",
        }
    }

    /// The button's id for scripts and tests.
    pub fn id(self) -> &'static str {
        match self {
            Command::ApplyRole => "apply-role",
            Command::ApplyRoleGroups => "apply-role-groups",
            Command::ApplyMix => "apply-mix",
            Command::UseSingleLaw => "use-single-law",
            Command::ApplyGrouping => "apply-grouping",
            Command::ApplySchedule => "apply-schedule",
        }
    }
}

impl Section {
    pub const ALL: [Section; 3] = [Section::Mix, Section::Grouping, Section::Schedule];

    pub fn title(self) -> &'static str {
        match self {
            Section::Mix => "Two-law mix",
            Section::Grouping => "Grouped composition",
            Section::Schedule => "Two-stage schedule",
        }
    }

    /// The editor's id for scripts and tests.
    pub fn editor(self) -> &'static str {
        match self {
            Section::Mix => "two-law-mix",
            Section::Grouping => "grouped",
            Section::Schedule => "two-stage-schedule",
        }
    }

    /// The accessible name of the status under the section: what the canvas
    /// authored for it.
    pub fn status_label(self) -> &'static str {
        match self {
            Section::Mix => "Authored mix status",
            Section::Grouping => "Authored grouping status",
            Section::Schedule => "Authored schedule status",
        }
    }

    /// What to read before the fields.
    pub fn intro(self) -> &'static [&'static str] {
        match self {
            Section::Schedule => &[
                "Frame counts apply only when a stage stops after Frames. A capture applies its role when that stage ends.",
                "Applying starts stage 1 from the current layout. Source and role edits keep the running stage.",
            ],
            _ => &[],
        }
    }

    /// The fields, in the order they read.
    pub fn groups(self) -> Vec<Group> {
        match self {
            Section::Mix => vec![Group {
                title: None,
                note: Some("Shares set each law’s contribution; they do not need to add to one."),
                fields: vec![
                    FieldId::MixLaw(Slot::First),
                    FieldId::MixShare(Slot::First),
                    FieldId::MixLaw(Slot::Second),
                    FieldId::MixShare(Slot::Second),
                ],
            }],
            Section::Grouping => vec![Group {
                title: None,
                note: Some(
                    "The outer law's repulsion acts between groups; the inner law acts within each group. The outer share scales that repulsion and is not normalized.",
                ),
                fields: vec![
                    FieldId::GroupPartition,
                    FieldId::GroupOuter,
                    FieldId::GroupOuterShare,
                    FieldId::GroupInner,
                ],
            }],
            Section::Schedule => {
                let stage = |slot: Slot| Group {
                    title: Some(slot.stage_title()),
                    note: None,
                    fields: vec![
                        FieldId::StageLaw(slot),
                        FieldId::StageStop(slot),
                        FieldId::StageFrames(slot),
                        FieldId::StageCapture(slot),
                    ],
                };
                vec![
                    stage(Slot::First),
                    stage(Slot::Second),
                    Group {
                        title: Some("Common overlays"),
                        note: Some(
                            "Every selected overlay applies to both stages. If either law rejects one, the whole schedule is refused.",
                        ),
                        fields: PhysicsOverlay::ALL
                            .into_iter()
                            .map(FieldId::ScheduleOverlay)
                            .collect(),
                    },
                ]
            },
        }
    }

    /// The buttons, in order.
    pub fn commands(self) -> &'static [Command] {
        match self {
            Section::Mix => &[Command::ApplyMix, Command::UseSingleLaw],
            Section::Grouping => &[Command::ApplyGrouping],
            Section::Schedule => &[Command::ApplySchedule],
        }
    }
}

const STOP_OPTIONS: [(&str, &str); 3] = [
    (STOP_REST, "Rest"),
    (STOP_FRAMES, "Frames"),
    (STOP_LAW, "Law done"),
];
/// The stop choices' ids.
pub const STOP_REST: &str = "rest";
pub const STOP_FRAMES: &str = "frames";
pub const STOP_LAW: &str = "law";

/// What the canvas authored for the composing fields, as the readers answer:
/// the roles, the speed preset, and the three forms each as authored (`Ok`
/// with a value), absent (`Ok(None)`) or in a shape the form cannot show
/// (`Err` with the reason).
#[derive(Clone, Debug, PartialEq)]
pub struct Readback {
    pub role: Role,
    pub role_groups: PhysicsKindSource,
    /// An index into [`SPEED_PRESETS`].
    pub speed: usize,
    pub mix: Result<Option<TwoLawMix>, String>,
    pub grouping: Result<Option<TwoLawGrouping>, String>,
    pub schedule: Result<Option<TwoStageSchedule>, String>,
}

impl Default for Readback {
    /// What a canvas that authored nothing reads as: no composition, the
    /// seeded role, the default speed.
    fn default() -> Self {
        Self {
            role: Role::ALL[0],
            role_groups: PhysicsKindSource::ALL[0],
            speed: physics_controls::DEFAULT_SPEED_PRESET,
            mix: Ok(None),
            grouping: Ok(None),
            schedule: Ok(None),
        }
    }
}

impl Readback {
    pub fn of(canvas: &Canvas) -> Self {
        Self {
            role: canvas.arrangement_roles().default,
            role_groups: canvas.role_group_source(),
            speed: physics_controls::speed_preset(canvas),
            mix: physics_controls::read_two_law_mix(canvas),
            grouping: physics_controls::read_two_law_grouping(canvas),
            schedule: physics_controls::read_two_stage_schedule(canvas),
        }
    }

    /// Whether `other` read the same mix, grouping and schedule: the form's
    /// unapplied values need follow the canvas only when this is false.
    pub fn same_composition(&self, other: &Self) -> bool {
        self.mix == other.mix && self.grouping == other.grouping && self.schedule == other.schedule
    }

    /// The speed preset's id.
    pub fn speed_id(&self) -> &'static str {
        SPEED_PRESETS[self.speed.min(SPEED_PRESETS.len() - 1)].0
    }

    /// What the canvas authored for `section`, in words, or the note when it
    /// authored nothing the form can show.
    pub fn summary(&self, section: Section) -> String {
        match section {
            Section::Mix => match &self.mix {
                Ok(Some(mix)) => format!(
                    "{} {} + {} {}",
                    mix.first.label(),
                    mix.first_share,
                    mix.second.label(),
                    mix.second_share
                ),
                other => note_of("two-law mix", "mix", other),
            },
            Section::Grouping => match &self.grouping {
                Ok(Some(grouping)) => format!(
                    "{} between {} groups, {} within, outer share {}",
                    grouping.outer.label(),
                    grouping.partition.label(),
                    grouping.inner.label(),
                    grouping.outer_share
                ),
                other => note_of("grouping", "grouping", other),
            },
            Section::Schedule => match &self.schedule {
                Ok(Some(schedule)) => format!(
                    "{} then {}",
                    schedule.first.law.label(),
                    schedule.second.law.label()
                ),
                other => note_of("two-stage schedule", "schedule", other),
            },
        }
    }

    /// The note under `section`'s fields: empty when the form shows what the
    /// canvas authored.
    pub fn note(&self, section: Section) -> String {
        let authored = match section {
            Section::Mix => matches!(self.mix, Ok(Some(_))),
            Section::Grouping => matches!(self.grouping, Ok(Some(_))),
            Section::Schedule => matches!(self.schedule, Ok(Some(_))),
        };
        if authored {
            String::new()
        } else {
            self.summary(section)
        }
    }
}

/// The note for a section the canvas did not author in the form's shape.
fn note_of<T>(what: &str, shape: &str, authored: &Result<Option<T>, String>) -> String {
    match authored {
        Ok(Some(_)) => String::new(),
        Ok(None) => format!("No authored {what}"),
        Err(reason) => format!(
            "Unsupported authored {shape} shape · {reason} · editor values are not readback"
        ),
    }
}

/// A schedule stage as the form holds it: `frames` is kept while the stop
/// does not count them, so switching back restores what was typed.
#[derive(Clone, Debug, PartialEq)]
struct StageFields {
    law: PhysicsLaw,
    stop: &'static str,
    frames: String,
    capture: Option<Role>,
}

impl StageFields {
    fn new(law: PhysicsLaw) -> Self {
        Self {
            law,
            stop: STOP_REST,
            frames: DEFAULT_FRAMES.into(),
            capture: None,
        }
    }

    fn take(&mut self, draft: StageDraft) {
        self.law = draft.law;
        self.capture = draft.capture;
        match draft.stop {
            StageStop::Rest => self.stop = STOP_REST,
            StageStop::Frames(count) => {
                self.stop = STOP_FRAMES;
                self.frames = count.to_string();
            },
            StageStop::LawDone => self.stop = STOP_LAW,
        }
    }

    fn draft(&self, label: &str) -> Result<StageDraft, String> {
        let stop = match self.stop {
            STOP_FRAMES => StageStop::Frames(
                self.frames
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| format!("{label} frames must be a positive integer"))?,
            ),
            STOP_LAW => StageStop::LawDone,
            _ => StageStop::Rest,
        };
        Ok(StageDraft {
            law: self.law,
            stop,
            capture: self.capture,
        })
    }
}

/// The form: what each field holds between presses.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsForm {
    role: Role,
    role_groups: PhysicsKindSource,
    speed: usize,
    mix: [(PhysicsLaw, String); 2],
    grouping: (PhysicsKindSource, PhysicsLaw, PhysicsLaw, String),
    stages: [StageFields; 2],
    overlays: Vec<PhysicsOverlay>,
}

fn slot_index(slot: Slot) -> usize {
    match slot {
        Slot::First => 0,
        Slot::Second => 1,
    }
}

fn law_options() -> Vec<(&'static str, &'static str)> {
    CANVAS_PHYSICS_LAWS.to_vec()
}

fn kind_options() -> Vec<(&'static str, &'static str)> {
    CANVAS_PHYSICS_KIND_SOURCES.to_vec()
}

fn role_options() -> impl Iterator<Item = (&'static str, &'static str)> {
    Role::ALL.iter().map(|role| (role.id(), role.label()))
}

impl Default for PhysicsForm {
    fn default() -> Self {
        Self {
            role: Role::ALL[0],
            role_groups: PhysicsKindSource::ALL[0],
            speed: physics_controls::DEFAULT_SPEED_PRESET,
            mix: [
                (PhysicsLaw::ALL[0], "1".into()),
                (PhysicsLaw::ALL[1], "1".into()),
            ],
            grouping: (
                PhysicsKindSource::ALL[0],
                PhysicsLaw::Charge,
                PhysicsLaw::Springs,
                CHARGE_BETWEEN_WEIGHT.to_string(),
            ),
            stages: [
                StageFields::new(PhysicsLaw::ALL[0]),
                StageFields::new(PhysicsLaw::ALL[1]),
            ],
            overlays: Vec::new(),
        }
    }
}

impl PhysicsForm {
    /// The defaults, then what `canvas` authored.
    pub fn new(canvas: &Canvas) -> Self {
        let mut form = Self::default();
        let readback = Readback::of(canvas);
        form.follow_roles(&readback);
        form.follow_speed(&readback);
        form.follow_composition(&readback);
        form
    }

    /// Take the role and the role groups the canvas runs.
    pub fn follow_roles(&mut self, readback: &Readback) {
        self.role = readback.role;
        self.role_groups = readback.role_groups;
    }

    /// Take the speed preset the canvas runs. A host that applies the
    /// picked speed itself leaves this out, or the canvas's own pace would
    /// read as a pick.
    pub fn follow_speed(&mut self, readback: &Readback) {
        self.speed = readback.speed;
    }

    /// Take what the canvas authored, for each form it authored in the shape
    /// the form can show. A form it did not author keeps its values.
    pub fn follow_composition(&mut self, readback: &Readback) {
        if let Ok(Some(mix)) = &readback.mix {
            self.mix = [
                (mix.first, mix.first_share.to_string()),
                (mix.second, mix.second_share.to_string()),
            ];
        }
        if let Ok(Some(grouping)) = &readback.grouping {
            self.grouping = (
                grouping.partition,
                grouping.outer,
                grouping.inner,
                grouping.outer_share.to_string(),
            );
        }
        if let Ok(Some(schedule)) = &readback.schedule {
            self.stages[0].take(schedule.first);
            self.stages[1].take(schedule.second);
            self.overlays = schedule.overlays.clone();
        }
    }

    /// One field, as a host draws it.
    pub fn field(&self, id: FieldId) -> Field {
        let law = |current: PhysicsLaw| Control::Choice {
            options: law_options(),
            current: current.id(),
        };
        let number = |text: &str, min: f64, integer: bool| Control::Number {
            text: text.to_string(),
            min,
            integer,
        };
        let stage = |slot: Slot| &self.stages[slot_index(slot)];
        let control = match id {
            FieldId::Role => Control::Choice {
                options: role_options().collect(),
                current: self.role.id(),
            },
            FieldId::RoleGroups => Control::Choice {
                options: kind_options(),
                current: self.role_groups.id(),
            },
            FieldId::Speed => Control::Choice {
                options: SPEED_PRESETS.to_vec(),
                current: SPEED_PRESETS[self.speed.min(SPEED_PRESETS.len() - 1)].0,
            },
            FieldId::MixLaw(slot) => law(self.mix[slot_index(slot)].0),
            FieldId::MixShare(slot) => number(&self.mix[slot_index(slot)].1, 0.0, false),
            FieldId::GroupPartition => Control::Choice {
                options: kind_options(),
                current: self.grouping.0.id(),
            },
            FieldId::GroupOuter => law(self.grouping.1),
            FieldId::GroupInner => law(self.grouping.2),
            FieldId::GroupOuterShare => number(&self.grouping.3, 0.0, false),
            FieldId::StageLaw(slot) => law(stage(slot).law),
            FieldId::StageStop(slot) => Control::Choice {
                options: STOP_OPTIONS.to_vec(),
                current: stage(slot).stop,
            },
            FieldId::StageFrames(slot) => number(&stage(slot).frames, 1.0, true),
            FieldId::StageCapture(slot) => Control::Choice {
                options: std::iter::once((NO_CAPTURE, "No capture"))
                    .chain(role_options())
                    .collect(),
                current: stage(slot).capture.map_or(NO_CAPTURE, Role::id),
            },
            FieldId::ScheduleOverlay(overlay) => Control::Toggle(self.overlays.contains(&overlay)),
        };
        let relevant = match id {
            FieldId::StageFrames(slot) => stage(slot).stop == STOP_FRAMES,
            _ => true,
        };
        Field {
            id,
            label: id.label(),
            control,
            relevant,
        }
    }

    /// The fields of a group, as a host draws them.
    pub fn fields<'a>(&'a self, group: &'a Group) -> impl Iterator<Item = Field> + 'a {
        group.fields.iter().map(|id| self.field(*id))
    }

    /// What the field holds as text: the chosen id, the typed number, or
    /// `true` / `false`.
    pub fn value(&self, id: FieldId) -> String {
        match self.field(id).control {
            Control::Choice { current, .. } => current.to_string(),
            Control::Number { text, .. } => text,
            Control::Toggle(on) => on.to_string(),
        }
    }

    /// Pick option `option` of a choice field. False, and nothing changed,
    /// when the field is not a choice or does not offer that id.
    pub fn choose(&mut self, id: FieldId, option: &str) -> bool {
        let Control::Choice { options, .. } = self.field(id).control else {
            return false;
        };
        let Some(option) = options
            .iter()
            .map(|(known, _)| *known)
            .find(|known| *known == option)
        else {
            return false;
        };
        let (law, kind) = (PhysicsLaw::parse(option), PhysicsKindSource::parse(option));
        match id {
            FieldId::Role => self.role = Role::parse(option).unwrap_or(self.role),
            FieldId::RoleGroups => self.role_groups = kind.unwrap_or(self.role_groups),
            FieldId::Speed => {
                self.speed = SPEED_PRESETS
                    .iter()
                    .position(|(known, _)| *known == option)
                    .unwrap_or(self.speed);
            },
            FieldId::MixLaw(slot) => {
                self.mix[slot_index(slot)].0 = law.unwrap_or(self.mix[slot_index(slot)].0)
            },
            FieldId::GroupPartition => self.grouping.0 = kind.unwrap_or(self.grouping.0),
            FieldId::GroupOuter => self.grouping.1 = law.unwrap_or(self.grouping.1),
            FieldId::GroupInner => self.grouping.2 = law.unwrap_or(self.grouping.2),
            FieldId::StageLaw(slot) => {
                let stage = &mut self.stages[slot_index(slot)];
                stage.law = law.unwrap_or(stage.law);
            },
            FieldId::StageStop(slot) => {
                self.stages[slot_index(slot)].stop = option;
            },
            FieldId::StageCapture(slot) => {
                self.stages[slot_index(slot)].capture = Role::parse(option);
            },
            _ => return false,
        }
        true
    }

    /// Type into a number field. The text is kept as it is and judged when a
    /// button asks for the action. False when the field is not a number.
    pub fn enter(&mut self, id: FieldId, text: &str) -> bool {
        let slot = match id {
            FieldId::MixShare(slot) => &mut self.mix[slot_index(slot)].1,
            FieldId::GroupOuterShare => &mut self.grouping.3,
            FieldId::StageFrames(slot) => &mut self.stages[slot_index(slot)].frames,
            _ => return false,
        };
        *slot = text.to_string();
        true
    }

    /// Tick or untick a toggle field. False when the field is not a toggle.
    pub fn tick(&mut self, id: FieldId, on: bool) -> bool {
        let FieldId::ScheduleOverlay(overlay) = id else {
            return false;
        };
        self.overlays.retain(|known| *known != overlay);
        if on {
            self.overlays.push(overlay);
        }
        true
    }

    /// The overlays the schedule form ticks, in catalog order.
    pub fn schedule_overlays(&self) -> Vec<PhysicsOverlay> {
        physics_controls::ticked_overlays(|overlay| self.overlays.contains(&overlay))
    }

    /// The mix the fields hold, or why they do not hold one.
    pub fn mix(&self) -> Result<TwoLawMix, String> {
        let share = |text: &str| text.trim().parse::<f64>();
        match (share(&self.mix[0].1), share(&self.mix[1].1)) {
            (Ok(first_share), Ok(second_share)) => Ok(TwoLawMix {
                first: self.mix[0].0,
                second: self.mix[1].0,
                first_share,
                second_share,
            }),
            _ => Err("Mix shares must be numbers".into()),
        }
    }

    /// The grouping the fields hold, or why they do not hold one.
    pub fn grouping(&self) -> Result<TwoLawGrouping, String> {
        let outer_share = self
            .grouping
            .3
            .trim()
            .parse::<f64>()
            .map_err(|_| "Grouping outer share must be a number".to_string())?;
        Ok(TwoLawGrouping {
            partition: self.grouping.0,
            outer: self.grouping.1,
            inner: self.grouping.2,
            outer_share,
        })
    }

    /// The schedule the fields hold, or why they do not hold one.
    pub fn schedule(&self) -> Result<TwoStageSchedule, String> {
        let stage = |slot: Slot, label: &str| self.stages[slot_index(slot)].draft(label);
        Ok(TwoStageSchedule {
            first: stage(Slot::First, "First stage").map_err(refused("Schedule"))?,
            second: stage(Slot::Second, "Second stage").map_err(refused("Schedule"))?,
            overlays: self.schedule_overlays(),
        })
    }

    /// What pressing `command` asks of the canvas. `picked` is the flat
    /// choice the host's own law, overlay and source controls hold, which
    /// Use single law writes. An `Err` is a refusal, worded as a status.
    pub fn action(&self, command: Command, picked: &PhysicsChoice) -> Result<FormAction, String> {
        Ok(match command {
            Command::ApplyRole => FormAction::Role(self.role),
            Command::ApplyRoleGroups => FormAction::RoleGroups(self.role_groups),
            Command::ApplyMix => FormAction::Mix(self.mix()?),
            Command::UseSingleLaw => FormAction::SingleLaw(picked.clone()),
            Command::ApplyGrouping => FormAction::Grouping(self.grouping()?),
            Command::ApplySchedule => FormAction::Schedule(self.schedule()?),
        })
    }

    /// Run the speed the form holds.
    pub fn speed_action(&self) -> FormAction {
        FormAction::Speed(SPEED_PRESETS[self.speed.min(SPEED_PRESETS.len() - 1)].0)
    }
}

fn refused(what: &'static str) -> impl Fn(String) -> String {
    move |reason| format!("{what} refused · {reason}")
}

/// An edit the form asks of the canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum FormAction {
    Role(Role),
    RoleGroups(PhysicsKindSource),
    /// A speed preset's id, from [`SPEED_PRESETS`].
    Speed(&'static str),
    Mix(TwoLawMix),
    Grouping(TwoLawGrouping),
    Schedule(TwoStageSchedule),
    /// The flat choice, replacing any composition.
    SingleLaw(PhysicsChoice),
}

/// What running a [`FormAction`] came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormOutcome {
    /// Whether the canvas took the edit.
    pub applied: bool,
    /// The status line: the canvas's account, or the refusal.
    pub status: String,
}

impl FormAction {
    /// Whether the edit replaces the law, so a host that records where a law
    /// started records it before running this.
    pub fn changes_law(&self) -> bool {
        matches!(
            self,
            FormAction::Mix(_)
                | FormAction::Grouping(_)
                | FormAction::Schedule(_)
                | FormAction::SingleLaw(_)
        )
    }

    fn refusal_name(&self) -> &'static str {
        match self {
            FormAction::Role(_) => "Role change",
            FormAction::RoleGroups(_) => "Role groups change",
            FormAction::Speed(_) => "Speed",
            FormAction::Mix(_) => "Mix",
            FormAction::Grouping(_) => "Grouping",
            FormAction::Schedule(_) => "Schedule",
            FormAction::SingleLaw(_) => "Single law",
        }
    }

    /// Make the edit through [`physics_controls`]. The canvas admits or
    /// refuses it whole before its live state changes.
    pub fn run(&self, canvas: &mut Canvas) -> FormOutcome {
        let result = match self {
            FormAction::Role(role) => physics_controls::apply_role(canvas, *role),
            FormAction::RoleGroups(source) => {
                physics_controls::apply_role_group_source(canvas, *source)
            },
            FormAction::Speed(id) => physics_controls::apply_speed(canvas, id),
            FormAction::Mix(mix) => physics_controls::apply_mix(canvas, mix),
            FormAction::Grouping(grouping) => physics_controls::apply_grouping(canvas, grouping),
            FormAction::Schedule(schedule) => {
                physics_controls::apply_two_stage_schedule(canvas, schedule)
            },
            FormAction::SingleLaw(choice) => physics_controls::apply_single_law(canvas, choice),
        };
        match result {
            Ok(status) => FormOutcome {
                applied: true,
                status,
            },
            Err(reason) => FormOutcome {
                applied: false,
                status: refused(self.refusal_name())(reason),
            },
        }
    }
}
