mod component;
mod consume;
mod memoize;
mod provide;
mod scope;
mod view;

pub use component::ComponentElement;
pub use consume::Consume;
pub use provide::Provide;
pub use memoize::Memoize;
pub use scope::Scope;
pub use view::ViewElement;

use std::fmt;

use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::state::State;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeSeq};

pub trait Element<S: State, B> {
    type View: View<S, B>;

    type Components: ComponentStack<S, B, View = Self::View>;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B>;

    fn update(
        self,
        scope: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool;

    fn scope<F, OriginState>(self, f: F) -> Scope<Self, F, S>
    where
        Self: Sized,
        F: Fn(&OriginState) -> &S + Sync + Send + 'static,
    {
        Scope::new(self, f.into())
    }

    fn provide<F, T>(self, value: T) -> Provide<Self, T>
    where
        Self: Sized,
        T: 'static,
    {
        Provide::new(self, value)
    }
}

pub trait ElementSeq<S: State, B> {
    type Storage: ViewNodeSeq<S, B>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage;

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait DebuggableElement<S: State, B>:
    Element<
        S,
        B,
        View = <Self as DebuggableElement<S, B>>::View,
        Components = <Self as DebuggableElement<S, B>>::Components,
    > + fmt::Debug
{
    type View: View<S, B, Widget = Self::Widget, Children = Self::Children> + fmt::Debug;

    type Widget: fmt::Debug;

    type Children: ElementSeq<S, B, Storage = Self::Storage> + fmt::Debug;

    type Storage: ViewNodeSeq<S, B> + fmt::Debug;

    type Components: ComponentStack<S, B, View = <Self as DebuggableElement<S, B>>::View>
        + fmt::Debug;
}

impl<E, S, B> DebuggableElement<S, B> for E
where
    E: Element<S, B> + fmt::Debug,
    E::View: fmt::Debug,
    <E::View as View<S, B>>::Widget: fmt::Debug,
    <E::View as View<S, B>>::Children: fmt::Debug,
    <<E::View as View<S, B>>::Children as ElementSeq<S, B>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
    S: State,
{
    type View = E::View;

    type Widget = <E::View as View<S, B>>::Widget;

    type Children = <E::View as View<S, B>>::Children;

    type Storage = <<E::View as View<S, B>>::Children as ElementSeq<S, B>>::Storage;

    type Components = E::Components;
}
