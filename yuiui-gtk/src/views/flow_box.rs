use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, EventListener, Lifecycle, MessageContext, Store, Traversable, View,
    ViewNode, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::backend::GtkBackend;
use crate::element::GtkElement;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::FlowBox)]
pub struct FlowBox<Children> {
    accept_unpaired_release: Option<bool>,
    activate_on_single_click: Option<bool>,
    column_spacing: Option<u32>,
    homogeneous: Option<bool>,
    max_children_per_line: Option<u32>,
    min_children_per_line: Option<u32>,
    row_spacing: Option<u32>,
    selection_mode: Option<gtk::SelectionMode>,
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
    #[property(bind = false, setter = false)]
    _phantom: PhantomData<Children>,
}

impl<Children, S, M> View<S, M, GtkBackend> for FlowBox<Children>
where
    Children: ElementSeq<S, M, GtkBackend>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, GtkBackend>,
{
    type Children = Children;

    type State = gtk::FlowBox;

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
            Lifecycle::Update(old_view) => {
                self.update(&old_view, view_state);
            }
            _ => {}
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

impl<'event, Children> EventListener<'event> for FlowBox<Children> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::FlowBox,
    current_child: Option<gtk::Widget>,
    index: i32,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::FlowBox) -> Self {
        Self {
            container,
            current_child: container.first_child(),
            index: 0,
        }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<V, CS, S, M, B>, S, B> for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, B, State = gtk::FlowBoxChild>,
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
        let new_child = node.state().as_view_state().unwrap();
        loop {
            match &self.current_child {
                Some(child) if new_child == child => {
                    self.current_child = child.next_sibling();
                    self.index += 1;
                    break;
                }
                Some(child) if new_child.parent().is_some() => {
                    self.container.remove(new_child);
                    self.current_child = child.next_sibling();
                    self.index += 1;
                }
                Some(_) | None => {
                    self.container.insert(new_child, self.index);
                    self.index += 1;
                    break;
                }
            }
        }
    }
}

impl<'a> Drop for ReconcileChildrenVisitor<'a> {
    fn drop(&mut self) {
        while let Some(current_child) = self.current_child.take() {
            self.container.remove(&current_child);
            self.current_child = current_child.next_sibling();
        }
    }
}

#[derive(Debug, WidgetBuilder)]
#[widget(gtk::FlowBoxChild)]
pub struct FlowBoxChild<Child> {
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

impl<Child, S, M> View<S, M, GtkBackend> for FlowBoxChild<Child>
where
    Child: GtkElement<S, M>,
{
    type Children = Child;

    type State = gtk::FlowBoxChild;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut GtkBackend,
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
        child: &mut <Self::Children as ElementSeq<S, M, GtkBackend>>::Storage,
        _store: &Store<S>,
        _backend: &mut GtkBackend,
    ) -> Self::State {
        let container = self.build();
        let child = child.state().as_view_state().unwrap();
        container.set_child(Some(child.as_ref()));
        container
    }
}

impl<'event, Child> EventListener<'event> for FlowBoxChild<Child> {
    type Event = ();
}
