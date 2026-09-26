/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Opening files: a view asks its host for them, and the host's answer comes
//! back to that view as an event.
//!
//! [`open_file`] wraps an element. When its `requested` flag turns on, the view
//! files a [`FileRequest`] with the runner, as [`request_focus`] asks for
//! focus. The host takes the request after the dispatch that made it and shows
//! its platform's chooser: a browser's file input, or a desktop dialog. The
//! chosen files arrive at the element as a [`FileEvent`], through the same
//! message routing a click takes, and a cancelled choice arrives as an event
//! with no files, so the view can ask again. A test or a scenario supplies the
//! event directly, with no chooser.
//!
//! [`request_focus`]: crate::request_focus

use core::marker::PhantomData;

use genet_scripted_dom::NodeId;
use meristem::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};

use crate::pod::GenetElement;
use crate::{ElementView, GenetCtx, OptionalAction};

const OPEN_FILE_ID: ViewId = ViewId::new(0x4F50_4E46);

/// Which files a view offers the person.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileFilter {
    /// Extensions without their dot, such as `json`. Empty offers every file.
    pub extensions: Vec<String>,
    /// Whether the person may choose more than one.
    pub multiple: bool,
}

impl FileFilter {
    /// Offer files with these extensions.
    pub fn extensions<S: Into<String>>(extensions: impl IntoIterator<Item = S>) -> Self {
        Self {
            extensions: extensions.into_iter().map(Into::into).collect(),
            multiple: false,
        }
    }

    /// Let the person choose more than one file.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
}

/// What a view asks its host to open, and where the answer goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRequest {
    /// The element that asked. The host's [`FileEvent`] is dispatched to it.
    pub node: NodeId,
    pub filter: FileFilter,
}

/// One file the person chose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedFile {
    pub name: String,
    /// Its media type, when the platform says. A browser does; a desktop
    /// dialog does not.
    pub media_type: Option<String>,
    /// When it last changed, in milliseconds since the Unix epoch, when known.
    pub last_modified_ms: Option<u64>,
    pub bytes: Vec<u8>,
}

/// The host's answer to a [`FileRequest`]: the files chosen, or none when the
/// person cancelled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEvent {
    pub files: Vec<OpenedFile>,
}

/// Wraps an element view that opens files. See [`open_file`].
pub struct OpenFile<V, State, Action, F> {
    child: V,
    requested: bool,
    filter: FileFilter,
    handler: F,
    phantom: PhantomData<fn() -> (State, Action)>,
}

/// Ask the host for files when `requested` turns from false to true, and hand
/// its [`FileEvent`] to `handler`. Holding `requested` true does not ask again,
/// so a handler normally turns it off when the answer comes.
pub fn open_file<V, State, Action, OA, F>(
    child: V,
    requested: bool,
    filter: FileFilter,
    handler: F,
) -> OpenFile<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, FileEvent) -> OA + 'static,
{
    OpenFile {
        child,
        requested,
        filter,
        handler,
        phantom: PhantomData,
    }
}

/// Retained state for [`OpenFile`].
pub struct OpenFileState<S> {
    child_state: S,
    node: NodeId,
    path: Vec<ViewId>,
    requested: bool,
}

impl<V, State, Action, F> ViewMarker for OpenFile<V, State, Action, F> {}

impl<V, State, Action, OA, F> View<State, Action, GenetCtx> for OpenFile<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, FileEvent) -> OA + 'static,
{
    type ViewState = OpenFileState<V::ViewState>;
    type Element = GenetElement;

    fn build(&self, ctx: &mut GenetCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        ctx.with_id(OPEN_FILE_ID, |ctx| {
            let (element, child_state) = self.child.build(ctx, app_state);
            let node = element.node;
            let path = ctx.view_path().to_vec();
            ctx.register_file(node, path.clone());
            if self.requested {
                ctx.request_file(FileRequest {
                    node,
                    filter: self.filter.clone(),
                });
            }
            (
                element,
                OpenFileState {
                    child_state,
                    node,
                    path,
                    requested: self.requested,
                },
            )
        })
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        ctx.with_id(OPEN_FILE_ID, |ctx| {
            let prev_node = view_state.node;
            self.child.rebuild(
                &prev.child,
                &mut view_state.child_state,
                ctx,
                element.reborrow_mut(),
                app_state,
            );
            let node = *element.node;
            if node != prev_node || ctx.view_path() != view_state.path.as_slice() {
                ctx.unregister_file(prev_node);
                let path = ctx.view_path().to_vec();
                ctx.register_file(node, path.clone());
                view_state.node = node;
                view_state.path = path;
            }
            if self.requested && !view_state.requested {
                ctx.request_file(FileRequest {
                    node,
                    filter: self.filter.clone(),
                });
            }
            view_state.requested = self.requested;
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(OPEN_FILE_ID, |ctx| {
            ctx.unregister_file(view_state.node);
            self.child
                .teardown(&mut view_state.child_state, ctx, element);
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let Some(first) = message.take_first() else {
            return MessageResult::Stale;
        };
        if first != OPEN_FILE_ID {
            return MessageResult::Stale;
        }
        if message.remaining_path().is_empty() {
            match message.take_message::<FileEvent>() {
                Some(event) => match (self.handler)(app_state, *event).action() {
                    Some(action) => MessageResult::Action(action),
                    None => MessageResult::Nop,
                },
                None => MessageResult::Stale,
            }
        } else {
            self.child
                .message(&mut view_state.child_state, message, element, app_state)
        }
    }
}

impl<V, State, Action, OA, F> ElementView<State, Action> for OpenFile<V, State, Action, F>
where
    State: 'static,
    Action: 'static,
    V: ElementView<State, Action>,
    OA: OptionalAction<Action>,
    F: Fn(&mut State, FileEvent) -> OA + 'static,
{
}
