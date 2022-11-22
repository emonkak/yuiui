use std::marker::PhantomData;

use crate::component::{Component, HigherOrderComponent};
use crate::component_node::ComponentNode;
use crate::id::IdStack;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Memoize<Hoc: HigherOrderComponent<Deps, S, M, E>, Deps, S, M, E> {
    hoc: Hoc,
    deps: Deps,
    _phantom: PhantomData<(S, M, E)>,
}

impl<Hoc, Deps, S, M, E> Memoize<Hoc, Deps, S, M, E>
where
    Hoc: HigherOrderComponent<Deps, S, M, E>,
{
    #[inline]
    pub const fn new(hoc: Hoc, deps: Deps) -> Self {
        Self {
            hoc,
            deps,
            _phantom: PhantomData,
        }
    }
}

impl<Hoc, Deps, S, M, E> Element<S, M, E> for Memoize<Hoc, Deps, S, M, E>
where
    Hoc: HigherOrderComponent<Deps, S, M, E>,
    Hoc::Component: AsRef<Deps>,
    Deps: PartialEq,
{
    type View = <<Hoc::Component as Component<S, M, E>>::Element as Element<S, M, E>>::View;

    type Components = (
        ComponentNode<Hoc::Component, S, M, E>,
        <<Hoc::Component as Component<S, M, E>>::Element as Element<S, M, E>>::Components,
    );

    fn render(
        self,
        id_stack: &mut IdStack,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let component = self.hoc.build(self.deps);
        let element = ComponentElement::new(component);
        element.render(id_stack, state)
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        let (head_component, _) = node.components;
        let deps = head_component.component().as_ref();
        if deps != &self.deps {
            let component = self.hoc.build(self.deps);
            let element = ComponentElement::new(component);
            element.update(node, id_stack, state)
        } else {
            false
        }
    }
}

impl<Hoc, Deps, S, M, E> ElementSeq<S, M, E> for Memoize<Hoc, Deps, S, M, E>
where
    Hoc: HigherOrderComponent<Deps, S, M, E>,
    Hoc::Component: AsRef<Deps>,
    Deps: PartialEq,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, id_stack: &mut IdStack, state: &S) -> Self::Storage {
        self.render(id_stack, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_stack: &mut IdStack,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_stack, state)
    }
}
