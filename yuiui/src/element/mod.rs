mod adapt;
mod component;
mod memoize;
mod view;

pub use adapt::AdaptElement;
pub use component::ComponentElement;
pub use memoize::Memoize;
pub use view::ViewElement;

use std::fmt;

use crate::component_stack::ComponentStack;
use crate::id::IdContext;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut, ViewNodeSeq};

pub trait Element<S, M, E>:
    Sized + ElementSeq<S, M, E, Storage = ViewNode<Self::View, Self::Components, S, M, E>>
{
    type View: View<S, M, E>;

    type Components: ComponentStack<S, M, E, View = Self::View>;

    fn render(
        self,
        state: &S,
        id_context: &mut IdContext,
    ) -> ViewNode<Self::View, Self::Components, S, M, E>;

    fn update(
        self,
        node: ViewNodeMut<Self::View, Self::Components, S, M, E>,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool;

    fn adapt<PS, PM>(
        self,
        select_state: fn(&PS) -> &S,
        lift_message: fn(M) -> PM,
    ) -> AdaptElement<Self, PS, PM, S, M> {
        AdaptElement::new(self, select_state, lift_message)
    }
}

pub trait ElementSeq<S, M, E> {
    type Storage: ViewNodeSeq<S, M, E>;

    fn render_children(self, state: &S, id_context: &mut IdContext) -> Self::Storage;

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool;
}

pub trait DebuggableElement<S, M, E>:
    Element<
        S,
        M,
        E,
        View = <Self as DebuggableElement<S, M, E>>::View,
        Components = <Self as DebuggableElement<S, M, E>>::Components,
    > + fmt::Debug
{
    type View: View<S, M, E, State = Self::State, Children = Self::Children> + fmt::Debug;

    type State: fmt::Debug;

    type Children: ElementSeq<S, M, E, Storage = <Self as DebuggableElement<S, M, E>>::Storage>
        + fmt::Debug;

    type Storage: ViewNodeSeq<S, M, E> + fmt::Debug;

    type Components: ComponentStack<S, M, E, View = <Self as DebuggableElement<S, M, E>>::View>
        + fmt::Debug;
}

impl<Element, S, M, E> DebuggableElement<S, M, E> for Element
where
    Element: self::Element<S, M, E> + fmt::Debug,
    Element::View: fmt::Debug,
    <Element::View as View<S, M, E>>::State: fmt::Debug,
    <Element::View as View<S, M, E>>::Children: fmt::Debug,
    <<Element::View as View<S, M, E>>::Children as ElementSeq<S, M, E>>::Storage: fmt::Debug,
    Element::Components: fmt::Debug,
{
    type View = Element::View;

    type State = <Element::View as View<S, M, E>>::State;

    type Children = <Element::View as View<S, M, E>>::Children;

    type Storage = <<Element::View as View<S, M, E>>::Children as ElementSeq<S, M, E>>::Storage;

    type Components = Element::Components;
}
