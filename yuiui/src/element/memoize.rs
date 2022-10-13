use std::marker::PhantomData;

use crate::component::{Component, HigherOrderComponent};
use crate::component_node::ComponentNode;
use crate::id::IdContext;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Memoize<Hoc: HigherOrderComponent<Deps, S, M, B>, Deps, S, M, B> {
    hoc: Hoc,
    deps: Deps,
    _phantom: PhantomData<(S, M, B)>,
}

impl<Hoc, Deps, S, M, B> Memoize<Hoc, Deps, S, M, B>
where
    Hoc: HigherOrderComponent<Deps, S, M, B>,
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

impl<Hoc, Deps, S, M, B> Element<S, M, B> for Memoize<Hoc, Deps, S, M, B>
where
    Hoc: HigherOrderComponent<Deps, S, M, B>,
    Hoc::Component: PartialEq<Deps>,
{
    type View = <<Hoc::Component as Component<S, M, B>>::Element as Element<S, M, B>>::View;

    type Components = (
        ComponentNode<Hoc::Component, S, M, B>,
        <<Hoc::Component as Component<S, M, B>>::Element as Element<S, M, B>>::Components,
    );

    fn render(
        self,
        id_context: &mut IdContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let component = self.hoc.build(self.deps);
        let element = ComponentElement::new(component);
        element.render(id_context, state)
    }

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, B>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        let (head_component, _) = node.components;
        if head_component.component() != &self.deps {
            let component = self.hoc.build(self.deps);
            let element = ComponentElement::new(component);
            element.update(node, id_context, state)
        } else {
            false
        }
    }
}

impl<Hoc, Deps, S, M, B> ElementSeq<S, M, B> for Memoize<Hoc, Deps, S, M, B>
where
    Hoc: HigherOrderComponent<Deps, S, M, B>,
    Hoc::Component: PartialEq<Deps>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, B>>::View, <Self as Element<S, M, B>>::Components, S, M, B>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage {
        self.render(id_context, state)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool {
        self.update(storage.into(), id_context, state)
    }
}
