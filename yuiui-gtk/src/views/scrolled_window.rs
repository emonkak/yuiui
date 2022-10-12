use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui::{Element, ElementSeq, EventTarget, IdContext, Lifecycle, Store, View};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(WidgetBuilder)]
#[widget(gtk::ScrolledWindow)]
pub struct ScrolledWindow<Child> {
    hadjustment: Option<gtk::Adjustment>,
    has_frame: Option<bool>,
    hscrollbar_policy: Option<gtk::PolicyType>,
    kinetic_scrolling: Option<bool>,
    max_content_height: Option<i32>,
    max_content_width: Option<i32>,
    min_content_height: Option<i32>,
    min_content_width: Option<i32>,
    overlay_scrolling: Option<bool>,
    propagate_natural_height: Option<bool>,
    propagate_natural_width: Option<bool>,
    vadjustment: Option<gtk::Adjustment>,
    vscrollbar_policy: Option<gtk::PolicyType>,
    window_placement: Option<gtk::CornerType>,
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
    #[property(bind = false, setter = false)]
    _phantom: PhantomData<Child>,
}

impl<Child, S, M, R> View<S, M, R> for ScrolledWindow<Child>
where
    Child: Element<S, M, R>,
    <Child::View as View<S, M, R>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = gtk::ScrolledWindow;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _renderer: &mut R,
    ) {
        match lifecycle {
            Lifecycle::Mount | Lifecycle::Remount => {}
            Lifecycle::Update(old_view) => {
                self.update(&old_view, state);
            }
            Lifecycle::Unmount => {}
        }
    }

    fn build(
        &self,
        child: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::State {
        let widget = self.build();
        let child = child.state().unwrap().as_ref();
        widget.set_child(Some(child));
        widget
    }
}

impl<'event, Child> EventTarget<'event> for ScrolledWindow<Child> {
    type Event = ();
}
