use std::marker::PhantomData;
use std::mem;

use crate::component::{Component, ComponentLifecycle};
use crate::effect::EffectContext;
use crate::state::State;
use crate::widget_node::CommitMode;

#[derive(Debug)]
pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) state: PhantomData<S>,
    pub(crate) env: PhantomData<E>,
}

impl<C, S, E> ComponentNode<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: PhantomData,
            env: PhantomData,
        }
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let lifecycle = match mode {
            CommitMode::Mount => ComponentLifecycle::Mounted,
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("get pending component"),
                );
                ComponentLifecycle::Updated(old_component)
            }
            CommitMode::Unmount => ComponentLifecycle::Unmounted,
        };
        context.process_result(self.component.lifecycle(lifecycle, state, env));
        context.next_component();
    }
}

pub trait ComponentStack<S: State, E> {
    const LEN: usize;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

impl<C, CS, S, E> ComponentStack<S, E> for (ComponentNode<C, S, E>, CS)
where
    C: Component<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    const LEN: usize = 1 + CS::LEN;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, env, context);
        self.1.commit(mode, state, env, context);
    }
}

impl<S: State, E> ComponentStack<S, E> for () {
    const LEN: usize = 0;

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }
}
