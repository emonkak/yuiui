mod component_element;
mod view_element;

pub use component_element::ComponentElement;
pub use view_element::ViewElement;

use std::fmt;

use crate::adapt::Adapt;
use crate::component_node::ComponentStack;
use crate::context::RenderContext;
use crate::state::State;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeScope, ViewNodeSeq};

pub trait Element<S: State, E> {
    type View: View<S, E>;

    type Components: ComponentStack<S, E, View = Self::View>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E>;

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;

    fn adapt<F, OriginState>(self, f: F) -> Adapt<Self, F, S>
    where
        Self: Sized,
        F: Fn(&OriginState) -> &S + Sync + Send + 'static,
    {
        Adapt::new(self, f.into())
    }
}

pub trait ElementSeq<S: State, E> {
    type Storage: ViewNodeSeq<S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage;

    fn update(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait DebuggableElement<S: State, E>:
    Element<
        S,
        E,
        View = <Self as DebuggableElement<S, E>>::View,
        Components = <Self as DebuggableElement<S, E>>::Components,
    > + fmt::Debug
{
    type View: View<S, E, Widget = Self::Widget, Children = Self::Children> + fmt::Debug;

    type Widget: fmt::Debug;

    type Children: ElementSeq<S, E, Storage = Self::Storage> + fmt::Debug;

    type Storage: ViewNodeSeq<S, E> + fmt::Debug;

    type Components: ComponentStack<S, E, View = <Self as DebuggableElement<S, E>>::View>
        + fmt::Debug;
}

impl<El, S, E> DebuggableElement<S, E> for El
where
    El: Element<S, E> + fmt::Debug,
    El::View: fmt::Debug,
    <El::View as View<S, E>>::Widget: fmt::Debug,
    <El::View as View<S, E>>::Children: fmt::Debug,
    <<El::View as View<S, E>>::Children as ElementSeq<S, E>>::Storage: fmt::Debug,
    El::Components: fmt::Debug,
    S: State,
{
    type View = El::View;

    type Widget = <El::View as View<S, E>>::Widget;

    type Children = <El::View as View<S, E>>::Children;

    type Storage = <<El::View as View<S, E>>::Children as ElementSeq<S, E>>::Storage;

    type Components = El::Components;
}
