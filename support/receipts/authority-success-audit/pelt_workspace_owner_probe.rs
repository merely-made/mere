
#![allow(dead_code,unused_imports)]
mod lowering {
use std::collections::{HashMap,HashSet};
use accesskit::{Action,Affine,HasPopup,Live,Node as AccessNode,NodeId as AccessNodeId,Orientation,Rect as AccessRect,Role,Toggled,TreeUpdate};
use inker::{A11yCapability,DocumentA11yAction,DocumentA11yNodeId,DocumentA11yProjection,DocumentA11yRole,DocumentA11yNode,DocumentA11yState,DocumentA11ySupport};
use pelt_core::{PeltSessionIdentity,WorkspaceRect};
use workbench::TileId;
/// A Pelt-owned action target for one node in the composite tree.
#[derive(Clone, Debug)]
pub(super) enum WorkspaceA11yActionTarget {
    Frisket(usize),
    Document(DocumentA11yActionTarget),
}

impl WorkspaceA11yActionTarget {
    fn is_document(&self) -> bool {
        match self {
            Self::Frisket(_) => false,
            Self::Document(_) => true,
        }
    }
}

/// The session identity and action snapshot for a renderer-neutral document
/// semantic node. The document owns semantic truth; Pelt owns namespacing,
/// tile/session validation, and its ordinary pointer route.
#[derive(Clone, Copy, Debug)]
pub(super) struct DocumentA11yActionTarget {
    pub(super) tile: TileId,
    pub(super) session_identity: PeltSessionIdentity,
    pub(super) revision: u64,
    pub(super) placement_generation: u64,
    pub(super) local_node: DocumentA11yNodeId,
    pub(super) content_rect: WorkspaceRect,
    pub(super) actions: u8,
}

impl DocumentA11yActionTarget {
    const CLICK: u8 = 1 << 0;
    const FOCUS: u8 = 1 << 1;
    const SET_VALUE: u8 = 1 << 2;
    const SCROLL_INTO_VIEW: u8 = 1 << 3;
    const INCREMENT: u8 = 1 << 4;
    const DECREMENT: u8 = 1 << 5;

    fn from_actions(actions: &[DocumentA11yAction]) -> u8 {
        actions.iter().fold(0, |bits, action| {
            bits | match action {
                DocumentA11yAction::Click => Self::CLICK,
                DocumentA11yAction::Focus => Self::FOCUS,
                DocumentA11yAction::SetValue => Self::SET_VALUE,
                DocumentA11yAction::ScrollIntoView => Self::SCROLL_INTO_VIEW,
                DocumentA11yAction::Increment => Self::INCREMENT,
                DocumentA11yAction::Decrement => Self::DECREMENT,
            }
        })
    }

    pub(super) fn supports(self, action: DocumentA11yAction) -> bool {
        let bit = match action {
            DocumentA11yAction::Click => Self::CLICK,
            DocumentA11yAction::Focus => Self::FOCUS,
            DocumentA11yAction::SetValue => Self::SET_VALUE,
            DocumentA11yAction::ScrollIntoView => Self::SCROLL_INTO_VIEW,
            DocumentA11yAction::Increment => Self::INCREMENT,
            DocumentA11yAction::Decrement => Self::DECREMENT,
        };
        self.actions & bit != 0
    }
}

/// One tile-local namespace inside Pelt's root AccessKit tree.
///
/// AccessKit node IDs are only local-tree unique. Pelt owns these assignments
/// so independently constructed `ScriptedDom`s cannot collide when their
/// retained trees become siblings below Frisket's content apertures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DocumentA11ySession {
    pub(super) identity: PeltSessionIdentity,
}

pub(super) struct DocumentA11yNamespace {
    pub(super) session_identity: PeltSessionIdentity,
    pub(super) global_ids: HashMap<AccessNodeId, AccessNodeId>,
}

impl DocumentA11yNamespace {
    fn new(session_identity: PeltSessionIdentity) -> Self {
        Self {
            session_identity,
            global_ids: HashMap::new(),
        }
    }
}

/// The tree and action map delivered together to the one-tree platform bridge.
pub(super) struct WorkspaceA11yProjection {
    pub(super) tree: TreeUpdate,
    pub(super) root: AccessNodeId,
    pub(super) actions: HashMap<AccessNodeId, WorkspaceA11yActionTarget>,
}

