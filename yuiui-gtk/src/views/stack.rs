use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui_core::{
    CommitContext, ComponentStack, Element, ElementSeq, EventTarget, Lifecycle, Traversable, View,
    ViewNode, ViewNodeSeq, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(WidgetBuilder, Debug, Clone)]
#[widget(gtk::Stack)]
pub struct Stack<Children> {
    hhomogeneous: Option<bool>,
    interpolate_size: Option<bool>,
    transition_duration: Option<u32>,
    transition_type: Option<gtk::StackTransitionType>,
    vhomogeneous: Option<bool>,
    visible_child_name: Option<String>,
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
    _phantom: PhantomData<Children>,
}

impl<Children, S, M, E> View<S, M, E> for Stack<Children>
where
    Children: ElementSeq<S, M, E>,
    Children::Storage: for<'a, 'context> Traversable<
        ReconcileChildrenVisitor<'a>,
        CommitContext<'context, S, M, E>,
    >,
{
    type Children = Children;

    type State = gtk::Stack;

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

impl<'event, Children> EventTarget<'event> for Stack<Children> {
    type Event = ();
}

#[derive(WidgetBuilder, Debug, Clone)]
#[widget(gtk::StackSwitcher)]
pub struct StackSwitcher<Child> {
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
    _phantom: PhantomData<Child>,
}

impl<Child, S, M, E> View<S, M, E> for StackSwitcher<Child>
where
    Child: Element<S, M, E>,
    Child::View: View<S, M, E, State = gtk::Stack>,
{
    type Children = Child;

    type State = StackSwitcherState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) {
        match lifecycle {
            Lifecycle::Update(old_view) => {
                self.update(&old_view, &view_state.stack_switcher);
            }
            _ => {}
        }
    }

    fn build(
        &self,
        child: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let stack_switcher = self.build();
        let stack = child.view_state().unwrap();
        container.append(&stack_switcher);
        container.append(stack);
        stack_switcher.set_stack(Some(stack));
        StackSwitcherState {
            container,
            stack_switcher,
        }
    }
}

impl<'event, Child> EventTarget<'event> for StackSwitcher<Child> {
    type Event = ();
}

#[derive(Debug)]
pub struct StackSwitcherState {
    container: gtk::Box,
    stack_switcher: gtk::StackSwitcher,
}

impl AsRef<gtk::Widget> for StackSwitcherState {
    fn as_ref(&self) -> &gtk::Widget {
        self.container.as_ref()
    }
}

#[derive(Debug, Clone, WidgetBuilder)]
#[widget(gtk::StackPage)]
pub struct StackPage<Child> {
    icon_name: Option<String>,
    name: Option<String>,
    needs_attention: Option<bool>,
    title: Option<String>,
    use_underline: Option<bool>,
    visible: Option<bool>,
    #[property(bind = false, setter = false)]
    _phantom: PhantomData<Child>,
}

impl<Child, S, M, E> View<S, M, E> for StackPage<Child>
where
    Child: Element<S, M, E>,
    Child::View: View<S, M, E>,
    <Child::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
{
    type Children = Child;

    type State = Option<gtk::StackPage>;

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        None
    }
}

impl<'event, Child> EventTarget<'event> for StackPage<Child> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::Stack,
    current_child: Option<gtk::Widget>,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::Stack) -> Self {
        Self {
            container,
            current_child: container.first_child(),
        }
    }
}

impl<'a, Child, CS, S, M, E, Context> Visitor<ViewNode<StackPage<Child>, CS, S, M, E>, Context>
    for ReconcileChildrenVisitor<'a>
where
    Child: Element<S, M, E>,
    Child::View: View<S, M, E>,
    <Child::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, E, View = StackPage<Child>>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<StackPage<Child>, CS, S, M, E>,
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
                    let stack_page = self.container.add_child(new_child);
                    new_child.insert_before(self.container, Some(&child));
                    node.view_mut().force_update(&stack_page);
                    *node.view_state_mut().unwrap() = Some(stack_page);
                    self.current_child = Some(child);
                    break;
                }
                None => {
                    let stack_page = self.container.add_child(new_child);
                    node.view_mut().force_update(&stack_page);
                    *node.view_state_mut().unwrap() = Some(stack_page);
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
