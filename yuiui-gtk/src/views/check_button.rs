use gtk::prelude::*;
use gtk::{gdk, glib};
use yuiui::{CommitContext, ElementSeq, EventTarget, Lifecycle, View};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::CheckButton)]
pub struct CheckButton {
    active: Option<bool>,
    group: Option<gtk::CheckButton>,
    inconsistent: Option<bool>,
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

impl<S, M, E> View<S, M, E> for CheckButton {
    type Children = ();

    type State = gtk::CheckButton;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) {
        match lifecycle {
            Lifecycle::Update(old_view) => {
                self.update(&old_view, view_state);
            }
            _ => {}
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        self.build()
    }
}

impl<'event> EventTarget<'event> for CheckButton {
    type Event = ();
}
