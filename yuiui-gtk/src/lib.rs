pub mod views;

mod command_runtime;
mod entry_point;

pub use entry_point::EntryPoint;

use yuiui::{ComponentStack, Element, ElementSeq, View};

pub trait GtkElement<S, M, B = EntryPoint>:
    Element<
    S,
    M,
    B,
    View = <Self as GtkElement<S, M, B>>::View,
    Components = <Self as GtkElement<S, M, B>>::Components,
>
{
    type View: GtkView<S, M, B>;

    type Components: ComponentStack<S, M, B, View = <Self as GtkElement<S, M, B>>::View>;
}

impl<T, S, M, B> GtkElement<S, M, B> for T
where
    T: Element<S, M, B>,
    T::View: GtkView<S, M, B>,
{
    type View = T::View;

    type Components = T::Components;
}

pub trait GtkView<S, M, B>:
    View<
    S,
    M,
    B,
    Children = <Self as GtkView<S, M, B>>::Children,
    State = <Self as GtkView<S, M, B>>::State,
>
{
    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, B>;
}

impl<T, S, M, B> GtkView<S, M, B> for T
where
    T: View<S, M, B>,
    T::State: AsRef<gtk::Widget>,
{
    type State = T::State;

    type Children = T::Children;
}