/// A completed renderer-neutral document projection ready to attach below one
/// Frisket content aperture.
pub(super) struct DocumentA11yChild {
    pub(super) tile: TileId,
    pub(super) session_identity: PeltSessionIdentity,
    pub(super) aperture: AccessNodeId,
    pub(super) projection: DocumentA11yProjection,
    pub(super) content_rect: WorkspaceRect,
    pub(super) placement_generation: u64,
}


struct WorkspaceAccessibility {ids:HashMap<(TileId,PeltSessionIdentity,AccessNodeId),AccessNodeId>}
impl WorkspaceAccessibility {
fn new()->Self{Self{ids:HashMap::new()}}
fn child_global_id(&mut self,t:TileId,s:PeltSessionIdentity,n:AccessNodeId,_:&HashSet<AccessNodeId>)->AccessNodeId{let next=AccessNodeId((1<<63)+self.ids.len() as u64);*self.ids.entry((t,s,n)).or_insert(next)}
fn retain_document_namespaces(&mut self,_:&HashMap<TileId,DocumentA11ySession>){}
fn clear_stale_document_focus(&mut self,_:&HashMap<AccessNodeId,WorkspaceA11yActionTarget>){}
fn child_id_is_reserved(&self,n:AccessNodeId)->bool{n.0 >= (1<<63)}
fn frisket_focus(&self)->Option<&u64>{None}
}
struct FrisketA11yProjection{tree:TreeUpdate,root:AccessNodeId,nodes:HashMap<AccessNodeId,usize>,content_nodes:HashMap<TileId,AccessNodeId>}
struct FrisketFixture;
impl FrisketFixture {fn accessibility_projection(&self,_:Option<&u64>)->Option<FrisketA11yProjection>{Some(shell())}}
fn shell()->FrisketA11yProjection {
let root=AccessNodeId(1);let aperture=AccessNodeId(2);let mut top=AccessNode::new(Role::Window);top.set_children([aperture]);
FrisketA11yProjection{tree:TreeUpdate{tree:Some(accesskit::Tree::new(root)),nodes:vec![(root,top),(aperture,AccessNode::new(Role::Region))],tree_id:accesskit::TreeId::ROOT,focus:root},root,nodes:HashMap::new(),content_nodes:HashMap::from([(TileId(1),aperture)])}
}
struct WorkspaceApp{accessibility:WorkspaceAccessibility,frisket:FrisketFixture}
impl WorkspaceApp {
fn document_accessibility_children(&self,_:&FrisketA11yProjection)->(HashMap<TileId,DocumentA11ySession>,Vec<DocumentA11yChild>){(HashMap::new(),vec![child()])}
    fn accesskit_role(role: DocumentA11yRole) -> Role {
        match role {
            DocumentA11yRole::Window => Role::Window,
            DocumentA11yRole::Document => Role::Document,
            DocumentA11yRole::Article => Role::Article,
            DocumentA11yRole::Region => Role::Region,
            DocumentA11yRole::Group => Role::Group,
            DocumentA11yRole::Navigation => Role::Navigation,
            DocumentA11yRole::Main => Role::Main,
            DocumentA11yRole::Heading { .. } => Role::Heading,
            DocumentA11yRole::Paragraph => Role::Paragraph,
            DocumentA11yRole::StaticText => Role::TextRun,
            DocumentA11yRole::Link => Role::Link,
            DocumentA11yRole::Button => Role::Button,
            DocumentA11yRole::TextField => Role::TextInput,
            DocumentA11yRole::CheckBox => Role::CheckBox,
            DocumentA11yRole::RadioButton => Role::RadioButton,
            DocumentA11yRole::RadioGroup => Role::RadioGroup,
            DocumentA11yRole::Switch => Role::Switch,
            DocumentA11yRole::ComboBox => Role::ComboBox,
            DocumentA11yRole::List => Role::List,
            DocumentA11yRole::ListItem => Role::ListItem,
            DocumentA11yRole::ListBox => Role::ListBox,
            DocumentA11yRole::ListBoxOption => Role::ListBoxOption,
            DocumentA11yRole::Table => Role::Table,
            DocumentA11yRole::Row => Role::Row,
            DocumentA11yRole::Cell => Role::Cell,
            DocumentA11yRole::Image => Role::Image,
            DocumentA11yRole::Form => Role::Form,
            DocumentA11yRole::Dialog => Role::Dialog,
            DocumentA11yRole::Alert => Role::Alert,
            DocumentA11yRole::Menu => Role::Menu,
            DocumentA11yRole::MenuItem => Role::MenuItem,
            DocumentA11yRole::MenuItemCheckBox => Role::MenuItemCheckBox,
            DocumentA11yRole::MenuItemRadio => Role::MenuItemRadio,
            DocumentA11yRole::TabList => Role::TabList,
            DocumentA11yRole::Tab => Role::Tab,
            DocumentA11yRole::TabPanel => Role::TabPanel,
            DocumentA11yRole::Tree => Role::Tree,
            DocumentA11yRole::TreeItem => Role::TreeItem,
            DocumentA11yRole::Slider => Role::Slider,
            DocumentA11yRole::SpinButton => Role::SpinButton,
            DocumentA11yRole::Splitter => Role::Splitter,
            DocumentA11yRole::Toolbar => Role::Toolbar,
            DocumentA11yRole::ProgressIndicator => Role::ProgressIndicator,
            DocumentA11yRole::Label => Role::Label,
            DocumentA11yRole::Status => Role::Status,
            DocumentA11yRole::Log => Role::Log,
            DocumentA11yRole::Note => Role::Note,
            DocumentA11yRole::Unknown => Role::GenericContainer,
        }
    }

