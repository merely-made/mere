/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! What a host hands the view.

use incipit::SessionId;

/// Everything the view shows, filled by the host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MereViewModel {
    /// The mere's name, which labels the view's region.
    pub title: String,
    pub status: ViewStatus,
    pub sessions: Vec<SessionEntry>,
    /// Whether the host offers minting a session.
    pub can_mint: bool,
    /// Hide the duplicate graph when the host's editing canvas shows it.
    /// Session and archive controls still mount in this same component.
    pub hide_graph: bool,
    /// Archive readings and operations advertised by the host, if available.
    pub archive: Option<ArchiveModel>,
    /// Where the sessions and archives show: beside the graph, as an overmap
    /// band above it, or as an overmap tier on its canvas.
    pub placement: SessionPlacement,
    /// In a tier, the most archives a session or the shelf shows one by one;
    /// past it they collapse into one "+N" node. `None` takes
    /// [`DEFAULT_MAX_ARCHIVE_NODES`](crate::DEFAULT_MAX_ARCHIVE_NODES), 3.
    pub max_archive_nodes: Option<usize>,
    pub graph: GraphModel,
    /// The layout in use: one of cartography's graph-only strategy ids.
    pub layout: String,
    /// The layouts the host offers, as `(id, label)`. Empty offers no switch.
    pub layouts: Vec<(String, String)>,
    /// The host's action slot beside the graph: New, Open, recent.
    pub actions: Vec<HostAction>,
    /// The reason the host gave for the last request it declined.
    pub notice: Option<String>,
    /// A merge waiting for a choice on each conflict, shown as its card.
    pub merge: Option<MergeModel>,
}

/// A pending merge: a branch or picked-off session merging back into the
/// session it came from, and the conflicts that wait for a choice.
#[derive(Clone, Debug, PartialEq)]
pub struct MergeModel {
    pub source: SessionId,
    pub source_name: String,
    pub target_name: String,
    pub conflicts: Vec<MergeConflictEntry>,
}

impl MergeModel {
    /// Whether every conflict has a choice, so the merge can apply.
    pub fn resolved(&self) -> bool {
        self.conflicts
            .iter()
            .all(|conflict| conflict.chosen.is_some())
    }
}

/// One part both sides changed differently, as the host words it.
#[derive(Clone, Debug, PartialEq)]
pub struct MergeConflictEntry {
    /// Stable across readings; the conflict's `data-key`.
    pub key: String,
    /// What the part is, such as "Title of One".
    pub label: String,
    /// Each side's value in words; empty for absent.
    pub base: String,
    pub source: String,
    pub target: String,
    /// Whether "Add as new" applies: a picked node the target deleted.
    pub add_as_new: bool,
    /// The choice the host holds for it so far.
    pub chosen: Option<MergeOption>,
}

/// What a conflict takes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MergeOption {
    Source,
    Target,
    /// A value given by hand, as text.
    Hand(String),
    AddAsNew,
}

impl MergeOption {
    /// The `data-chosen` token.
    pub fn token(&self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Target => "target",
            Self::Hand(_) => "hand",
            Self::AddAsNew => "add-as-new",
        }
    }
}

/// Where the view puts a mere's sessions and archives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SessionPlacement {
    /// A column beside the graph, or above it when the tile is narrow.
    #[default]
    Beside,
    /// An overmap layer across the top of the graph's area, one level up
    /// from the graph: each session a container node with the archives
    /// saved from it on it, an archive shelf for the rest, and minting last.
    /// For a host whose graph is not a session's own, such as a file catalog
    /// under a resident mere.
    Overmap,
    /// An overmap tier on the graph's own canvas, one level up from the
    /// graph: each session a node across the canvas's top, the shown one
    /// marked, and each archive a node beside the session it was saved from
    /// (or with the archive's others). A session node opens in place into its
    /// card; an archive node peeks on hover and pins a popover on click. New
    /// session is a control of the tier. The graph's own nodes are laid out
    /// below the tier.
    Tier,
}

impl SessionPlacement {
    /// The `data-sessions` token.
    pub fn token(self) -> &'static str {
        match self {
            Self::Beside => "beside",
            Self::Overmap => "overmap",
            Self::Tier => "tier",
        }
    }
}

/// An immutable archive catalog. These are readings, never a second store.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArchiveModel {
    pub entries: Vec<ArchiveEntry>,
    pub save_sessions: Vec<SessionId>,
    pub can_compose: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchiveEntry {
    /// Stable archive identifier, supplied by the owner.
    pub key: String,
    pub name: String,
    pub detail: String,
    pub can_open: bool,
    /// The session this archive was saved from, when its owner says. The
    /// overmap puts the archive on that session; `None` puts it on the shelf.
    pub session: Option<SessionId>,
}

/// Whether there is a graph to show, and if not, why not.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ViewStatus {
    #[default]
    Ready,
    /// Nothing to show, such as no mere or catalog configured.
    Empty(StatusNote),
    /// The source could not be read, such as a locked reservoir.
    Unavailable(StatusNote),
    /// A reading is still being built.
    Building(StatusNote),
}

