use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};
use yuiui::{ElementSeq, EventTarget, IdPathBuf, IdStack, Lifecycle, Store, View};
use yuiui_gtk_derive::WidgetBuilder;

use crate::entry_point::EntryPoint;

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
    on_activate: Option<Box<dyn Fn(&str, &S) -> Option<M>>>,
    #[property(bind = false)]
    on_change: Option<Box<dyn Fn(&str, &S) -> Option<M>>>,
}

impl<S, M> View<S, M, EntryPoint> for Entry<S, M> {
    type Children = ();

    type State = EntryState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, EntryPoint>>::Storage,
        id_stack: &mut IdStack,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        entry_point: &EntryPoint,
    ) {
        match lifecycle {
            Lifecycle::Mount | Lifecycle::Remount => {
                if self.on_activate.is_some() {
                    view_state.connect_activate(id_stack.id_path().to_vec(), entry_point.clone());
                }
                if self.on_change.is_some() {
                    view_state.connect_changed(id_stack.id_path().to_vec(), entry_point.clone());
                }
            }
            Lifecycle::Update(old_view) => {
                match (&self.on_activate, &old_view.on_activate) {
                    (Some(_), None) => {
                        view_state.disconnect_activate();
                    }
                    (None, Some(_)) => {
                        view_state
                            .connect_activate(id_stack.id_path().to_vec(), entry_point.clone());
                    }
                    _ => {}
                }
                match (&self.on_change, &old_view.on_change) {
                    (Some(_), None) => {
                        view_state.disconnect_changed();
                    }
                    (None, Some(_)) => {
                        view_state
                            .connect_changed(id_stack.id_path().to_vec(), entry_point.clone());
                    }
                    _ => {}
                }
                self.update(&old_view, &view_state.widget);
                view_state.update_text(self.text.as_deref());
            }
            Lifecycle::Unmount => {
                view_state.disconnect_activate();
                view_state.disconnect_changed();
            }
        }
    }

    fn event(
        &self,
        event: <Self as EventTarget>::Event,
        view_state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, EntryPoint>>::Storage,
        _id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        _entry_point: &EntryPoint,
    ) {
        match event {
            Event::Activate => {
                if let Some(on_activate) = &self.on_activate {
                    let message = on_activate(view_state.current_text.as_str(), store.state());
                    messages.extend(message);
                }
            }
            Event::Changed => {
                if let Some(on_change) = &self.on_change {
                    view_state.refresh_text();
                    let message = on_change(view_state.current_text.as_str(), store.state());
                    messages.extend(message);
                }
            }
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, EntryPoint>>::Storage,
        _store: &Store<S>,
        _entry_point: &EntryPoint,
    ) -> Self::State {
        let widget = self.build();
        if let Some(text) = &self.text {
            widget.set_text(text);
        }
        EntryState::new(widget)
    }
}

impl<'event, S, M> EventTarget<'event> for Entry<S, M> {
    type Event = &'event Event;
}

#[derive(Debug)]
pub struct EntryState {
    widget: gtk::Entry,
    current_text: glib::GString,
    activate_signal: Option<glib::SignalHandlerId>,
    changed_signal: Option<glib::SignalHandlerId>,
}

impl EntryState {
    fn new(widget: gtk::Entry) -> Self {
        let current_text = widget.text();
        Self {
            widget,
            current_text,
            activate_signal: None,
            changed_signal: None,
        }
    }

    fn connect_activate(&mut self, id_path: IdPathBuf, entry_point: EntryPoint) {
        self.changed_signal = self
            .widget
            .connect_activate(move |_| {
                entry_point.dispatch_event(id_path.clone(), Event::Activate);
            })
            .into();
    }

    fn connect_changed(&mut self, id_path: IdPathBuf, entry_point: EntryPoint) {
        self.changed_signal = self
            .widget
            .connect_changed(move |_| {
                entry_point.dispatch_event(id_path.clone(), Event::Changed);
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

    fn refresh_text(&mut self) {
        self.current_text = self.widget.text();
    }

    fn update_text(&mut self, new_text: Option<&str>) {
        if let Some(signal_id) = &self.changed_signal {
            self.widget.block_signal(signal_id);
        }
        match new_text {
            Some(new_text) => {
                if new_text != self.current_text {
                    self.widget.set_text(new_text);
                    self.current_text = new_text.into();
                }
            }
            None => {
                if !self.current_text.is_empty() {
                    self.widget.set_text("");
                    self.current_text = "".into();
                }
            }
        }
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
    Activate,
    Changed,
}
