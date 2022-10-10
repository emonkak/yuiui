use gtk::{gdk, gio, glib, pango, prelude::*};
use yuiui::{ElementSeq, IdPathBuf, Lifecycle, MessageContext, Store, View};
use yuiui_gtk_derive::WidgetBuilder;

use crate::renderer::{EventPort, Renderer};

#[derive(WidgetBuilder)]
#[widget(gtk::Entry)]
pub struct Entry<S, M> {
    activates_default: Option<bool>,
    attributes: Option<pango::AttrList>,
    buffer: Option<gtk::EntryBuffer>,
    completion: Option<gtk::EntryCompletion>,
    enable_emoji_completion: Option<bool>,
    extra_menu: Option<gio::MenuModel>,
    has_frame: Option<bool>,
    im_module: Option<String>,
    input_hints: Option<gtk::InputHints>,
    input_purpose: Option<gtk::InputPurpose>,
    invisible_char: Option<u32>,
    invisible_char_set: Option<bool>,
    max_length: Option<i32>,
    overwrite_mode: Option<bool>,
    placeholder_text: Option<String>,
    primary_icon_activatable: Option<bool>,
    primary_icon_gicon: Option<gio::Icon>,
    primary_icon_name: Option<String>,
    primary_icon_paintable: Option<gdk::Paintable>,
    primary_icon_sensitive: Option<bool>,
    primary_icon_tooltip_markup: Option<String>,
    primary_icon_tooltip_text: Option<String>,
    progress_fraction: Option<f64>,
    progress_pulse_step: Option<f64>,
    secondary_icon_activatable: Option<bool>,
    secondary_icon_gicon: Option<gio::Icon>,
    secondary_icon_name: Option<String>,
    secondary_icon_paintable: Option<gdk::Paintable>,
    secondary_icon_sensitive: Option<bool>,
    secondary_icon_tooltip_markup: Option<String>,
    secondary_icon_tooltip_text: Option<String>,
    show_emoji_icon: Option<bool>,
    tabs: Option<pango::TabArray>,
    truncate_multiline: Option<bool>,
    visibility: Option<bool>,
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
    editing_canceled: Option<bool>,
    editable: Option<bool>,
    enable_undo: Option<bool>,
    max_width_chars: Option<i32>,
    #[property(bind = false)]
    text: Option<String>,
    width_chars: Option<i32>,
    xalign: Option<f32>,
    #[property(bind = false)]
    on_activate: Option<Box<dyn Fn(&str, &S) -> M>>,
    #[property(bind = false)]
    on_change: Option<Box<dyn Fn(&str, &S) -> M>>,
}

impl<S, M> Entry<S, M> {
    fn update_text(&self, old_text: Option<&String>, widget: &gtk::Entry) {
        match (old_text, &self.text) {
            (Some(old_text), Some(new_text)) => {
                if old_text != new_text && widget.text() != new_text.as_str() {
                    widget.set_text(new_text);
                }
            }
            (Some(_), None) => {
                if !widget.text().is_empty() {
                    widget.set_text("");
                }
            }
            (None, Some(new_text)) => {
                if widget.text() != new_text.as_str() {
                    widget.set_text(new_text);
                }
            }
            (None, None) => {}
        }
    }
}

impl<S, M> View<S, M, Renderer> for Entry<S, M> {
    type Children = ();

    type State = EntryState;

    type Event = Event;

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
                if self.on_activate.is_some() {
                    state.connect_activate(
                        context.id_path().to_vec(),
                        renderer.event_port().clone(),
                    );
                }
                if self.on_change.is_some() {
                    state
                        .connect_changed(context.id_path().to_vec(), renderer.event_port().clone());
                }
            }
            Lifecycle::Update(old_view) => {
                match (&self.on_activate, &old_view.on_activate) {
                    (Some(_), None) => {
                        state.disconnect_activate();
                    }
                    (None, Some(_)) => {
                        state.connect_activate(
                            context.id_path().to_vec(),
                            renderer.event_port().clone(),
                        );
                    }
                    _ => {}
                }
                match (&self.on_change, &old_view.on_change) {
                    (Some(_), None) => {
                        state.disconnect_changed();
                    }
                    (None, Some(_)) => {
                        state.connect_changed(
                            context.id_path().to_vec(),
                            renderer.event_port().clone(),
                        );
                    }
                    _ => {}
                }
                state.block_changed_signal();
                self.update(&old_view, &state.widget);
                self.update_text(old_view.text.as_ref(), &state.widget);
                state.unblock_changed_signal();
            }
            Lifecycle::Unmount => {
                state.disconnect_changed();
            }
        }
    }

    fn event(
        &self,
        event: &Self::Event,
        _state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, Renderer>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        _renderer: &mut Renderer,
    ) {
        match event {
            Event::Activate(text) => {
                if let Some(on_activate) = &self.on_activate {
                    let message = on_activate(text.as_str(), store);
                    context.push_message(message);
                }
            }
            Event::Changed(text) => {
                if let Some(on_change) = &self.on_change {
                    let message = on_change(text.as_str(), store);
                    context.push_message(message);
                }
            }
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, Renderer>>::Storage,
        _store: &Store<S>,
        _renderer: &mut Renderer,
    ) -> Self::State {
        let widget = self.build();
        if let Some(text) = &self.text {
            widget.set_text(text);
        }
        EntryState::new(widget)
    }
}

#[derive(Debug)]
pub struct EntryState {
    widget: gtk::Entry,
    activate_signal: Option<glib::SignalHandlerId>,
    changed_signal: Option<glib::SignalHandlerId>,
}

impl EntryState {
    fn new(widget: gtk::Entry) -> Self {
        Self {
            widget,
            activate_signal: None,
            changed_signal: None,
        }
    }

    fn connect_activate(&mut self, id_path: IdPathBuf, event_port: EventPort) {
        self.changed_signal = self
            .widget
            .connect_activate(move |widget| {
                event_port
                    .send((id_path.clone(), Box::new(Event::Activate(widget.text()))))
                    .unwrap();
            })
            .into();
    }

    fn connect_changed(&mut self, id_path: IdPathBuf, event_port: EventPort) {
        self.changed_signal = self
            .widget
            .connect_changed(move |widget| {
                event_port
                    .send((id_path.clone(), Box::new(Event::Changed(widget.text()))))
                    .unwrap();
            })
            .into();
    }

    fn disconnect_activate(&mut self) {
        if let Some(signal_id) = self.activate_signal.take() {
            self.widget.disconnect(signal_id);
        }
    }

    fn disconnect_changed(&mut self) {
        if let Some(signal_id) = self.changed_signal.take() {
            self.widget.disconnect(signal_id);
        }
    }

    fn block_changed_signal(&mut self) {
        if let Some(signal_id) = &self.changed_signal {
            self.widget.block_signal(signal_id);
        }
    }

    fn unblock_changed_signal(&mut self) {
        if let Some(signal_id) = &self.changed_signal {
            self.widget.unblock_signal(signal_id);
        }
    }
}

impl AsRef<gtk::Widget> for EntryState {
    fn as_ref(&self) -> &gtk::Widget {
        self.widget.as_ref()
    }
}

#[derive(Debug)]
pub enum Event {
    Activate(glib::GString),
    Changed(glib::GString),
}
