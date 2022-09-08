use std::fmt;
use std::marker::PhantomData;

use crate::component::Component;
use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::event::Lifecycle;
use crate::id::Depth;
use crate::state::State;
use crate::view_node::CommitMode;

pub struct ComponentNode<C: Component<S, B>, S: State, B> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) state: C::State,
    _phantom: PhantomData<(S, B)>,
}

impl<C, S, B> ComponentNode<C, S, B>
where
    C: Component<S, B>,
    S: State,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: C::State::default(),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn render(&self, state: &S, backend: &B) -> C::Element {
        self.component.render(&self.state, state, backend)
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        depth: Depth,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        context.set_depth(depth);
        match mode {
            CommitMode::Mount => self.component.lifecycle(
                Lifecycle::Mount,
                &mut self.state,
                context,
                state,
                backend,
            ),
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let result = pending_component.lifecycle(
                        Lifecycle::Update(&self.component),
                        &mut self.state,
                        context,
                        state,
                        backend,
                    );
                    self.component = pending_component;
                    result
                } else {
                    EffectOps::nop()
                }
            }
            CommitMode::Unmount => self.component.lifecycle(
                Lifecycle::Unmount,
                &mut self.state,
                context,
                state,
                backend,
            ),
        }
    }
}

impl<C, S, B> fmt::Debug for ComponentNode<C, S, B>
where
    C: Component<S, B> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .finish()
    }
}
