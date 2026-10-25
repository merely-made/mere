// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Browser navigation policy used by Turnstone and Graphshell. Applications
//! outside this graph browser family supply their own identities and routes.
//! An explicit contextual route can nest neighborhoods or skip default levels.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeLevel {
    Document,
    /// An explicitly entered arrangement, never an implicit graph parent.
    Forme(uuid::Uuid),
    Neighborhood,
    #[default]
    Graph,
    Overmap,
}
impl ScopeLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Forme(_) => "Forme",
            Self::Neighborhood => "Neighborhood",
            Self::Graph => "Graph",
            Self::Overmap => "Meres",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeContext<G, M> {
    pub level: ScopeLevel,
    pub graph: G,
    pub root: Option<M>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopePlace<G, M> {
    pub level: ScopeLevel,
    pub graph: G,
    pub root: Option<M>,
    pub focus: Option<M>,
    /// Outer-to-inner parents. None admits legacy browser defaults; Some([])
    /// explicitly names a root. Context identity stays separate from focus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<ScopeContext<G, M>>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeDestination<G, M> {
    Level(ScopeLevel),
    Place(ScopePlace<G, M>),
}
impl<G, M> From<ScopeLevel> for ScopeDestination<G, M> {
    fn from(level: ScopeLevel) -> Self {
        Self::Level(level)
    }
}
impl<G: Clone + PartialEq, M: Clone + PartialEq> ScopePlace<G, M> {
    fn context(&self) -> ScopeContext<G, M> {
        ScopeContext {
            level: self.level,
            graph: self.graph.clone(),
            root: if self.level == ScopeLevel::Neighborhood {
                self.root.clone()
            } else {
                None
            },
        }
    }
    pub fn parents(&self) -> Vec<ScopeContext<G, M>> {
        self.contexts.clone().unwrap_or_else(|| {
            let levels: &[ScopeLevel] = match self.level {
                ScopeLevel::Document => &[
                    ScopeLevel::Overmap,
                    ScopeLevel::Graph,
                    ScopeLevel::Neighborhood,
                ],
                ScopeLevel::Neighborhood => &[ScopeLevel::Overmap, ScopeLevel::Graph],
                ScopeLevel::Forme(_) => &[ScopeLevel::Overmap, ScopeLevel::Graph],
                ScopeLevel::Graph => &[ScopeLevel::Overmap],
                ScopeLevel::Overmap => &[],
            };
            levels
                .iter()
                .map(|level| ScopeContext {
                    level: *level,
                    graph: self.graph.clone(),
                    root: if *level == ScopeLevel::Neighborhood {
                        self.root.clone().or_else(|| self.focus.clone())
                    } else {
                        None
                    },
                })
                .collect()
        })
    }
    pub fn up(&self) -> Option<Self> {
        let mut contexts = self.parents();
        let parent = contexts.pop()?;
        Some(Self {
            level: parent.level,
            graph: parent.graph,
            root: parent.root,
            focus: self.focus.clone(),
            contexts: Some(contexts),
        })
    }
    pub fn enter(&self, level: ScopeLevel) -> Self {
        let mut contexts = self.parents();
        contexts.push(self.context());
        Self {
            level,
            root: self.root.clone().or_else(|| self.focus.clone()),
            contexts: Some(contexts),
            ..self.clone()
        }
    }
    pub fn down(&self) -> Option<Self> {
        let level = match self.level {
            ScopeLevel::Overmap => ScopeLevel::Graph,
            ScopeLevel::Graph if self.root.is_some() || self.focus.is_some() => {
                ScopeLevel::Neighborhood
            },
            ScopeLevel::Neighborhood if self.focus.is_some() => ScopeLevel::Document,
            ScopeLevel::Forme(_) if self.focus.is_some() => ScopeLevel::Document,
            _ => return None,
        };
        Some(self.enter(level))
    }
    pub fn select(&mut self, focus: M) {
        if self.focus.as_ref() != Some(&focus) && self.level == ScopeLevel::Graph {
            self.root = Some(focus.clone());
        }
        self.focus = Some(focus);
    }
    pub fn same_scope(&self, other: &Self) -> bool {
        self.level == other.level
            && self.graph == other.graph
            && self.parents() == other.parents()
            && match self.level {
                ScopeLevel::Document => self.focus == other.focus,
                ScopeLevel::Neighborhood => self.root == other.root,
                _ => true,
            }
    }
    pub fn valid(&self) -> bool {
        (!matches!(self.level, ScopeLevel::Document | ScopeLevel::Neighborhood)
            || self.focus.is_some())
            && self.contexts.as_ref().is_none_or(|path| {
                path.len() <= 64
                    && path.iter().all(|p| {
                        p.level != ScopeLevel::Document
                            && (p.level != ScopeLevel::Neighborhood || p.root.is_some())
                    })
            })
    }
    pub fn trail(&self) -> Vec<Self> {
        let mut trail = vec![self.clone()];
        let mut current = self.clone();
        while let Some(parent) = current.up() {
            trail.push(parent.clone());
            current = parent;
        }
        trail.reverse();
        trail
    }
    /// Reconcile an external projection change with its real ancestor route.
    /// A graph projection cannot keep a document's child contexts as parents.
    pub fn project(&self, level: ScopeLevel, root: Option<M>, focus: Option<M>) -> Self {
        let mut place = self
            .trail()
            .into_iter()
            .rev()
            .find(|p| p.level == level && (level != ScopeLevel::Neighborhood || p.root == root))
            .unwrap_or(Self {
                level,
                graph: self.graph.clone(),
                root: root.clone(),
                focus: focus.clone(),
                contexts: None,
            });
        place.focus = focus;
        if level == ScopeLevel::Neighborhood {
            place.root = root;
        }
        place
    }
    pub fn at_level(&self, level: ScopeLevel) -> Option<Self> {
        if self.level == level {
            return Some(self.clone());
        }
        if let Some(parent) = self.trail().into_iter().rev().find(|p| p.level == level) {
            return Some(parent);
        }
        let mut child = self.clone();
        while let Some(next) = child.down() {
            if next.level == level {
                return Some(next);
            }
            child = next;
        }
        None
    }
    pub fn reading(&self, graph: G, member: M) -> Self {
        if self.graph == graph && matches!(self.level, ScopeLevel::Neighborhood | ScopeLevel::Forme(_)) {
            let mut next = self.enter(ScopeLevel::Document);
            next.focus = Some(member);
            return next;
        }
        if self.graph == graph && self.level == ScopeLevel::Document {
            return Self {
                focus: Some(member),
                ..self.clone()
            };
        }
        if self.graph == graph && self.level == ScopeLevel::Graph {
            let mut graph = self.clone();
            graph.select(member);
            return graph
                .enter(ScopeLevel::Neighborhood)
                .enter(ScopeLevel::Document);
        }
        Self {
            level: ScopeLevel::Document,
            graph,
            root: Some(member.clone()),
            focus: Some(member),
            contexts: None,
        }
    }
}
impl<G: Clone + PartialEq, M: Clone + PartialEq> ScopeDestination<G, M> {
    pub fn resolve(&self, origin: &ScopePlace<G, M>) -> Option<ScopePlace<G, M>> {
        match self {
            Self::Level(level) => origin.at_level(*level),
            Self::Place(place) => place.valid().then(|| place.clone()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn doc() -> ScopePlace<u8, u8> {
        ScopePlace {
            level: ScopeLevel::Document,
            graph: 1,
            root: Some(2),
            focus: Some(3),
            contexts: None,
        }
    }
    #[test]
    fn forme_is_an_explicit_context_and_does_not_change_default_graph_navigation() {
        let graph=ScopePlace {level:ScopeLevel::Graph,graph:1u8,root:Some(2u8),focus:Some(3),contexts:None};
        assert_eq!(graph.down().unwrap().level,ScopeLevel::Neighborhood);
        let a=graph.enter(ScopeLevel::Forme(uuid::Uuid::from_u128(40)));
        let b=graph.enter(ScopeLevel::Forme(uuid::Uuid::from_u128(41)));
        assert!(!a.same_scope(&b));
        let document=a.reading(1,3);
        assert!(document.valid());
        assert_eq!(document.up().unwrap().level,a.level);
        assert_eq!(document.up().unwrap().up().unwrap().level,ScopeLevel::Graph);
        assert_eq!(document.trail().iter().map(|p|p.level).collect::<Vec<_>>(),vec![ScopeLevel::Overmap,ScopeLevel::Graph,a.level,ScopeLevel::Document]);
        let encoded=serde_json::to_string(&document).unwrap();
        assert_eq!(serde_json::from_str::<ScopePlace<u8,u8>>(&encoded).unwrap(),document);
    }
    #[test]
    fn contextual_up_preserves_nested_roots_and_down_enters_current_focus() {
        let mut place = doc();
        place.root=Some(4);
        place.contexts = Some(vec![
            ScopeContext {
                level: ScopeLevel::Graph,
                graph: 1,
                root: Some(2),
            },
            ScopeContext {
                level: ScopeLevel::Neighborhood,
                graph: 1,
                root: Some(2),
            },
            ScopeContext {
                level: ScopeLevel::Neighborhood,
                graph: 1,
                root: Some(4),
            },
        ]);
        let inner = place.up().unwrap();
        assert_eq!(inner.root, Some(4));
        assert_eq!(inner.focus, Some(3));
        let outer = inner.up().unwrap();
        assert_eq!(outer.root, Some(2));
        assert_eq!(outer.down().unwrap().focus, Some(3));
        assert_eq!(inner.down().unwrap(), place);
    }
    #[test]
    fn a_route_can_skip_levels_and_the_same_document_can_have_multiple_contexts() {
        let mut a = doc();
        a.contexts = Some(vec![ScopeContext {
            level: ScopeLevel::Graph,
            graph: 1,
            root: Some(2),
        }]);
        assert_eq!(a.up().unwrap().level, ScopeLevel::Graph);
        assert_eq!(a.trail().len(), 2);
        let mut b = a.clone();
        b.contexts.as_mut().unwrap()[0].root = Some(4);
        assert!(!a.same_scope(&b));
        assert_eq!(a.focus, b.focus);
    }
    #[test]
    fn changing_projection_uses_the_real_ancestor_and_rejects_incomplete_contexts() {
        let place = doc();
        let graph = place.project(ScopeLevel::Graph, None, Some(3));
        assert_eq!(graph.trail().len(), 2);
        assert_eq!(graph.up().unwrap().level, ScopeLevel::Overmap);
        let mut invalid = doc();
        invalid.contexts = Some(vec![ScopeContext {
            level: ScopeLevel::Neighborhood,
            graph: 1,
            root: None,
        }]);
        assert!(!invalid.valid());
    }
    #[test]
    fn legacy_routes_have_browser_defaults_without_serializing_new_fields() {
        let old = r#"{"level":"Document","graph":1,"root":2,"focus":3}"#;
        let place: ScopePlace<u8, u8> = serde_json::from_str(old).unwrap();
        assert_eq!(place.up().unwrap().level, ScopeLevel::Neighborhood);
        assert!(!serde_json::to_string(&place).unwrap().contains("contexts"));
    }
}
