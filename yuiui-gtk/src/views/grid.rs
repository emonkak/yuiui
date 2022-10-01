use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, EventListener, Lifecycle, MessageContext, Store, Traversable, View,
    ViewNode, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

use crate::backend::GtkBackend;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::Grid)]
pub struct Grid<Children> {
    baseline_row: Option<i32>,
    column_homogeneous: Option<bool>,
    column_spacing: Option<i32>,
    row_homogeneous: Option<bool>,
    row_spacing: Option<i32>,
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

impl<Children, S, M> View<S, M, GtkBackend> for Grid<Children>
where
    Children: ElementSeq<S, M, GtkBackend>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, GtkBackend>,
{
    type Children = Children;

    type State = gtk::Grid;

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

impl<'event, Children> EventListener<'event> for Grid<Children> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::Grid,
    current_child: Option<gtk::Widget>,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::Grid) -> Self {
        Self {
            container,
            current_child: container.first_child(),
        }
    }
}

impl<'a, V, CS, S, M, B> Visitor<ViewNode<GridChild<V>, CS, S, M, B>, S, B>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, B>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, B, View = GridChild<V>>,
    GridChild<V>: View<S, M, B, Children = V::Children, State = V::State>,
{
    type Context = MessageContext<M>;

    type Output = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<GridChild<V>, CS, S, M, B>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::Output {
        let new_child: &gtk::Widget = node.state().as_view_state().unwrap().as_ref();
        loop {
            match &self.current_child {
                Some(child) if new_child == child => {
                    self.current_child = child.next_sibling();
                    break;
                }
                Some(child) if new_child.parent().is_some() => {
                    self.container.remove(new_child);
                    self.current_child = child.next_sibling();
                }
                Some(child) => {
                    let prev_sibling = child.prev_sibling();
                    new_child.insert_after(self.container, prev_sibling.as_ref());
                    node.state()
                        .as_view()
                        .grid_cell
                        .attach_grid_child(self.container, new_child);
                    break;
                }
                None => {
                    new_child.set_parent(self.container);
                    node.state()
                        .as_view()
                        .grid_cell
                        .attach_grid_child(self.container, new_child);
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

#[derive(Debug, Clone)]
pub struct GridChild<Child> {
    child: Child,
    grid_cell: GridCell,
}

impl<Child> GridChild<Child> {
    pub fn new(child: Child, grid_cell: GridCell) -> Self {
        Self { child, grid_cell }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GridCell {
    column: i32,
    row: i32,
    column_span: i32,
    row_span: i32,
}

impl GridCell {
    pub fn new(column: i32, row: i32, column_span: i32, row_span: i32) -> Self {
        Self {
            column,
            row,
            column_span,
            row_span,
        }
    }

    pub fn column(&self) -> i32 {
        self.column
    }

    pub fn row(&self) -> i32 {
        self.row
    }

    pub fn column_span(&self) -> i32 {
        self.column_span
    }

    pub fn row_span(&self) -> i32 {
        self.row_span
    }

    fn attach_grid_child(&self, container: &gtk::Grid, child: &gtk::Widget) {
        let layout_manager = container.layout_manager().expect("get layout manager");
        let layout_child: gtk::GridLayoutChild = layout_manager
            .layout_child(child)
            .downcast()
            .expect("downcast the widget to GridLayoutChild");
        layout_child.set_column(self.column);
        layout_child.set_column_span(self.column_span);
        layout_child.set_row(self.row);
        layout_child.set_row_span(self.row_span);
    }
}

impl<Child, S, M, B> View<S, M, B> for GridChild<Child>
where
    Child: View<S, M, B>,
{
    type Children = Child::Children;

    type State = Child::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        let lifecycle = lifecycle.map(|view| view.child);
        self.child
            .lifecycle(lifecycle, view_state, children, context, store, backend)
    }

    fn event(
        &self,
        event: <Self as EventListener>::Event,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        self.child
            .event(event, view_state, children, context, store, backend)
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::State {
        self.child.build(children, store, backend)
    }
}

impl<'event, Child: EventListener<'event>> EventListener<'event> for GridChild<Child> {
    type Event = Child::Event;
}
