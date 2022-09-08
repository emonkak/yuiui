use gtk::prelude::*;
use std::marker::PhantomData;
use yuiui::{
    EffectContext, ElementSeq, EventResult, HasEvent, Lifecycle, State, View, ViewElement,
};

use crate::backend::Backend;

pub trait GtkView<S: State>:
    View<
    S,
    Backend<S>,
    Widget = <Self as GtkView<S>>::Widget,
    Children = <Self as GtkView<S>>::Children,
>
{
    type Widget: IsA<gtk::Widget>;

    type Children: ElementSeq<S, Backend<S>>;
}

impl<V, S> GtkView<S> for V
where
    V: View<S, Backend<S>>,
    V::Widget: IsA<gtk::Widget>,
    S: State,
{
    type Widget = V::Widget;

    type Children = V::Children;
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
    type Widget = gtk::ApplicationWindow;

    type Children = ViewElement<Child, S, Backend<S>>;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        widget: &mut Self::Widget,
        _children: &<Self::Children as ElementSeq<S, Backend<S>>>::Storage,
        _context: &EffectContext,
        _state: &S,
        _backend: &Backend<S>,
    ) -> EventResult<S> {
        match lifecycle {
            Lifecycle::Mounted => {
                widget.show();
            }
            Lifecycle::Updated(old_view) => {
                if self.title != old_view.title {
                    widget.set_title(self.title.as_deref());
                }
            }
            Lifecycle::Unmounted => {
                widget.hide();
            }
        }
        EventResult::nop()
    }

    fn build(
        &self,
        child: &<Self::Children as ElementSeq<S, Backend<S>>>::Storage,
        _state: &S,
        env: &Backend<S>,
    ) -> Self::Widget {
        let mut builder = gtk::ApplicationWindow::builder();

        if let Some(title) = &self.title {
            builder = builder.title(title)
        }

        builder
            .application(env.application())
            .child(child.as_widget().unwrap())
            .build()
    }
}

impl<'event, Child> HasEvent<'event> for ApplicationWindow<Child> {
    type Event = ();
}
