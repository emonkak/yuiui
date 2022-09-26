use std::fmt;
use std::marker::PhantomData;
use std::mem;

use crate::component::Component;
use crate::context::MessageContext;
use crate::element::Element;
use crate::event::Lifecycle;
use crate::state::Store;
use crate::view_node::{CommitMode, ViewRef};

pub struct ComponentNode<C: Component<S, M, B>, S, M, B> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    phantom: PhantomData<(S, M, B)>,
}

impl<C, S, M, B> ComponentNode<C, S, M, B>
where
    C: Component<S, M, B>,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            phantom: PhantomData,
        }
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        view_ref: ViewRef<'_, <C::Element as Element<S, M, B>>::View, S, M, B>,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                self.component
                    .lifecycle(Lifecycle::Mount, view_ref, context, store, backend);
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let old_component = mem::replace(&mut self.component, pending_component);
                    self.component.lifecycle(
                        Lifecycle::Update(old_component),
                        view_ref,
                        context,
                        store,
                        backend,
                    );
                    true
                } else {
                    false
                }
            }
            CommitMode::Unmount => {
                self.component
                    .lifecycle(Lifecycle::Unmount, view_ref, context, store, backend);
                true
            }
        }
    }
}

impl<C, S, M, B> fmt::Debug for ComponentNode<C, S, M, B>
where
    C: Component<S, M, B> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .finish()
    }
}
