use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentEl, Element, ElementSeq};

pub struct Memoize<F: Fn(&DS) -> E, DS, E> {
    render_fn: F,
    deps: DS,
    _phantom: PhantomData<E>,
}

impl<F, DS, E> Memoize<F, DS, E>
where
    F: Fn(&DS) -> E,
{
    pub fn new(render_fn: F, deps: DS) -> Self {
        Self {
            render_fn,
            deps,
            _phantom: PhantomData,
        }
    }
}

impl<F, DS, E, S, M, B> Element<S, M, B> for Memoize<F, DS, E>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, B>,
{
    type View = E::View;

    type Components = (ComponentNode<Memoized<Self>, S, M, B>, E::Components);

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let element = ComponentEl::new(Memoized::new(self));
        element.render(context, store)
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let (head_node, _) = node.components;
        if head_node.component().inner.deps != self.deps {
            let element = ComponentEl::new(Memoized::new(self));
            Element::update(element, node, context, store)
        } else {
            false
        }
    }
}

impl<F, DS, E, S, M, B> ElementSeq<S, M, B> for Memoize<F, DS, E>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, B>,
{
    type Storage = ViewNode<E::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        self.update(storage.borrow_mut(), context, store)
    }
}

pub struct Memoized<T> {
    inner: T,
}

impl<T> Memoized<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<F, DS, E, S, M, B> Component<S, M, B> for Memoized<Memoize<F, DS, E>>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, B>,
{
    type Element = E;

    fn render(&self, _state: &S) -> Self::Element {
        (self.inner.render_fn)(&self.inner.deps)
    }
}
