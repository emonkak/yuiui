use gtk::{gdk, gio, glib, pango, prelude::*};
use yuiui::{ElementSeq, EventListener, Lifecycle, MessageContext, Store, View};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::Entry)]
pub struct Entry {
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
    text: Option<String>,
    width_chars: Option<i32>,
    xalign: Option<f32>,
}

impl<S, M, R> View<S, M, R> for Entry {
    type Children = ();

    type State = gtk::Entry;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) {
        match lifecycle {
            Lifecycle::Update(old_view) => {
                self.update(&old_view, state);
            }
            _ => {}
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::State {
        self.build()
    }
}

impl<'event> EventListener<'event> for Entry {
    type Event = ();
}
