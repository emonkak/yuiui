use std::rc::Rc;

use crate::context::RenderContext;
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct Provide<E, T> {
    element: E,
    value: T,
}

impl<E, T> Provide<E, T> {
    pub const fn new(element: E, value: T) -> Self {
        Self { element, value }
    }
}

impl<E, T, S, B> Element<S, B> for Provide<E, T>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type View = E::View;

    type Components = E::Components;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let mut node = self.element.render(state, backend, context);
        node.env = Some(Rc::new(self.value));
        node
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let result = self.element.update(node, state, backend, context);
        *node.env = Some(Rc::new(self.value));
        result
    }
}

impl<E, T, S, B> ElementSeq<S, B> for Provide<E, T>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type Storage = ViewNode<E::View, E::Components, S, B>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        self.render(state, backend, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), state, backend, context)
    }
}
