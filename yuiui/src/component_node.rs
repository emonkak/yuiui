use std::marker::PhantomData;
use std::{fmt, mem};

use crate::component::Component;
use crate::element::Element;
use crate::event::Lifecycle;
use crate::id::{Depth, IdContext};
use crate::store::Store;
use crate::view_node::{CommitMode, ViewNodeMut};

pub struct ComponentNode<C: Component<S, M, B>, S, M, B> {
    component: C,
    depth: Depth,
    pending_component: Option<C>,
    is_mounted: bool,
    _phantom: PhantomData<(S, M, B)>,
}

impl<C, S, M, B> ComponentNode<C, S, M, B>
where
    C: Component<S, M, B>,
{
    pub(crate) fn new(component: C, depth: Depth) -> Self {
        Self {
            component,
            depth,
            pending_component: None,
            is_mounted: false,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn update(&mut self, component: C) {
        self.pending_component = Some(component);
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        view_node: ViewNodeMut<
            '_,
            <C::Element as Element<S, M, B>>::View,
            <C::Element as Element<S, M, B>>::Components,
            S,
            M,
            B,
        >,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &mut B,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                let lifecycle = if self.is_mounted {
                    Lifecycle::Remount
                } else {
                    Lifecycle::Mount
                };
                self.component
                    .lifecycle(lifecycle, view_node, id_context, store, messages, backend);
                self.is_mounted = true;
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let old_component = mem::replace(&mut self.component, pending_component);
                    self.component.lifecycle(
                        Lifecycle::Update(old_component),
                        view_node,
                        id_context,
                        store,
                        messages,
                        backend,
                    );
                    true
                } else {
                    false
                }
            }
            CommitMode::Unmount => {
                self.component.lifecycle(
                    Lifecycle::Unmount,
                    view_node,
                    id_context,
                    store,
                    messages,
                    backend,
                );
                true
            }
        }
    }

    pub fn component(&self) -> &C {
        &self.component
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }
}

impl<C, S, M, B> fmt::Debug for ComponentNode<C, S, M, B>
where
    C: Component<S, M, B> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("depth", &self.depth)
            .field("pending_component", &self.pending_component)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}
