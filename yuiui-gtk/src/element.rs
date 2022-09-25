use yuiui::{ComponentStack, Element, ElementSeq, View, ViewNode, ViewNodeSeq};

use crate::backend::GtkBackend;

pub trait GtkElement<S, M>:
    Element<
        S,
        M,
        GtkBackend,
        View = <Self as GtkElement<S, M>>::View,
        Components = <Self as GtkElement<S, M>>::Components,
    > + ElementSeq<
        S,
        M,
        GtkBackend,
        Storage = ViewNode<
            <Self as GtkElement<S, M>>::View,
            <Self as GtkElement<S, M>>::Components,
            S,
            M,
            GtkBackend,
        >,
    >
{
    type View: View<S, M, GtkBackend, State = Self::State, Children = Self::Children>;

    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, GtkBackend, Storage = <Self as GtkElement<S, M>>::Storage>;

    type Storage: ViewNodeSeq<S, M, GtkBackend>;

    type Components: ComponentStack<S, M, GtkBackend, View = <Self as GtkElement<S, M>>::View>;
}

impl<E, S, M> GtkElement<S, M> for E
where
    E: Element<S, M, GtkBackend>
        + ElementSeq<S, M, GtkBackend, Storage = ViewNode<E::View, E::Components, S, M, GtkBackend>>,
    <E::View as View<S, M, GtkBackend>>::State: AsRef<gtk::Widget>,
{
    type View = E::View;

    type State = <E::View as View<S, M, GtkBackend>>::State;

    type Children = <E::View as View<S, M, GtkBackend>>::Children;

    type Storage =
        <<E::View as View<S, M, GtkBackend>>::Children as ElementSeq<S, M, GtkBackend>>::Storage;

    type Components = E::Components;
}
