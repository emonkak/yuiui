pub mod views;

mod entry_point;
mod execution_context;
mod renderer;

pub use entry_point::{DefaultEntryPoint, EntryPoint};
pub use renderer::Renderer;

use yuiui::{ComponentStack, Element, ElementSeq, View};

pub trait GtkElement<S, M, B>:
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

impl<E, S, M, B> GtkElement<S, M, B> for E
where
    E: Element<S, M, B>,
    <E::View as View<S, M, B>>::State: AsRef<gtk::Widget>,
{
    type View = E::View;

    type Components = E::Components;
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

impl<V, S, M, B> GtkView<S, M, B> for V
where
    V: View<S, M, B>,
    V::State: AsRef<gtk::Widget>,
{
    type State = V::State;

    type Children = V::Children;
}
