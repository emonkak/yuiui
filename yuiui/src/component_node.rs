use std::marker::PhantomData;
use std::{fmt, mem};

use crate::component::Component;
use crate::element::Element;
use crate::event::Lifecycle;
use crate::id::IdContext;
use crate::view_node::{CommitMode, ViewNodeMut};

pub struct ComponentNode<C: Component<S, M, E>, S, M, E> {
    component: C,
    pending_component: Option<C>,
    is_mounted: bool,
    _phantom: PhantomData<(S, M, E)>,
}

impl<C, S, M, E> ComponentNode<C, S, M, E>
where
    C: Component<S, M, E>,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            is_mounted: false,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn render(&self, state: &S, id_context: &mut IdContext) -> C::Element {
        self.component.render(state, id_context)
    }

    pub(crate) fn update(&mut self, component: C) {
        self.pending_component = Some(component);
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        view_node: ViewNodeMut<
            '_,
            <C::Element as Element<S, M, E>>::View,
            <C::Element as Element<S, M, E>>::Components,
            S,
            M,
            E,
        >,
        state: &S,
        id_context: &mut IdContext,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                let lifecycle = if self.is_mounted {
                    Lifecycle::Remount
                } else {
                    Lifecycle::Mount
                };
                self.component.lifecycle(
                    lifecycle,
                    view_node,
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                self.is_mounted = true;
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let old_component = mem::replace(&mut self.component, pending_component);
                    self.component.lifecycle(
                        Lifecycle::Update(old_component),
                        view_node,
                        state,
                        messages,
                        entry_point,
                        id_context,
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
                    state,
                    messages,
                    entry_point,
                    id_context,
                );
                true
            }
        }
    }

    pub fn component(&self) -> &C {
        &self.component
    }
}

impl<C, S, M, E> fmt::Debug for ComponentNode<C, S, M, E>
where
    C: Component<S, M, E> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}
