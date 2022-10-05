pub mod views;

mod entry_point;
mod execution_context;
mod renderer;

pub use entry_point::{DefaultEntryPoint, EntryPoint};
pub use renderer::Renderer;

use yuiui::{ComponentStack, Element, ElementSeq, View};

pub trait GtkElement<S, M, R>:
    Element<
    S,
    M,
    R,
    View = <Self as GtkElement<S, M, R>>::View,
    Components = <Self as GtkElement<S, M, R>>::Components,
>
{
    type View: GtkView<S, M, R>;

    type Components: ComponentStack<S, M, R, View = <Self as GtkElement<S, M, R>>::View>;
}

impl<E, S, M, R> GtkElement<S, M, R> for E
where
    E: Element<S, M, R>,
    E::View: GtkView<S, M, R>,
{
    type View = E::View;

    type Components = E::Components;
}

pub trait GtkView<S, M, R>:
    View<
    S,
    M,
    R,
    Children = <Self as GtkView<S, M, R>>::Children,
    State = <Self as GtkView<S, M, R>>::State,
>
{
    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, R>;
}

impl<V, S, M, R> GtkView<S, M, R> for V
where
    V: View<S, M, R>,
    V::State: AsRef<gtk::Widget>,
{
    type State = V::State;

    type Children = V::Children;
}