impl ViewStatus {
    /// The `data-status` token.
    pub fn token(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Empty(_) => "empty",
            Self::Unavailable(_) => "unavailable",
            Self::Building(_) => "building",
        }
    }

    pub fn note(&self) -> Option<&StatusNote> {
        match self {
            Self::Ready => None,
            Self::Empty(note) | Self::Unavailable(note) | Self::Building(note) => Some(note),
        }
    }
}

/// A plain sentence, and the action the host offers with it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusNote {
    pub message: String,
    pub action: Option<HostAction>,
}

/// One action the host offers, reported back by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAction {
    pub key: String,
    pub label: String,
}

impl HostAction {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// One of the mere's sessions.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionEntry {
    pub id: SessionId,
    pub name: String,
    /// A short line the host words, such as when it last changed.
    pub detail: Option<String>,
    /// Whether this host shows this session now.
    pub attached: bool,
    pub trashed: bool,
    /// The steps the host offers on this session now.
    pub steps: Vec<SessionStep>,
}

/// A lifecycle step on one session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SessionStep {
    Switch,
    Fork,
    /// A copy meant to merge back.
    Branch,
    /// Merge a branch or picked-off session into the session it came from.
    Merge,
    Trash,
    Restore,
}

impl SessionStep {
    /// The `data-step` token.
    pub fn token(self) -> &'static str {
        match self {
            Self::Switch => "switch",
            Self::Fork => "fork",
            Self::Branch => "branch",
            Self::Merge => "merge",
            Self::Trash => "trash",
            Self::Restore => "restore",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Switch => "Switch to",
            Self::Fork => "Fork",
            Self::Branch => "Branch",
            Self::Merge => "Merge into source",
            Self::Trash => "Trash",
            Self::Restore => "Restore",
        }
    }
}

/// The graph the host shows: a mere session's, or a catalog of its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphModel {
    pub nodes: Vec<NodeEntry>,
    pub relations: Vec<RelationEntry>,
}

/// One node, keyed by the host.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeEntry {
    /// Stable across snapshots and layouts; the node's `data-key`.
    pub key: String,
    pub label: String,
    pub state: NodeState,
}

/// What the host says of a node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeState {
    #[default]
    Available,
    Unavailable,
    Open,
    Dirty,
}

impl NodeState {
    /// The `data-state` token.
    pub fn token(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
            Self::Open => "open",
            Self::Dirty => "dirty",
        }
    }

    /// The word a node's label carries, so a state never rides on colour
    /// alone. An available node carries none.
    pub fn word(self) -> Option<&'static str> {
        match self {
            Self::Available => None,
            Self::Unavailable => Some("unavailable"),
            Self::Open => Some("open"),
            Self::Dirty => Some("unsaved"),
        }
    }
}

/// What a node on the view's canvas is, so a host colours the graph's own
/// nodes, a tier's sessions and its archives apart. [`MereView::paint_leaf`]
/// hands each node's kind to the host's palette.
///
/// [`MereView::paint_leaf`]: crate::MereView::paint_leaf
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MereNodeKind {
    /// One of the host's graph nodes, in the state the host says.
    Graph(NodeState),
    /// A session in the tier.
    Session { attached: bool, trashed: bool },
    /// An archive in the tier; `held` when it sits on the session it was
    /// saved from rather than on the shelf.
    Archive { held: bool },
    /// The tier's archive shelf, holding the archives no live session holds.
    Shelf,
    /// A "+N" node standing for a session's or the shelf's archives, past
    /// the few the tier shows one by one.
    More,
}

impl MereNodeKind {
    /// The `data-node-kind` token.
    pub fn token(self) -> &'static str {
        match self {
            Self::Graph(_) => "graph",
            Self::Session { .. } => "session",
            Self::Archive { .. } => "archive",
            Self::Shelf => "shelf",
            Self::More => "more",
        }
    }

    /// The nearest graph node state, for a host whose palette knows only
    /// those: the shown session reads as open, a trashed one as unavailable,
    /// every other tier node as available.
    pub fn state(self) -> NodeState {
        match self {
            Self::Graph(state) => state,
            Self::Session { trashed: true, .. } => NodeState::Unavailable,
            Self::Session { attached: true, .. } => NodeState::Open,
            Self::Session { .. } | Self::Archive { .. } | Self::Shelf | Self::More => {
                NodeState::Available
            },
        }
    }
}

/// The card a tier node opens in place: a session's, or the archive shelf's.
/// One is open at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TierCard {
    Session(SessionId),
    Shelf,
}

/// One relation, never merged with another on the same endpoints.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationEntry {
    /// Stable across snapshots; the relation cell's id.
    pub key: String,
    pub from: String,
    pub to: String,
    pub provenance: Provenance,
}

/// Where a relation came from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Provenance {
    /// Read out of a node's content, such as a link in a document.
    Extracted,
    /// Made by a person.
    Authored,
    /// Proposed by an engine and not yet accepted.
    Suggested,
    /// Any other kind the host names, such as a kernel relation family.
    Other(String),
}

impl Provenance {
    /// The relation cell's `data-kind` token, and the prefix of its name.
    pub fn token(&self) -> &str {
        match self {
            Self::Extracted => "extracted",
            Self::Authored => "authored",
            Self::Suggested => "suggested",
            Self::Other(kind) => kind,
        }
    }
}
