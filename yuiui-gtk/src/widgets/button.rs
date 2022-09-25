use std::marker::PhantomData;

use gtk::glib::object::ObjectExt;
use gtk::glib::SignalHandlerId;
use gtk::{gdk, glib, prelude::*};
use yuiui::{
    ElementSeq, EventDestination, EventListener, Lifecycle, MessageContext, Store, View, ViewEl,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::backend::GtkBackend;
use crate::element::GtkElement;

pub fn button<OnClick, Child, S, M>(
    builder: ButtonBuilder,
    on_click: OnClick,
    child: Child,
) -> ViewEl<Button<OnClick, Child>, S, M, GtkBackend>
where
    OnClick: Fn(&S) -> M,
    Child: GtkElement<S, M>,
{
    Button::new(builder, on_click).el_with(child)
}

#[derive(Debug)]
pub struct Button<OnClick, Child> {
    builder: ButtonBuilder,
    on_click: OnClick,
    _phantom: PhantomData<Child>,
}

impl<OnClick, Child> Button<OnClick, Child> {
    fn new(builder: ButtonBuilder, on_click: OnClick) -> Self {
        Self {
            builder,
            on_click,
            _phantom: PhantomData,
        }
    }
}

impl<OnClick, Child, S, M> View<S, M, GtkBackend> for Button<OnClick, Child>
where
    OnClick: Fn(&S) -> M,
    Child: GtkElement<S, M>,
{
    type Children = Child;

    type State = ButtonState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        context: &mut MessageContext<M>,
        _store: &Store<S>,
        backend: &mut GtkBackend,
    ) {
        match lifecycle {
            Lifecycle::Mount => {
                let event_port = backend.event_port();
                let id_path = context.id_path().to_vec();
                view_state.clicked_signal = view_state
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
                self.builder.update(&old_view.builder, &view_state.widget);
            }
            Lifecycle::Unmount => {
                if let Some(signal_id) = view_state.clicked_signal.take() {
                    view_state.widget.disconnect(signal_id);
                }
            }
        }
    }

    fn event(
        &self,
        _event: <Self as EventListener>::Event,
        _view_state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        _backend: &mut GtkBackend,
    ) {
        let message = (self.on_click)(store);
        context.push_message(message);
    }

    fn build(
        &self,
        child: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        _store: &Store<S>,
        _backend: &mut GtkBackend,
    ) -> Self::State {
        let widget = self.builder.build();
        widget.set_child(Some(child.state().as_view_state().unwrap().as_ref()));
        ButtonState::new(widget)
    }
}

impl<'event, OnClick, Child> EventListener<'event> for Button<OnClick, Child> {
    type Event = &'event Event;
}

#[derive(Debug, WidgetBuilder)]
#[widget(gtk::Button)]
pub struct ButtonBuilder {
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