use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, EventListener, Lifecycle, MessageContext, Store, Traversable, View,
    ViewNode, ViewNodeSeq, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

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

impl<Children, S, M, R> View<S, M, R> for Grid<Children>
where
    Children: ElementSeq<S, M, R>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, M, R>,
{
    type Children = Children;

    type State = gtk::Grid;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        let is_dynamic = <Self::Children as ElementSeq<S, M, R>>::Storage::IS_DYNAMIC;
        let needs_reconcile = match lifecycle {
            Lifecycle::Mount => true,
            Lifecycle::Remount | Lifecycle::Unmount => is_dynamic,
            Lifecycle::Update(old_view) => {
                self.update(&old_view, state);
                is_dynamic
            }
        };
        if needs_reconcile {
            let mut visitor = ReconcileChildrenVisitor::new(state);
            children.for_each(&mut visitor, context, store, renderer);
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

impl<'event, Children> EventListener<'event> for Grid<Children> {
    type Event = ();
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
}

impl<Child, S, M, R> View<S, M, R> for GridChild<Child>
where
    Child: View<S, M, R>,
{
    type Children = Child::Children;

    type State = Child::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        let lifecycle = lifecycle.map(|view| view.child);
        self.child
            .lifecycle(lifecycle, state, children, context, store, renderer)
    }

    fn event(
        &self,
        event: <Self as EventListener>::Event,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        self.child
            .event(event, state, children, context, store, renderer)
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        store: &Store<S>,
        renderer: &mut R,
    ) -> Self::State {
        self.child.build(children, store, renderer)
    }
}

impl<'event, Child: EventListener<'event>> EventListener<'event> for GridChild<Child> {
    type Event = Child::Event;
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

impl<'a, V, CS, S, M, R> Visitor<ViewNode<GridChild<V>, CS, S, M, R>, S, R>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, R>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, R, View = GridChild<V>>,
    GridChild<V>: View<S, M, R, Children = V::Children, State = V::State>,
{
    type Context = MessageContext<M>;

    type Output = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<GridChild<V>, CS, S, M, R>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::Output {
        let new_child: &gtk::Widget = node.state().as_view_state().unwrap().as_ref();
        loop {
            match self.current_child.take() {
                Some(child) if new_child == &child => {
                    self.current_child = child.next_sibling();
                    break;
                }
                Some(child) if new_child.parent().is_some() => {
                    self.current_child = child.next_sibling();
                    self.container.remove(&child);
                }
                Some(child) => {
                    let grid_cell = &node.state().as_view().grid_cell;
                    self.container.attach(
                        new_child,
                        grid_cell.column,
                        grid_cell.row,
                        grid_cell.column_span,
                        grid_cell.row_span,
                    );
                    new_child.insert_before(self.container, Some(&child));
                    self.current_child = Some(child);
                    break;
                }
                None => {
                    let grid_cell = &node.state().as_view().grid_cell;
                    self.container.attach(
                        new_child,
                        grid_cell.column,
                        grid_cell.row,
                        grid_cell.column_span,
                        grid_cell.row_span,
                    );
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
