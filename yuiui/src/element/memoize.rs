use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::RenderContext;
use crate::state::State;
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

impl<E, Deps, S, B> Element<S, B> for Memoize<E, Deps, S, B>
where
    E: Element<S, B>,
    Deps: PartialEq,
    S: State,
{
    type View = E::View;

    type Components = (ComponentNode<AsComponent<Self>, S, B>, E::Components);

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let element = ComponentElement::new(AsComponent::new(self));
        element.render(context, state, backend)
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let (head_node, _) = node.components;
        if head_node.component.inner.deps != self.deps {
            let element = ComponentElement::new(AsComponent::new(self));
            Element::update(element, node, context, state, backend)
        } else {
            head_node.pending_component = Some(AsComponent::new(self));
            false
        }
    }
}

impl<E, Deps, S, B> ElementSeq<S, B> for Memoize<E, Deps, S, B>
where
    E: Element<S, B>,
    Deps: PartialEq,
    S: State,
{
    type Storage = ViewNode<E::View, (ComponentNode<AsComponent<Self>, S, B>, E::Components), S, B>;

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

pub struct AsComponent<T> {
    inner: T,
}

impl<T> AsComponent<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<E, Deps, S, B> Component<S, B> for AsComponent<Memoize<E, Deps, S, B>>
where
    E: Element<S, B>,
    Deps: PartialEq,
    S: State,
{
    type Element = E;

    type State = ();

    fn render(&self, _local_state: &Self::State, state: &S, backend: &B) -> Self::Element {
        (self.inner.render)(&self.inner.deps, state, backend)
    }
}
