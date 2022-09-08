use gtk::prelude::*;
use std::marker::PhantomData;
use yuiui::{EffectContext, EffectOps, ElementSeq, HasEvent, Lifecycle, State, View, ViewElement};

use crate::backend::Backend;

pub trait GtkView<S: State>:
    View<S, Backend<S>, State = <Self as GtkView<S>>::State, Children = <Self as GtkView<S>>::Children>
{
    type State: IsA<gtk::Widget>;

    type Children: ElementSeq<S, Backend<S>>;
}

impl<V, S> GtkView<S> for V
where
    V: View<S, Backend<S>>,
    V::State: IsA<gtk::Widget>,
    S: State,
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

impl<Child, S> View<S, Backend<S>> for ApplicationWindow<Child>
where
    Child: GtkView<S>,
    S: State,
{
    type Children = ViewElement<Child, S, Backend<S>>;

    type State = gtk::ApplicationWindow;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, Backend<S>>>::Storage,
        _context: &EffectContext,
        _state: &S,
        _backend: &Backend<S>,
    ) -> EffectOps<S> {
        match lifecycle {
            Lifecycle::Mounted => {
                view_state.show();
            }
            Lifecycle::Updated(old_view) => {
                if self.title != old_view.title {
                    view_state.set_title(self.title.as_deref());
                }
            }
            Lifecycle::Unmounted => {
                view_state.hide();
            }
        }
        EffectOps::nop()
    }

    fn build(
        &self,
        child: &<Self::Children as ElementSeq<S, Backend<S>>>::Storage,
        _state: &S,
        env: &Backend<S>,
    ) -> Self::State {
        let mut builder = gtk::ApplicationWindow::builder();

        if let Some(title) = &self.title {
            builder = builder.title(title)
        }

        builder
            .application(env.application())
            .child(child.as_view_state().unwrap())
            .build()
    }
}

impl<'event, Child> HasEvent<'event> for ApplicationWindow<Child> {
    type Event = ();
}
