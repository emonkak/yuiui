use std::fmt;
use std::marker::PhantomData;

use crate::component::Component;
use crate::context::EffectContext;
use crate::event::Lifecycle;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::view_node::CommitMode;

pub struct ComponentNode<C: Component<S, B>, S: State, B> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
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
            _phantom: PhantomData,
        }
    }

    pub(crate) fn render(&self, state: &S, backend: &B) -> C::Element {
        self.component.render(state, backend)
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        component_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut EffectContext<S>,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                let result = self.component.lifecycle(Lifecycle::Mounted, state, backend);
                context.process_result(result, component_index);
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let result = pending_component.lifecycle(
                        Lifecycle::Updated(&self.component),
                        state,
                        backend,
                    );
                    self.component = pending_component;
                    context.process_result(result, component_index);
                    true
                } else {
                    false
                }
            }
            CommitMode::Unmount => {
                let result = self
                    .component
                    .lifecycle(Lifecycle::Unmounted, state, backend);
                context.process_result(result, component_index);
                true
            }
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
