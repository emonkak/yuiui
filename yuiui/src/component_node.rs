use std::fmt;
use std::marker::PhantomData;
use std::mem;

use crate::component::Component;
use crate::context::EffectContext;
use crate::event::Lifecycle;
use crate::state::State;
use crate::view_node::CommitMode;

pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) local_state: C::LocalState,
    _phantom: PhantomData<(S, E)>,
}

impl<C, S, E> ComponentNode<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub(crate) fn new(component: C, local_state: C::LocalState) -> Self {
        Self {
            component,
            pending_component: None,
            local_state,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn render(&self, state: &S, env: &E) -> C::Element {
        self.component.render(&self.local_state, state, env)
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let result = match mode {
            CommitMode::Mount => {
                self.component
                    .lifecycle(Lifecycle::Mounted, &mut self.local_state, state, env)
            }
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("take pending component"),
                );
                self.component.lifecycle(
                    Lifecycle::Updated(&old_component),
                    &mut self.local_state,
                    state,
                    env,
                )
            }
            CommitMode::Unmount => {
                self.component
                    .lifecycle(Lifecycle::Unmounted, &mut self.local_state, state, env)
            }
        };
        context.process_result(result);
    }
}

impl<C, S, E> fmt::Debug for ComponentNode<C, S, E>
where
    C: Component<S, E> + fmt::Debug,
    C::LocalState: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .field("local_state", &self.local_state)
            .finish()
    }
}