    fn accesskit_actions(actions: &[DocumentA11yAction], node: &mut AccessNode) {
        for action in actions {
            node.add_action(match action {
                DocumentA11yAction::Click => Action::Click,
                DocumentA11yAction::Focus => Action::Focus,
                DocumentA11yAction::SetValue => Action::SetValue,
                DocumentA11yAction::ScrollIntoView => Action::ScrollIntoView,
                DocumentA11yAction::Increment => Action::Increment,
                DocumentA11yAction::Decrement => Action::Decrement,
            });
        }
    }

    pub fn workspace_accessibility_projection(
        &mut self,
    ) -> Result<WorkspaceA11yProjection, String> {
        let frisket_focus = self.accessibility.frisket_focus().cloned();
        let shell = self
            .frisket
            .accessibility_projection(frisket_focus.as_ref())
            .ok_or_else(|| {
                "Frisket has no completed retained layout for accessibility".to_owned()
            })?;
        let (live_sessions, children) = self.document_accessibility_children(&shell);
        self.accessibility
            .retain_document_namespaces(&live_sessions);
        let shell_ids = shell
            .tree
            .nodes
            .iter()
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>();
        let FrisketA11yProjection {
            mut tree,
            root,
            nodes,
            ..
        } = shell;
        let mut actions = nodes
            .into_iter()
            .filter(|(id, _)| !self.accessibility.child_id_is_reserved(*id))
            .map(|(id, node)| (id, WorkspaceA11yActionTarget::Frisket(node)))
            .collect::<HashMap<_, _>>();

        for child in children {
            let DocumentA11yChild {
                tile,
                session_identity,
                aperture,
                projection,
                content_rect,
                placement_generation,
            } = child;
            let local_ids = projection
                .nodes()
                .iter()
                .map(|node| {
                    let local = AccessNodeId(node.id.get());
                    let global = self.accessibility.child_global_id(
                        tile,
                        session_identity,
                        local,
                        &shell_ids,
                    );
                    (node.id, global)
                })
                .collect::<HashMap<_, _>>();
            let Some(&child_root) = local_ids.get(&projection.root()) else {
                continue;
            };
            let Some((_, aperture_node)) = tree.nodes.iter_mut().find(|(id, _)| *id == aperture)
            else {
                continue;
            };
            aperture_node.set_children([child_root]);
            for semantic in projection.nodes() {
                let Some(&global_id) = local_ids.get(&semantic.id) else {
                    continue;
                };
                let mut node = AccessNode::new(Self::accesskit_role(semantic.role));
                if let DocumentA11yRole::Heading { level } = semantic.role
                    && level > 0
                {
                    node.set_level(usize::from(level));
                }
                if let Some(name) = &semantic.name {
                    node.set_label(name.clone());
                }
                if let Some(value) = &semantic.value {
                    node.set_value(value.clone());
                }
                if semantic.state.disabled {
                    node.set_disabled();
                }
                if semantic.state.hidden {
                    node.set_hidden();
                }
                if semantic.state.read_only {
                    node.set_read_only();
                }
                if semantic.state.required {
                    node.set_required();
                }
                if let Some(selected) = semantic.state.selected {
                    node.set_selected(selected);
                }
                if let Some(expanded) = semantic.state.expanded {
                    node.set_expanded(expanded);
                }
                if let Some(toggled) = semantic.state.toggled {
                    node.set_toggled(match toggled {
                        inker::DocumentA11yToggled::On => Toggled::True,
                        inker::DocumentA11yToggled::Off => Toggled::False,
                        inker::DocumentA11yToggled::Mixed => Toggled::Mixed,
                    });
                } else if let Some(checked) = semantic.state.checked {
                    node.set_toggled(if checked {
                        Toggled::True
                    } else {
                        Toggled::False
                    });
                }
                if let Some(live) = semantic.state.live {
                    node.set_live(match live {
                        inker::DocumentA11yLive::Off => Live::Off,
                        inker::DocumentA11yLive::Polite => Live::Polite,
                        inker::DocumentA11yLive::Assertive => Live::Assertive,
                    });
                }
                if let Some(orientation) = semantic.state.orientation {
                    node.set_orientation(match orientation {
                        inker::DocumentA11yOrientation::Horizontal => Orientation::Horizontal,
                        inker::DocumentA11yOrientation::Vertical => Orientation::Vertical,
                    });
                }
                if let Some(has_popup) = semantic.state.has_popup {
                    node.set_has_popup(match has_popup {
                        inker::DocumentA11yHasPopup::Menu => HasPopup::Menu,
                        inker::DocumentA11yHasPopup::ListBox => HasPopup::Listbox,
                        inker::DocumentA11yHasPopup::Tree => HasPopup::Tree,
                        inker::DocumentA11yHasPopup::Grid => HasPopup::Grid,
                        inker::DocumentA11yHasPopup::Dialog => HasPopup::Dialog,
                    });
                }
                if let Some(value) = semantic.numeric_value {
                    node.set_numeric_value(value);
                }
                if let Some(value) = semantic.numeric_minimum {
                    node.set_min_numeric_value(value);
                }
                if let Some(value) = semantic.numeric_maximum {
                    node.set_max_numeric_value(value);
                }
                if let Some(bounds) = semantic.bounds {
                    node.set_bounds(AccessRect::new(
                        f64::from(bounds.x),
                        f64::from(bounds.y),
                        f64::from(bounds.x + bounds.width),
                        f64::from(bounds.y + bounds.height),
                    ));
                }
                let descendants = semantic
                    .children
                    .iter()
                    .filter_map(|child| local_ids.get(child).copied())
                    .collect::<Vec<_>>();
                node.set_children(descendants);
                if semantic.id == projection.root() {
                    node.set_transform(Affine::translate((
                        f64::from(content_rect.x),
                        f64::from(content_rect.y),
                    )));
                }
                Self::accesskit_actions(&semantic.actions, &mut node);
                if semantic.state.focused {
                    tree.focus = global_id;
                }
                let action_bits = DocumentA11yActionTarget::from_actions(&semantic.actions);
                if action_bits != 0 {
                    actions.insert(
                        global_id,
                        WorkspaceA11yActionTarget::Document(DocumentA11yActionTarget {
                            tile,
                            session_identity,
                            revision: projection.revision(),
                            placement_generation,
                            local_node: semantic.id,
                            content_rect,
                            actions: action_bits,
                        }),
                    );
                }
                tree.nodes.push((global_id, node));
            }
        }
        self.accessibility.clear_stale_document_focus(&actions);
        Ok(WorkspaceA11yProjection {
            tree,
            root,
            actions,
        })
    }


}
fn append_document_child(
    accessibility: &mut WorkspaceAccessibility,
    tree: &mut TreeUpdate,
    actions: &mut HashMap<AccessNodeId, WorkspaceA11yActionTarget>,
    shell_ids: &HashSet<AccessNodeId>,
    child: DocumentA11yChild,
) {
    let DocumentA11yChild {
        tile,
        session_identity,
        aperture,
        projection,
        content_rect,
        placement_generation,
    } = child;
    let local_ids = projection
        .nodes()
        .iter()
        .map(|node| {
            let local = AccessNodeId(node.id.get());
            let global = accessibility.child_global_id(tile, session_identity, local, shell_ids);
            (node.id, global)
        })
        .collect::<HashMap<_, _>>();
    let Some(&child_root) = local_ids.get(&projection.root()) else {
        return;
    };
    let Some((_, aperture_node)) = tree.nodes.iter_mut().find(|(id, _)| *id == aperture) else {
        return;
    };
    aperture_node.set_children([child_root]);
    for semantic in projection.nodes() {
        let Some(&global_id) = local_ids.get(&semantic.id) else {
            continue;
        };
        let mut node = AccessNode::new(WorkspaceApp::accesskit_role(semantic.role));
        if let DocumentA11yRole::Heading { level } = semantic.role
            && level > 0
        {
            node.set_level(usize::from(level));
        }
        if let Some(name) = &semantic.name {
            node.set_label(name.clone());
        }
        if let Some(value) = &semantic.value {
            node.set_value(value.clone());
        }
        if semantic.state.disabled {
            node.set_disabled();
        }
        if semantic.state.hidden {
            node.set_hidden();
        }
        if semantic.state.read_only {
            node.set_read_only();
        }
        if semantic.state.required {
            node.set_required();
        }
        if let Some(selected) = semantic.state.selected {
            node.set_selected(selected);
        }
        if let Some(expanded) = semantic.state.expanded {
            node.set_expanded(expanded);
        }
        if let Some(toggled) = semantic.state.toggled {
            node.set_toggled(match toggled {
                inker::DocumentA11yToggled::On => Toggled::True,
                inker::DocumentA11yToggled::Off => Toggled::False,
                inker::DocumentA11yToggled::Mixed => Toggled::Mixed,
            });
        } else if let Some(checked) = semantic.state.checked {
            node.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        }
        if let Some(live) = semantic.state.live {
            node.set_live(match live {
                inker::DocumentA11yLive::Off => Live::Off,
                inker::DocumentA11yLive::Polite => Live::Polite,
                inker::DocumentA11yLive::Assertive => Live::Assertive,
            });
        }
        if let Some(orientation) = semantic.state.orientation {
            node.set_orientation(match orientation {
                inker::DocumentA11yOrientation::Horizontal => Orientation::Horizontal,
                inker::DocumentA11yOrientation::Vertical => Orientation::Vertical,
            });
        }
        if let Some(has_popup) = semantic.state.has_popup {
            node.set_has_popup(match has_popup {
                inker::DocumentA11yHasPopup::Menu => HasPopup::Menu,
                inker::DocumentA11yHasPopup::ListBox => HasPopup::Listbox,
                inker::DocumentA11yHasPopup::Tree => HasPopup::Tree,
                inker::DocumentA11yHasPopup::Grid => HasPopup::Grid,
                inker::DocumentA11yHasPopup::Dialog => HasPopup::Dialog,
            });
        }
        if let Some(value) = semantic.numeric_value {
            node.set_numeric_value(value);
        }
        if let Some(value) = semantic.numeric_minimum {
            node.set_min_numeric_value(value);
        }
        if let Some(value) = semantic.numeric_maximum {
            node.set_max_numeric_value(value);
        }
        if let Some(bounds) = semantic.bounds {
            node.set_bounds(AccessRect::new(
                f64::from(bounds.x),
                f64::from(bounds.y),
                f64::from(bounds.x + bounds.width),
                f64::from(bounds.y + bounds.height),
            ));
        }
        let descendants = semantic
            .children
            .iter()
            .filter_map(|child| local_ids.get(child).copied())
            .collect::<Vec<_>>();
        node.set_children(descendants);
        if semantic.id == projection.root() {
            node.set_transform(Affine::translate((
                f64::from(content_rect.x),
                f64::from(content_rect.y),
            )));
        }
        WorkspaceApp::accesskit_actions(&semantic.actions, &mut node);
        if semantic.state.focused {
            tree.focus = global_id;
        }
        let action_bits = DocumentA11yActionTarget::from_actions(&semantic.actions);
        if action_bits != 0 {
            actions.insert(
                global_id,
                WorkspaceA11yActionTarget::Document(DocumentA11yActionTarget {
                    tile,
                    session_identity,
                    revision: projection.revision(),
                    placement_generation,
                    local_node: semantic.id,
                    content_rect,
                    actions: action_bits,
                }),
            );
        }
        tree.nodes.push((global_id, node));
    }
}


