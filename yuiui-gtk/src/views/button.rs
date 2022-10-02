use std::marker::PhantomData;

use gtk::glib::object::ObjectExt;
use gtk::glib::SignalHandlerId;
use gtk::{gdk, glib, prelude::*};
use yuiui::{
    Element, ElementSeq, EventDestination, EventListener, Lifecycle, MessageContext, Store, View,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::backend::GtkBackend;

#[derive(WidgetBuilder)]
#[widget(gtk::Button)]
pub struct Button<Child, S, M> {
    child: Option<gtk::Widget>,
    has_frame: Option<bool>,
    icon_name: Option<String>,
    label: Option<String>,
    use_underline: Option<bool>,
    can_focus: Option<bool>,
    can_target: Option<bool>,
    css_classes: Option<Vec<String>>,
    css_name: Option<String>,
    cursor: Option<gdk::Cursor>,
    focus_on_click: Option<bool>,
    focusable: Option<bool>,
    halign: Option<gtk::Align>,
    has_tooltip: Option<bool>,
    height_request: Option<i32>,
    hexpand: Option<bool>,
    hexpand_set: Option<bool>,
    layout_manager: Option<gtk::LayoutManager>,
    margin_bottom: Option<i32>,
    margin_end: Option<i32>,
    margin_start: Option<i32>,
    margin_top: Option<i32>,
    name: Option<String>,
    opacity: Option<f64>,
    overflow: Option<gtk::Overflow>,
    receives_default: Option<bool>,
    sensitive: Option<bool>,
    tooltip_markup: Option<String>,
    tooltip_text: Option<String>,
    valign: Option<gtk::Align>,
    vexpand: Option<bool>,
    vexpand_set: Option<bool>,
    visible: Option<bool>,
    width_request: Option<i32>,
    accessible_role: Option<gtk::AccessibleRole>,
    action_name: Option<String>,
    action_target: Option<glib::Variant>,
    #[property(bind = false)]
    on_click: Option<Box<dyn Fn(&S) -> M>>,
    #[property(bind = false, setter = false)]
    _phantom: PhantomData<Child>,
}

impl<Child, S, M> View<S, M, GtkBackend> for Button<Child, S, M>
where
    Child: Element<S, M, GtkBackend>,
    <Child::View as View<S, M, GtkBackend>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = ButtonState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        context: &mut MessageContext<M>,
        _store: &Store<S>,
        backend: &mut GtkBackend,
    ) {
        match lifecycle {
            Lifecycle::Mount | Lifecycle::Remount => {
                let event_port = backend.event_port().clone();
                let id_path = context.id_path().to_vec();
                state.clicked_signal = state
                    .widget
                    .connect_clicked(move |_| {
                        event_port
                            .send((
                                Box::new(Event::Clicked),
                                EventDestination::Local(id_path.clone()),
                            ))
                            .unwrap();
                    })
                    .into();
            }
            Lifecycle::Update(old_view) => {
                self.update(&old_view, &state.widget);
            }
            Lifecycle::Unmount => {
                if let Some(signal_id) = state.clicked_signal.take() {
                    state.widget.disconnect(signal_id);
                }
            }
        }
    }

    fn event(
        &self,
        _event: <Self as EventListener>::Event,
        _state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        _backend: &mut GtkBackend,
    ) {
        if let Some(on_click) = &self.on_click {
            let message = on_click(store);
            context.push_message(message);
        }
    }

    fn build(
        &self,
        child: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        _store: &Store<S>,
        _backend: &mut GtkBackend,
    ) -> Self::State {
        let widget = self.build();
        widget.set_child(Some(child.state().as_view_state().unwrap().as_ref()));
        ButtonState::new(widget)
    }
}

impl<'event, Child, S, M> EventListener<'event> for Button<Child, S, M> {
    type Event = &'event Event;
}

#[derive(Debug)]
pub struct ButtonState {
    widget: gtk::Button,
    clicked_signal: Option<SignalHandlerId>,
}

impl ButtonState {
    fn new(widget: gtk::Button) -> Self {
        Self {
            widget,
            clicked_signal: None,
        }
    }
}

impl AsRef<gtk::Widget> for ButtonState {
    fn as_ref(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}

#[derive(Debug)]
pub enum Event {
    Clicked,
}
