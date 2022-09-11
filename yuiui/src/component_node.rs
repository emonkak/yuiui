use std::fmt;
use std::marker::PhantomData;

use crate::component::Component;
use crate::context::MessageContext;
use crate::event::Lifecycle;
use crate::state::Store;
use crate::view_node::CommitMode;

pub struct ComponentNode<C: Component<S, M, B>, S, M, B> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) state: C::State,
    _phantom: PhantomData<(S, B)>,
}

impl<C, S, M, B> ComponentNode<C, S, M, B>
where
    C: Component<S, M, B>,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: C::State::default(),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn render(&self, store: &Store<S>) -> C::Element {
        self.component.render(&self.state, store)
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                self.component.lifecycle(
                    Lifecycle::Mount,
                    &mut self.state,
                    context,
                    store,
                    backend,
                );
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    pending_component.lifecycle(
                        Lifecycle::Update(&self.component),
                        &mut self.state,
                        context,
                        store,
                        backend,
                    );
                    self.component = pending_component;
                    true
                } else {
                    false
                }
            }
            CommitMode::Unmount => {
                self.component.lifecycle(
                    Lifecycle::Unmount,
                    &mut self.state,
                    context,
                    store,
                    backend,
                );
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
