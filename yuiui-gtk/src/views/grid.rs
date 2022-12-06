use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui_core::{
    CommitContext, ComponentStack, Element, ElementSeq, EventTarget, Lifecycle, Traversable, View,
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

impl<Children, S, M, E> View<S, M, E> for Grid<Children>
where
    Children: ElementSeq<S, M, E>,
    Children::Storage: for<'a, 'context> Traversable<
        ReconcileChildrenVisitor<'a>,
        CommitContext<'context, S, M, E>,
    >,
{
    type Children = Children;

    type State = gtk::Grid;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let is_static = <Self::Children as ElementSeq<S, M, E>>::Storage::IS_STATIC;
        let needs_reconcile = match lifecycle {
            Lifecycle::Mount => true,
            Lifecycle::Remount | Lifecycle::Unmount => !is_static,
            Lifecycle::Update(old_view) => {
                self.update(&old_view, view_state);
                !is_static
            }
        };
        if needs_reconcile {
            let mut visitor = ReconcileChildrenVisitor::new(view_state);
            children.for_each(&mut visitor, context);
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

impl<'event, Children> EventTarget<'event> for Grid<Children> {
    type Event = ();
}

#[derive(Debug, Clone)]
pub struct GridChild<Child> {
    column: i32,
    row: i32,
    column_span: i32,
    row_span: i32,
    _phantom: PhantomData<Child>,
}

impl<Child> GridChild<Child> {
    pub fn new(column: i32, row: i32, column_span: i32, row_span: i32) -> Self {
        Self {
            column,
            row,
            column_span,
            row_span,
            _phantom: PhantomData,
        }
    }
}

impl<Child, S, M, E> View<S, M, E> for GridChild<Child>
where
    Child: Element<S, M, E>,
    Child::View: View<S, M, E>,
    <Child::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = ();

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        ()
    }
}

impl<'event, Child> EventTarget<'event> for GridChild<Child> {
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

impl<'a, Child, CS, S, M, E, Context> Visitor<ViewNode<GridChild<Child>, CS, S, M, E>, Context>
    for ReconcileChildrenVisitor<'a>
where
    Child: Element<S, M, E>,
    Child::View: View<S, M, E>,
    <Child::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, E, View = GridChild<Child>>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<GridChild<Child>, CS, S, M, E>,
        _context: &mut Context,
    ) {
        let new_child: &gtk::Widget = node.children().view_state().unwrap().as_ref();
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