fn child()->DocumentA11yChild {
DocumentA11yChild {tile:TileId(1),session_identity:PeltSessionIdentity{instance_id:17,generation:1},aperture:AccessNodeId(2),content_rect:WorkspaceRect::new(10.0,20.0,400.0,300.0),placement_generation:1,
projection:DocumentA11yProjection::new(1,DocumentA11ySupport::new(A11yCapability::Partial,["fixture"]).unwrap(),DocumentA11yNodeId::new(1),vec![DocumentA11yNode {
id:DocumentA11yNodeId::new(1),parent:None,children:vec![],role:DocumentA11yRole::TextField,name:Some("Account".into()),description:Some("Use your public account name".into()),value:Some("orchard".into()),numeric_value:None,numeric_minimum:None,numeric_maximum:None,bounds:None,state:DocumentA11yState::default(),actions:vec![DocumentA11yAction::SetValue],}])}
}
fn assert_dropped(tree:&TreeUpdate){let (_,node)=tree.nodes.iter().find(|(_,n)|n.label()==Some("Account")).unwrap();assert_eq!(node.role(),Role::TextInput);assert_eq!(node.value(),Some("orchard"));assert!(node.supports_action(Action::SetValue));assert_eq!(node.description(),None);}
#[test]fn secondary_helper_drops_real_neutral_description(){let c=child();assert_eq!(c.projection.nodes()[0].description.as_deref(),Some("Use your public account name"));let mut s=shell();let mut a=WorkspaceAccessibility::new();let mut actions=HashMap::new();let ids=s.tree.nodes.iter().map(|(id,_)|*id).collect();append_document_child(&mut a,&mut s.tree,&mut actions,&ids,c);assert_dropped(&s.tree);}
#[test]fn copied_primary_lowering_drops_real_neutral_description(){let mut a=WorkspaceApp{accessibility:WorkspaceAccessibility::new(),frisket:FrisketFixture};assert_dropped(&a.workspace_accessibility_projection().unwrap().tree);}
#[test]fn accesskit_can_retain_supplemental_description_control(){let mut n=AccessNode::new(Role::TextInput);n.set_description("Use your public account name");assert_eq!(n.description(),Some("Use your public account name"));}
}
mod receipt_guards {
#[derive(Clone,Copy,PartialEq)]enum WorkspaceReceipt{Mixed,Other}
struct Config{workspace_receipt:Option<WorkspaceReceipt>,interaction_receipt:bool,capability_receipt:bool,tearout_receipt:bool,tearout_cancellation_receipt:bool,urls:Vec<String>,workspace_size_matrix:Option<Vec<(u32,u32)>>}
fn validate(config:&Config)->Result<(),String>{
    if config.workspace_receipt.is_some()
        && (config.interaction_receipt
            || config.capability_receipt
            || config.tearout_receipt
            || config.tearout_cancellation_receipt)
    {
        return Err(
            "a named workspace receipt cannot be combined with the P3, P4, or W4 receipt driver"
                .to_owned(),
        );
    }
    if config.tearout_receipt && config.tearout_cancellation_receipt {
        return Err("W4 acceptance and cancellation receipts are separate runs".to_owned());
    }
    if (config.tearout_receipt || config.tearout_cancellation_receipt) && config.urls.len() < 2 {
        return Err(
            "W4 tearout receipt needs a sibling source tile to keep the primary host live"
                .to_owned(),
        );
    }
    if let Some(sizes) = config.workspace_size_matrix.as_deref() {
        if config.workspace_receipt != Some(WorkspaceReceipt::Mixed) {
            return Err("a workspace size matrix requires the mixed workspace receipt".to_owned());
        }
        if sizes.len() < 2
            || sizes
                .iter()
                .any(|&(width, height)| width == 0 || height == 0)
            || sizes
                .iter()
                .enumerate()
                .any(|(index, size)| sizes[..index].contains(size))
        {
            return Err(
                "a workspace size matrix needs at least two unique positive physical sizes"
                    .to_owned(),
            );
        }
    }

Ok(())}
struct ActiveEventLoop;impl ActiveEventLoop{fn exit(&self){}}
struct WorkspaceApp{config:Config,redraws:u32,receipt_complete:bool,receipt_error:Option<String>,pending_workspace_assertion:Option<String>,cap_ready:bool,interaction_completes:bool,cap_calls:usize,interaction_calls:usize,tearout_calls:usize,cancel_calls:usize}
impl WorkspaceApp {
fn capability_receipt_ready(&self)->bool{self.cap_ready}
fn validate_capability_receipt(&mut self)->Result<(),String>{self.cap_calls+=1;Ok(())}
fn drive_receipt_step(&mut self)->Result<(),String>{self.interaction_calls+=1;if self.interaction_completes {self.receipt_complete=true;}Ok(())}
fn drive_workspace_receipt(&mut self)->Result<Option<String>,String>{Ok(None)}
fn drive_tearout_receipt(&mut self,_:&ActiveEventLoop)->Result<bool,String>{self.tearout_calls+=1;Ok(false)}
fn drive_tearout_cancellation_receipt(&mut self,_:&ActiveEventLoop)->Result<bool,String>{self.cancel_calls+=1;Ok(false)}
fn run_drivers(&mut self,event_loop:&ActiveEventLoop){
        if self.config.workspace_receipt.is_some() && self.redraws > 0 && !self.receipt_complete {
            match self.drive_workspace_receipt() {
                Ok(Some(assertion)) => {
                    self.pending_workspace_assertion = Some(assertion);
                    self.receipt_complete = true;
                },
                Ok(None) => {},
                Err(error) => {
                    self.receipt_error = Some(error);
                    event_loop.exit();
                    return;
                },
            }
        }
        if self.config.capability_receipt
            && self.redraws > 0
            && !self.receipt_complete
            && self.capability_receipt_ready()
        {
            if let Err(error) = self.validate_capability_receipt() {
                self.receipt_error = Some(error);
                event_loop.exit();
                return;
            }
            self.receipt_complete = true;
        }
        if self.config.interaction_receipt && self.redraws > 0 && !self.receipt_complete {
            if let Err(error) = self.drive_receipt_step() {
                self.receipt_error = Some(error);
                event_loop.exit();
                return;
            }
        }
        if self.config.tearout_receipt && self.redraws > 0 && !self.receipt_complete {
            match self.drive_tearout_receipt(event_loop) {
                Ok(true) => self.receipt_complete = true,
                Ok(false) => {},
                Err(error) => {
                    self.receipt_error = Some(error);
                    event_loop.exit();
                    return;
                },
            }
        }
        if self.config.tearout_cancellation_receipt && self.redraws > 0 && !self.receipt_complete {
            match self.drive_tearout_cancellation_receipt(event_loop) {
                Ok(true) => self.receipt_complete = true,
                Ok(false) => {},
                Err(error) => {
                    self.receipt_error = Some(error);
                    event_loop.exit();
                    return;
                },
            }
        }

}}
fn app()->WorkspaceApp {WorkspaceApp{config:Config{workspace_receipt:None,interaction_receipt:false,capability_receipt:false,tearout_receipt:false,tearout_cancellation_receipt:false,urls:vec!["a".into(),"b".into(),"c".into(),"d".into()],workspace_size_matrix:None},redraws:1,receipt_complete:false,receipt_error:None,pending_workspace_assertion:None,cap_ready:false,interaction_completes:false,cap_calls:0,interaction_calls:0,tearout_calls:0,cancel_calls:0}}
#[test]fn admitted_capability_completion_skips_requested_interaction(){let mut a=app();a.config.interaction_receipt=true;a.config.capability_receipt=true;a.cap_ready=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(a.receipt_complete);assert_eq!(a.cap_calls,1);assert_eq!(a.interaction_calls,0);}
#[test]fn capability_pending_control_runs_interaction(){let mut a=app();a.config.interaction_receipt=true;a.config.capability_receipt=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(!a.receipt_complete);assert_eq!(a.interaction_calls,1);}
#[test]fn admitted_interaction_completion_skips_requested_tearout(){let mut a=app();a.config.interaction_receipt=true;a.config.tearout_receipt=true;a.interaction_completes=true;assert!(validate(&a.config).is_ok());a.run_drivers(&ActiveEventLoop);assert!(a.receipt_complete);assert_eq!(a.interaction_calls,1);assert_eq!(a.tearout_calls,0);}
#[test]fn named_and_dual_tearout_validation_controls_refuse(){let mut a=app();a.config.workspace_receipt=Some(WorkspaceReceipt::Other);a.config.interaction_receipt=true;assert!(validate(&a.config).is_err());a.config.workspace_receipt=None;a.config.tearout_receipt=true;a.config.tearout_cancellation_receipt=true;assert!(validate(&a.config).is_err());}
}
