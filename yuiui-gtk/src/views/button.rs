use std::marker::PhantomData;

use gtk::glib::object::ObjectExt;
use gtk::{gdk, glib, prelude::*};
use yuiui::{
    Element, ElementSeq, EventDestination, EventListener, IdPathBuf, Lifecycle, MessageContext,
    Store, View,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::renderer::{EventPort, Renderer};

#[derive(WidgetBuilder)]
#[widget(gtk::Button)]
pub struct Button<Child, M> {
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
    on_click: Option<Box<dyn Fn() -> M>>,
    #[property(bind = false, setter = false)]
    _phantom: PhantomData<Child>,
}

impl<Child, S, M> View<S, M, Renderer> for Button<Child, M>
where
    Child: Element<S, M, Renderer>,
    <Child::View as View<S, M, Renderer>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = ButtonState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, Renderer>>::Storage,
        context: &mut MessageContext<M>,
        _store: &Store<S>,
        renderer: &mut Renderer,
    ) {
        match lifecycle {
            Lifecycle::Mount | Lifecycle::Remount => {
                if self.on_click.is_some() {
                    state
                        .connect_clicked(context.id_path().to_vec(), renderer.event_port().clone());
                }
            }
            Lifecycle::Update(old_view) => {
                match (&self.on_click, &old_view.on_click) {
                    (Some(_), None) => {
                        state.disconnect_clicked();
                    }
                    (None, Some(_)) => {
                        state.connect_clicked(
                            context.id_path().to_vec(),
                            renderer.event_port().clone(),
                        );
                    }
                    _ => {}
                }
                self.update(&old_view, &state.widget);
            }
            Lifecycle::Unmount => {
                state.disconnect_clicked();
            }
        }
    }

    fn event(
        &self,
        event: <Self as EventListener>::Event,
        _state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, Renderer>>::Storage,
        context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut Renderer,
    ) {
        match event {
            Event::Clicked => {
                if let Some(on_click) = &self.on_click {
                    let message = on_click();
                    context.push_message(message);
                }
            }
        }
    }

    fn build(
        &self,
        child: &mut <Self::Children as ElementSeq<S, M, Renderer>>::Storage,
        _store: &Store<S>,
        _renderer: &mut Renderer,
    ) -> Self::State {
        let widget = self.build();
        let child = child.state().as_view_state().unwrap().as_ref();
        widget.set_child(Some(child));
        ButtonState::new(widget)
    }
}

impl<'event, Child, M> EventListener<'event> for Button<Child, M> {
    type Event = &'event Event;
}

#[derive(Debug)]
pub struct ButtonState {
    widget: gtk::Button,
    clicked_signal: Option<glib::SignalHandlerId>,
}

impl ButtonState {
    fn new(widget: gtk::Button) -> Self {
        Self {
            widget,
            clicked_signal: None,
        }
    }

    fn connect_clicked(&mut self, id_path: IdPathBuf, event_port: EventPort) {
        self.clicked_signal = self
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

    fn disconnect_clicked(&mut self) {
        if let Some(signal_id) = self.clicked_signal.take() {
            self.widget.disconnect(signal_id);
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
