use std::marker::PhantomData;
use std::mem;

use crate::context::EffectContext;
use crate::element::{ComponentElement, Element};
use crate::sequence::CommitMode;
use crate::state::State;

pub trait Component<S: State>: Sized {
    type Element: Element<S>;

    fn lifecycle(
        &self,
        _lifecycle: ComponentLifecycle<Self>,
        _state: &S,
        _context: &mut EffectContext<S>,
    ) {
    }

    fn render(&self, state: &S) -> Self::Element;

    fn should_update(&self, _other: &Self, _state: &S) -> bool {
        true
    }

    fn el(self) -> ComponentElement<Self, S>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

#[derive(Debug)]
pub struct ComponentNode<C: Component<S>, S: State> {
    pub component: C,
    pub pending_component: Option<C>,
    pub state: PhantomData<S>,
}

impl<C: Component<S>, S: State> ComponentNode<C, S> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: PhantomData,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        let lifecycle = match mode {
            CommitMode::Mount => {
                ComponentLifecycle::Mounted
            }
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("get pending component"),
                );
                ComponentLifecycle::Updated(old_component)
            }
            CommitMode::Unmount => {
                ComponentLifecycle::Unmounted
            }
        };
        self.component.lifecycle(lifecycle, state, context);
        context.next_component();
    }
}

pub trait ComponentStack<S: State> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>);
}

impl<C, CS, S> ComponentStack<S> for (ComponentNode<C, S>, CS)
where
    C: Component<S>,
    CS: ComponentStack<S>,
    S: State,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, context);
        self.1.commit(mode, state, context);
    }
}

impl<S: State> ComponentStack<S> for () {
    fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut EffectContext<S>) {}
}

#[derive(Debug)]
pub enum ComponentLifecycle<C> {
    Mounted,
    Updated(C),
    Unmounted,
}
