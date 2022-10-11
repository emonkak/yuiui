use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, IdContext, Lifecycle, Store, Traversable, View, ViewNode,
    ViewNodeSeq, Visitor,
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
    Children::Storage: for<'a> Traversable<ReconcileChildrenVisitor<'a>, (), S, M, R>,
{
    type Children = Children;

    type State = gtk::Grid;

    type Event = ();

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        id_context: &mut IdContext,
        store: &Store<S>,
        _messages: &mut Vec<M>,
        renderer: &mut R,
    ) {
        let is_static: bool = <Self::Children as ElementSeq<S, M, R>>::Storage::IS_STATIC;
        let needs_reconcile = match lifecycle {
            Lifecycle::Mount => true,
            Lifecycle::Remount | Lifecycle::Unmount => !is_static,
            Lifecycle::Update(old_view) => {
                self.update(&old_view, state);
                !is_static
            }
        };
        if needs_reconcile {
            let mut visitor = ReconcileChildrenVisitor::new(state);
            children.for_each(&mut visitor, &mut (), id_context, store, renderer);
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

#[derive(Debug, Clone)]
pub struct GridChild<Child> {
    child: Child,
    column: i32,
    row: i32,
    column_span: i32,
    row_span: i32,
}

impl<Child> GridChild<Child> {
    pub fn new(child: Child, column: i32, row: i32, column_span: i32, row_span: i32) -> Self {
        Self {
            child,
            column,
            row,
            column_span,
            row_span,
        }
    }
}

impl<Child, S, M, R> View<S, M, R> for GridChild<Child>
where
    Child: View<S, M, R>,
{
    type Children = Child::Children;

    type State = Child::State;

    type Event = Child::Event;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) {
        let lifecycle = lifecycle.map(|view| view.child);
        self.child.lifecycle(
            lifecycle, state, children, id_context, store, messages, renderer,
        )
    }

    fn event(
        &self,
        event: &Self::Event,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        renderer: &mut R,
    ) {
        self.child.event(
            event, state, children, id_context, store, messages, renderer,
        )
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

impl<'a, V, CS, S, M, R> Visitor<ViewNode<GridChild<V>, CS, S, M, R>, S, M, R>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, R>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, R, View = GridChild<V>>,
    GridChild<V>: View<S, M, R, Children = V::Children, State = V::State>,
{
    type Accumulator = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<GridChild<V>, CS, S, M, R>,
        _accumulator: &mut Self::Accumulator,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _renderer: &mut R,
    ) {
        let new_child: &gtk::Widget = node.state().unwrap().as_ref();
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
                    let grid_child = node.view();
                    self.container.attach(
                        new_child,
                        grid_child.column,
                        grid_child.row,
                        grid_child.column_span,
                        grid_child.row_span,
                    );
                    new_child.insert_before(self.container, Some(&child));
                    self.current_child = Some(child);
                    break;
                }
                None => {
                    let grid_child = &node.view();
                    self.container.attach(
                        new_child,
                        grid_child.column,
                        grid_child.row,
                        grid_child.column_span,
                        grid_child.row_span,
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
