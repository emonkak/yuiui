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
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let mut node = self.element.render(context, state, backend);
        node.env = Some(Rc::new(self.value));
        node
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let result = self.element.update(node, context, state, backend);
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

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        self.render(context, state, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, state, backend)
    }
}
