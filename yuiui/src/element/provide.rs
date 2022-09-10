use std::rc::Rc;

use crate::context::RenderContext;
use crate::state::Store;
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

impl<E, T, S, M, B> Element<S, M, B> for Provide<E, T>
where
    E: Element<S, M, B>,
    T: 'static,
{
    type View = E::View;

    type Components = E::Components;

    const DEPTH: usize = E::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let mut node = self.element.render(context, store, backend);
        let env = Rc::new(self.value);
        context.push_env(env.clone());
        node.env = Some(env);
        node
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        let result = self.element.update(node, context, store, backend);
        let env = Rc::new(self.value);
        context.push_env(env.clone());
        *node.env = Some(env);
        result
    }
}

impl<E, T, S, M, B> ElementSeq<S, M, B> for Provide<E, T>
where
    E: Element<S, M, B>,
    T: 'static,
{
    type Storage = ViewNode<E::View, E::Components, S, M, B>;

    const DEPTH: usize = E::DEPTH;

    fn render_children(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::Storage {
        self.render(context, store, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &mut B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store, backend)
    }
}
