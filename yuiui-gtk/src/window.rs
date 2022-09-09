use gtk::prelude::*;
use std::marker::PhantomData;
use yuiui::{ElementSeq, Store, HasEvent, Lifecycle, MessageContext, View, ViewElement};

use crate::backend::Backend;

pub trait GtkView<S, M>:
    View<
    S,
    M,
    Backend,
    State = <Self as GtkView<S, M>>::State,
    Children = <Self as GtkView<S, M>>::Children,
>
{
    type State: IsA<gtk::Widget>;

    type Children: ElementSeq<S, M, Backend>;
}

impl<V, S, M> GtkView<S, M> for V
where
    V: View<S, M, Backend>,
    V::State: IsA<gtk::Widget>,
{
    type Children = V::Children;

    type State = V::State;
}

pub struct ApplicationWindow<Child> {
    title: Option<String>,
    child: PhantomData<Child>,
}

impl<Child: IsA<gtk::Widget>> ApplicationWindow<Child> {
    pub fn new(title: Option<String>) -> Self {
        Self {
            title,
            child: PhantomData,
        }
    }
}

impl<Child, S, M> View<S, M, Backend> for ApplicationWindow<Child>
where
    Child: GtkView<S, M>,
    M: Send + 'static,
{
    type Children = ViewElement<Child, S, M, Backend>;

    type State = gtk::ApplicationWindow;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        view_state: &mut Self::State,
        _children: &<Self::Children as ElementSeq<S, M, Backend>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &Backend,
    ) {
        match lifecycle {
            Lifecycle::Mount => {
                view_state.show();
            }
            Lifecycle::Update(old_view) => {
                if self.title != old_view.title {
                    view_state.set_title(self.title.as_deref());
                }
            }
            Lifecycle::Unmount => {
                view_state.hide();
            }
        }
    }

    fn build(
        &self,
        child: &<Self::Children as ElementSeq<S, M, Backend>>::Storage,
        _store: &Store<S>,
        backend: &Backend,
    ) -> Self::State {
        let mut builder = gtk::ApplicationWindow::builder();

        if let Some(title) = &self.title {
            builder = builder.title(title)
        }

        builder
            .application(backend.application())
            .child(child.as_view_state().unwrap())
            .build()
    }
}

impl<'event, Child> HasEvent<'event> for ApplicationWindow<Child> {
    type Event = ();
}
