use std::marker::PhantomData;

use crate::context::Context;
use crate::element::{ComponentElement, Element};
use crate::sequence::CommitMode;
use crate::state::State;

pub trait Component<S: State> {
    type Element: Element<S>;

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
    pub state: PhantomData<S>,
}

impl<C: Component<S>, S: State> ComponentNode<C, S> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            state: PhantomData,
        }
    }

    pub fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut Context) {}
}

pub trait ComponentStack<S> {
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context);
}

impl<C, CS, S> ComponentStack<S> for (ComponentNode<C, S>, CS)
where
    C: Component<S>,
    CS: ComponentStack<S>,
    S: State,
{
    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        self.0.commit(mode, state, context);
        self.1.commit(mode, state, context);
    }
}

impl<S> ComponentStack<S> for () {
    fn commit(&mut self, _mode: CommitMode, _state: &S, _context: &mut Context) {}
}
