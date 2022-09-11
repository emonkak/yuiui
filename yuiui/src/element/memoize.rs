use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Memoize<Deps, S, E> {
    render: fn(&Deps, &S) -> E,
    deps: Deps,
}

impl<Deps, S, E> Memoize<Deps, S, E> {
    pub fn new(render: fn(&Deps, &S) -> E, deps: Deps) -> Self {
        Self { render, deps }
    }
}

impl<Deps, S, E, M, B> Element<S, M, B> for Memoize<Deps, S, E>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
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

impl<Deps, S, E, M, B> ElementSeq<S, M, B> for Memoize<Deps, S, E>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
{
    type Storage =
        ViewNode<E::View, (ComponentNode<AsComponent<Self>, S, M, B>, E::Components), S, M, B>;

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

impl<Deps, S, E, M, B> Component<S, M, B> for AsComponent<Memoize<Deps, S, E>>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
{
    type Element = E;

    fn render(&self, state: &S) -> Self::Element {
        (self.inner.render)(&self.inner.deps, state)
    }
}
