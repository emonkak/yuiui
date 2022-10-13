mod component;
mod connect;
mod memoize;
mod view;

pub use component::ComponentElement;
pub use connect::ConnectEl;
pub use memoize::Memoize;
pub use view::ViewElement;

use std::fmt;

use crate::component_stack::ComponentStack;
use crate::id::IdContext;
use crate::store::Store;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeSeq};

pub trait Element<S, M, B>:
    Sized + ElementSeq<S, M, B, Storage = ViewNode<Self::View, Self::Components, S, M, B>>
{
    type View: View<S, M, B>;

    type Components: ComponentStack<S, M, B, View = Self::View>;

    fn render(
        self,
        id_context: &mut IdContext,
        state: &S,
    ) -> ViewNode<Self::View, Self::Components, S, M, B>;

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, B>,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool;

    fn connect<PS, PM>(
        self,
        select_store: fn(&PS) -> &Store<S>,
        lift_message: fn(M) -> PM,
    ) -> ConnectEl<Self, PS, PM, S, M> {
        ConnectEl::new(self, select_store, lift_message)
    }
}

pub trait ElementSeq<S, M, B> {
    type Storage: ViewNodeSeq<S, M, B>;

    fn render_children(self, id_context: &mut IdContext, state: &S) -> Self::Storage;

    fn update_children(
        self,
        storage: &mut Self::Storage,
        id_context: &mut IdContext,
        state: &S,
    ) -> bool;
}

pub trait DebuggableElement<S, M, B>:
    Element<
        S,
        M,
        B,
        View = <Self as DebuggableElement<S, M, B>>::View,
        Components = <Self as DebuggableElement<S, M, B>>::Components,
    > + fmt::Debug
{
    type View: View<S, M, B, State = Self::State, Children = Self::Children> + fmt::Debug;

    type State: fmt::Debug;

    type Children: ElementSeq<S, M, B, Storage = <Self as DebuggableElement<S, M, B>>::Storage>
        + fmt::Debug;

    type Storage: ViewNodeSeq<S, M, B> + fmt::Debug;

    type Components: ComponentStack<S, M, B, View = <Self as DebuggableElement<S, M, B>>::View>
        + fmt::Debug;
}

impl<E, S, M, B> DebuggableElement<S, M, B> for E
where
    E: Element<S, M, B> + fmt::Debug,
    E::View: fmt::Debug,
    <E::View as View<S, M, B>>::State: fmt::Debug,
    <E::View as View<S, M, B>>::Children: fmt::Debug,
    <<E::View as View<S, M, B>>::Children as ElementSeq<S, M, B>>::Storage: fmt::Debug,
    E::Components: fmt::Debug,
{
    type View = E::View;

    type State = <E::View as View<S, M, B>>::State;

    type Children = <E::View as View<S, M, B>>::Children;

    type Storage = <<E::View as View<S, M, B>>::Children as ElementSeq<S, M, B>>::Storage;

    type Components = E::Components;
}
