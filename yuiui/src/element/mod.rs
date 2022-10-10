mod component;
mod connect;
mod memoize;
mod view;

pub use component::ComponentEl;
pub use connect::ConnectEl;
pub use memoize::Memoize;
pub use view::ViewEl;

use std::fmt;

use crate::component_stack::ComponentStack;
use crate::context::RenderContext;
use crate::store::Store;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeSeq};

pub trait Element<S, M, R>:
    Sized + ElementSeq<S, M, R, Storage = ViewNode<Self::View, Self::Components, S, M, R>>
{
    type View: View<S, M, R>;

    type Components: ComponentStack<S, M, R, View = Self::View>;

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, R>;

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, R>,
        context: &mut RenderContext,
        state: &S,
    ) -> bool;

    fn connect<PS, PM>(
        self,
        state_selector: fn(&PS) -> &Store<S>,
        message_selector: fn(M) -> PM,
    ) -> ConnectEl<Self, PS, PM, S, M> {
        ConnectEl::new(self, state_selector, message_selector)
    }
}

pub trait ElementSeq<S, M, R> {
    type Storage: ViewNodeSeq<S, M, R>;

    fn render_children(self, context: &mut RenderContext, state: &S) -> Self::Storage;

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
    ) -> bool;
}

pub trait DebuggableElement<S, M, R>:
    Element<
        S,
        M,
        R,
        View = <Self as DebuggableElement<S, M, R>>::View,
        Components = <Self as DebuggableElement<S, M, R>>::Components,
    > + fmt::Debug
{
    type View: View<S, M, R, State = Self::State, Children = Self::Children> + fmt::Debug;

    type State: fmt::Debug;

    type Children: ElementSeq<S, M, R, Storage = <Self as DebuggableElement<S, M, R>>::Storage>
        + fmt::Debug;

    type Storage: ViewNodeSeq<S, M, R> + fmt::Debug;

    type Components: ComponentStack<S, M, R, View = <Self as DebuggableElement<S, M, R>>::View>
        + fmt::Debug;
}

impl<E, S, M, R> DebuggableElement<S, M, R> for E
where
    E: Element<S, M, R> + fmt::Debug,
    E::View: fmt::Debug,
    <E::View as View<S, M, R>>::State: fmt::Debug,
    <E::View as View<S, M, R>>::Children: fmt::Debug,
    <<E::View as View<S, M, R>>::Children as ElementSeq<S, M, R>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
{
    type View = E::View;

    type State = <E::View as View<S, M, R>>::State;

    type Children = <E::View as View<S, M, R>>::Children;

    type Storage = <<E::View as View<S, M, R>>::Children as ElementSeq<S, M, R>>::Storage;

    type Components = E::Components;
}
