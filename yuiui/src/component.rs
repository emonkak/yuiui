use std::marker::PhantomData;
use std::mem;

use crate::context::EffectContext;
use crate::element::{ComponentElement, Element};
use crate::sequence::CommitMode;
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type Element: Element<S, E>;

    fn lifecycle(
        &self,
        _lifecycle: ComponentLifecycle<Self>,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) {
    }

    fn render(&self, state: &S, env: &E) -> Self::Element;

    fn should_update(&self, _other: &Self, _state: &S, _env: &E) -> bool {
        true
    }

    fn el(self) -> ComponentElement<Self, S, E>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

#[derive(Debug)]
pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub component: C,
    pub pending_component: Option<C>,
    pub state: PhantomData<S>,
    pub env: PhantomData<E>,
}

impl<C, S, E> ComponentNode<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: PhantomData,
            env: PhantomData,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
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
        self.component.lifecycle(lifecycle, state, env, context);
        context.next_component();
    }
}

pub trait ComponentStack<S: State, E> {
    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

impl<C, CS, S, E> ComponentStack<S, E> for (ComponentNode<C, S, E>, CS)
where
    C: Component<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, env, context);
        self.1.commit(mode, state, env, context);
    }
}

impl<S: State, E> ComponentStack<S, E> for () {
    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }
}

#[derive(Debug)]
pub enum ComponentLifecycle<C> {
    Mounted,
    Updated(C),
    Unmounted,
}
