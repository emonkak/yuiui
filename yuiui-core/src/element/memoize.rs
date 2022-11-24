use crate::component::{Component, HigherOrderComponent};
use crate::context::RenderContext;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct MemoizeElement<Hoc, Deps> {
    hoc: Hoc,
    deps: Deps,
}

impl<Hoc, Deps> MemoizeElement<Hoc, Deps> {
    #[inline]
    pub const fn new(hoc: Hoc, deps: Deps) -> Self {
        Self { hoc, deps }
    }
}

impl<Hoc, Deps, S, M, E> Element<S, M, E> for MemoizeElement<Hoc, Deps>
where
    Hoc: HigherOrderComponent<Deps, S, M, E>,
    Hoc::Component: AsRef<Deps>,
    Deps: PartialEq,
{
    type View = <<Hoc::Component as Component<S, M, E>>::Element as Element<S, M, E>>::View;

    type Components = (
        Hoc::Component,
        <<Hoc::Component as Component<S, M, E>>::Element as Element<S, M, E>>::Components,
    );

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let component = self.hoc.build(self.deps);
        let element = ComponentElement::new(component);
        element.render(context)
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        let (head_component, _) = node.components;
        let deps = head_component.as_ref();
        if deps != &self.deps {
            let component = self.hoc.build(self.deps);
            let element = ComponentElement::new(component);
            element.update(node, context)
        } else {
            false
        }
    }
}

impl<Hoc, Deps, S, M, E> ElementSeq<S, M, E> for MemoizeElement<Hoc, Deps>
where
    Hoc: HigherOrderComponent<Deps, S, M, E>,
    Hoc::Component: AsRef<Deps>,
    Deps: PartialEq,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_node(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}
