use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, EventListener, Lifecycle, MessageContext, Store, Traversable, View,
    ViewNode, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::backend::GtkBackend;

#[derive(Debug, WidgetBuilder)]
#[widget(gtk::Box)]
pub struct Box<Children> {
    baseline_position: Option<gtk::BaselinePosition>,
    homogeneous: Option<bool>,
    spacing: Option<i32>,
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
    orientation: Option<gtk::Orientation>,
    #[property(bind = false)]
    _phantom: PhantomData<Children>,
}

impl<Children, S, M> View<S, M, GtkBackend> for Box<Children>
where
    Children: ElementSeq<S, M, GtkBackend>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, GtkBackend>,
{
    type Children = Children;

    type State = gtk::Box;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut GtkBackend,
    ) {
        match lifecycle {
            Lifecycle::Mount => {}
            Lifecycle::Update(old_view) => {
                self.update(&old_view, view_state);
            }
            Lifecycle::Unmount => {}
        }
        let mut visitor = ReconcileChildrenVisitor::new(view_state);
        children.for_each(&mut visitor, context, store, backend);
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        _store: &Store<S>,
        _backend: &mut GtkBackend,
    ) -> Self::State {
        self.build()
    }
}

impl<'event, Children> EventListener<'event> for Box<Children> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::Box,
    current_widget: Option<gtk::Widget>,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::Box) -> Self {
        Self {
            container,
            current_widget: container.first_child(),
        }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, B>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, B, View = V>,
{
    type Context = MessageContext<M>;

    type Output = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, B>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::Output {
        let new_widget = node.state().as_view_state().unwrap().as_ref();
        if let Some(current_widget) = &self.current_widget {
            if new_widget == current_widget {
                self.current_widget = current_widget.next_sibling();
            } else {
                let prev_sibling = current_widget.prev_sibling();
                if let Some(parent) = new_widget.parent() {
                    assert!(&parent == self.container);
                    self.container
                        .reorder_child_after(new_widget, prev_sibling.as_ref());
                } else {
                    self.container
                        .insert_child_after(new_widget, prev_sibling.as_ref());
                }
            }
        } else {
            self.container.append(new_widget);
        }
    }
}

impl<'a> Drop for ReconcileChildrenVisitor<'a> {
    fn drop(&mut self) {
        while let Some(current_widget) = self.current_widget.take() {
            self.container.remove(&current_widget);
            self.current_widget = current_widget.next_sibling();
        }
    }
}
