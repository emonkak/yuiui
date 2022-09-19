use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Memoize<F: Fn(&D) -> E, D, E> {
    render_fn: F,
    deps: D,
    phantom: PhantomData<E>,
}

impl<F, D, E> Memoize<F, D, E>
where
    F: Fn(&D) -> E,
    D: PartialEq,
{
    pub fn new(render_fn: F, deps: D) -> Self {
        Self {
            render_fn,
            deps,
            phantom: PhantomData,
        }
    }
}

impl<F, D, E, S, M, B> Element<S, M, B> for Memoize<F, D, E>
where
    F: Fn(&D) -> E,
    D: PartialEq,
    E: Element<S, M, B>,
{
    type View = E::View;

    type Components = (ComponentNode<AsComponent<Self>, S, M, B>, E::Components);

    fn render(
        self,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let element = ComponentElement::new(AsComponent::new(self));
        element.render(context, store)
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        let (head_node, _) = node.components;
        if head_node.component.inner.deps != self.deps {
            let element = ComponentElement::new(AsComponent::new(self));
            Element::update(element, node, context, store)
        } else {
            false
        }
    }
}

impl<F, D, E, S, M, B> ElementSeq<S, M, B> for Memoize<F, D, E>
where
    F: Fn(&D) -> E,
    D: PartialEq,
    E: Element<S, M, B>,
{
    type Storage = ViewNode<E::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, context: &mut RenderContext, store: &mut Store<S>) -> Self::Storage {
        self.render(context, store)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store)
    }
}

pub struct AsComponent<T> {
    inner: T,
}

impl<T> AsComponent<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<F, D, E, S, M, B> Component<S, M, B> for AsComponent<Memoize<F, D, E>>
where
    F: Fn(&D) -> E,
    D: PartialEq,
    E: Element<S, M, B>,
{
    type Element = E;

    fn render(&self, _state: &S) -> Self::Element {
        (self.inner.render_fn)(&self.inner.deps)
    }
}
