use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui::{
    CommitContext, ComponentStack, Element, ElementSeq, EventTarget, IdStack, Lifecycle, Store,
    Traversable, View, ViewNode, ViewNodeSeq, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

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

impl<Children, S, M, E> View<S, M, E> for FlowBox<Children>
where
    Children: ElementSeq<S, M, E>,
    Children::Storage: for<'a, 'context> Traversable<
        ReconcileChildrenVisitor<'a>,
        CommitContext<'context, S, M, E>,
        S,
        M,
        E,
    >,
{
    type Children = Children;

    type State = gtk::FlowBox;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) {
        let is_static = <Self::Children as ElementSeq<S, M, E>>::Storage::IS_STATIC;
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
            let mut context = CommitContext {
                store,
                messages,
                entry_point,
            };
            children.for_each(&mut visitor, &mut context, id_stack);
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _store: &Store<S>,
        _entry_point: &E,
    ) -> Self::State {
        self.build()
    }
}

impl<'event, Children> EventTarget<'event> for FlowBox<Children> {
    type Event = ();
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

impl<Child, S, M, E> View<S, M, E> for FlowBoxChild<Child>
where
    Child: Element<S, M, E>,
    <Child::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = gtk::FlowBoxChild;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        state: &mut Self::State,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _id_stack: &mut IdStack,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _entry_point: &E,
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
        child: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _store: &Store<S>,
        _entry_point: &E,
    ) -> Self::State {
        let container = self.build();
        let child = child.state().unwrap();
        container.set_child(Some(child.as_ref()));
        container
    }
}

impl<'event, Child> EventTarget<'event> for FlowBoxChild<Child> {
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

impl<'a, V, CS, S, M, E, Context> Visitor<ViewNode<V, CS, S, M, E>, Context, S, M, E>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, E, State = gtk::FlowBoxChild>,
    CS: ComponentStack<S, M, E, View = V>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, E>,
        _context: &mut Context,
        _id_stack: &mut IdStack,
    ) {
        let new_child = node.state().unwrap();
        loop {
            match self.current_child.take() {
                Some(child) if new_child == &child => {
                    self.current_child = child.next_sibling();
                    self.index += 1;
                    break;
                }
                Some(child) if new_child.parent().is_some() => {
                    self.current_child = child.next_sibling();
                    self.container.remove(new_child);
                    self.index += 1;
                }
                Some(child) => {
                    self.current_child = Some(child);
                    self.container.insert(new_child, self.index);
                    self.index += 1;
                    break;
                }
                None => {
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
