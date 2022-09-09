use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::Store;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Memoize<E, Deps, S, B> {
    render: fn(&Deps, &S, &B) -> E,
    deps: Deps,
}

impl<E, Deps, S, B> Memoize<E, Deps, S, B> {
    pub const fn new(render: fn(&Deps, &S, &B) -> E, deps: Deps) -> Self {
        Self { render, deps }
    }
}

impl<E, Deps, S, M, B> Element<S, M, B> for Memoize<E, Deps, S, B>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
{
    type View = E::View;

    type Components = (ComponentNode<AsComponent<Self>, S, M, B>, E::Components);

    const DEPTH: usize = E::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let element = ComponentElement::new(AsComponent::new(self));
        element.render(context, store, backend)
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> bool {
        let (head_node, _) = node.components;
        if head_node.component.inner.deps != self.deps {
            let element = ComponentElement::new(AsComponent::new(self));
            Element::update(element, node, context, store, backend)
        } else {
            head_node.pending_component = Some(AsComponent::new(self));
            false
        }
    }
}

impl<E, Deps, S, M, B> ElementSeq<S, M, B> for Memoize<E, Deps, S, B>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
{
    type Storage =
        ViewNode<E::View, (ComponentNode<AsComponent<Self>, S, M, B>, E::Components), S, M, B>;

    const DEPTH: usize = E::DEPTH;

    fn render_children(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> Self::Storage {
        self.render(context, store, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, store, backend)
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

impl<E, Deps, S, M, B> Component<S, M, B> for AsComponent<Memoize<E, Deps, S, B>>
where
    E: Element<S, M, B>,
    Deps: PartialEq,
{
    type Element = E;

    type State = ();

    fn render(&self, _local_state: &Self::State, store: &Store<S>, backend: &B) -> Self::Element {
        (self.inner.render)(&self.inner.deps, store, backend)
    }
}
