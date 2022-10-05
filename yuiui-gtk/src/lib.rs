pub mod views;

mod entry_point;
mod execution_context;
mod renderer;

pub use entry_point::{DefaultEntryPoint, EntryPoint};
pub use renderer::Renderer;

use yuiui::{ComponentStack, Element, ElementSeq, View};

pub trait GtkElement<S, M>:
    Element<
    S,
    M,
    Renderer,
    View = <Self as GtkElement<S, M>>::View,
    Components = <Self as GtkElement<S, M>>::Components,
>
{
    type View: GtkView<S, M>;

    type Components: ComponentStack<S, M, Renderer, View = <Self as GtkElement<S, M>>::View>;
}

impl<E, S, M> GtkElement<S, M> for E
where
    E: Element<S, M, Renderer>,
    E::View: GtkView<S, M>,
{
    type View = E::View;

    type Components = E::Components;
}

pub trait GtkView<S, M>:
    View<
    S,
    M,
    Renderer,
    Children = <Self as GtkView<S, M>>::Children,
    State = <Self as GtkView<S, M>>::State,
>
{
    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, Renderer>;
}

impl<V, S, M> GtkView<S, M> for V
where
    V: View<S, M, Renderer>,
    V::State: AsRef<gtk::Widget>,
{
    type State = V::State;

    type Children = V::Children;
}
