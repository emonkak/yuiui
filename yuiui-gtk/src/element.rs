use yuiui::{ComponentStack, Element, ElementSeq, View, ViewNodeSeq};

pub trait GtkElement<S, M, B>:
    Element<
    S,
    M,
    B,
    View = <Self as GtkElement<S, M, B>>::View,
    Components = <Self as GtkElement<S, M, B>>::Components,
>
{
    type View: View<S, M, B, State = Self::State, Children = Self::Children>;

    type State: AsRef<gtk::Widget>;

    type Children: ElementSeq<S, M, B, Storage = <Self as GtkElement<S, M, B>>::Storage>;

    type Storage: ViewNodeSeq<S, M, B>;

    type Components: ComponentStack<S, M, B, View = <Self as GtkElement<S, M, B>>::View>;
}

impl<E, S, M, B> GtkElement<S, M, B> for E
where
    E: Element<S, M, B>,
    <E::View as View<S, M, B>>::State: AsRef<gtk::Widget>,
{
    type View = E::View;

    type State = <E::View as View<S, M, B>>::State;

    type Children = <E::View as View<S, M, B>>::Children;

    type Storage = <<E::View as View<S, M, B>>::Children as ElementSeq<S, M, B>>::Storage;

    type Components = E::Components;
}
