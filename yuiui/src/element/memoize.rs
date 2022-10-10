use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
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

impl<F, DS, E, S, M, R> Element<S, M, R> for Memoize<F, DS, E>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, R>,
{
    type View = E::View;

    type Components = (ComponentNode<Memoized<Self>, S, M, R>, E::Components);

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, R> {
        let element = ComponentEl::new(Memoized::new(self));
        element.render(context, state)
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        context: &mut RenderContext,
        state: &S,
    ) -> bool {
        let (head_node, _) = node.components;
        if head_node.component().inner.deps != self.deps {
            let element = ComponentEl::new(Memoized::new(self));
            Element::update(element, node, context, state)
        } else {
            false
        }
    }
}

impl<F, DS, E, S, M, R> ElementSeq<S, M, R> for Memoize<F, DS, E>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, R>,
{
    type Storage = ViewNode<E::View, <Self as Element<S, M, R>>::Components, S, M, R>;

    fn render_children(self, context: &mut RenderContext, state: &S) -> Self::Storage {
        self.render(context, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
    ) -> bool {
        self.update(storage.into(), context, state)
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

impl<F, DS, E, S, M, R> Component<S, M, R> for Memoized<Memoize<F, DS, E>>
where
    F: Fn(&DS) -> E,
    DS: PartialEq,
    E: Element<S, M, R>,
{
    type Element = E;

    fn render(&self, _state: &S) -> Self::Element {
        (self.inner.render_fn)(&self.inner.deps)
    }
}
