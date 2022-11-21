pub mod views;

mod backend;
mod command_runtime;
mod entry_point;

pub use backend::Backend;
pub use entry_point::EntryPoint;

use yuiui::{ComponentStack, Element, ElementSeq, View};

pub trait GtkElement<S, M, E = gtk::ApplicationWindow>:
    Element<
    S,
    M,
    Backend<E>,
    View = <Self as GtkElement<S, M, E>>::View,
    Components = <Self as GtkElement<S, M, E>>::Components,
>
{
    type View: GtkView<S, M, E>;

    type Components: ComponentStack<S, M, Backend<E>, View = <Self as GtkElement<S, M, E>>::View>;
}

impl<T, S, M, E> GtkElement<S, M, E> for T
where
    T: Element<S, M, Backend<E>>,
    T::View: GtkView<S, M, E>,
{
    type View = T::View;

    type Components = T::Components;
}

pub trait GtkView<S, M, E>:
    View<
    S,
    M,
    Backend<E>,
    Children = <Self as GtkView<S, M, E>>::Children,
    State = <Self as GtkView<S, M, E>>::State,
>
{
    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, Backend<E>>;
}

impl<T, S, M, E> GtkView<S, M, E> for T
where
    T: View<S, M, Backend<E>>,
    T::State: AsRef<gtk::Widget>,
{
    type State = T::State;

    type Children = T::Children;
}
