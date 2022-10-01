use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, Element, ElementSeq, EventListener, Lifecycle, MessageContext, Store,
    Traversable, View, ViewNode, ViewNodeSeq, Visitor,
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

impl<Children, S, M, B> View<S, M, B> for Stack<Children>
where
    Children: ElementSeq<S, M, B>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, B>,
{
    type Children = Children;

    type State = gtk::Stack;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        let is_dynamic = <Self::Children as ElementSeq<S, M, B>>::Storage::IS_DYNAMIC;
        let needs_reconcile = match lifecycle {
            Lifecycle::Mount => true,
            Lifecycle::Remount | Lifecycle::Unmount => is_dynamic,
            Lifecycle::Update(old_view) => {
                self.update(&old_view, view_state);
                is_dynamic
            }
        };
        if needs_reconcile {
            let mut visitor = ReconcileChildrenVisitor::new(view_state);
            children.for_each(&mut visitor, context, store, backend);
        }
    }

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::State {
        self.build()
    }
}

impl<'event, Children> EventListener<'event> for Stack<Children> {
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

impl<Child, S, M, B> View<S, M, B> for StackSwitcher<Child>
where
    Child: Element<S, M, B>,
    Child::View: View<S, M, B, State = gtk::Stack>,
{
    type Children = Child;

    type State = StackSwitcherState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        _child: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
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
        child: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::State {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        let stack_switcher = self.build();
        let stack = child.state().as_view_state().unwrap();
        container.append(&stack_switcher);
        container.append(stack);
        stack_switcher.set_stack(Some(stack));
        StackSwitcherState {
            container,
            stack_switcher,
        }
    }
}

impl<'event, Children> EventListener<'event> for StackSwitcher<Children> {
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
    #[property(argument = true, bind = false, setter = false)]
    child: Child,
    icon_name: Option<String>,
    name: Option<String>,
    needs_attention: Option<bool>,
    title: Option<String>,
    use_underline: Option<bool>,
    visible: Option<bool>,
}

impl<Child, S, M, B> View<S, M, B> for StackPage<Child>
where
    Child: View<S, M, B>,
{
    type Children = Child::Children;

    type State = StackPageState<Child::State>;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &mut B,
    ) {
        match &lifecycle {
            Lifecycle::Update(old_view) => {
                if let Some(stack_page) = &view_state.stack_page {
                    self.update(old_view, stack_page);
                }
            }
            _ => {}
        }
        let lifecycle = lifecycle.map(|view| view.child);
        self.child.lifecycle(
            lifecycle,
            &mut view_state.child_state,
            children,
            context,
            store,
            backend,
        )
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
        self.child.event(
            event,
            &mut view_state.child_state,
            children,
            context,
            store,
            backend,
        )
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, B>>::Storage,
        store: &Store<S>,
        backend: &mut B,
    ) -> Self::State {
        let child_state = self.child.build(children, store, backend);
        StackPageState::new(child_state)
    }
}

impl<'event, Child: EventListener<'event>> EventListener<'event> for StackPage<Child> {
    type Event = Child::Event;
}

#[derive(Debug)]
pub struct StackPageState<State> {
    child_state: State,
    stack_page: Option<gtk::StackPage>,
}

impl<State> StackPageState<State> {
    fn new(child_state: State) -> Self {
        Self {
            child_state,
            stack_page: None,
        }
    }
}

impl<State: AsRef<gtk::Widget>> AsRef<gtk::Widget> for StackPageState<State> {
    fn as_ref(&self) -> &gtk::Widget {
        self.child_state.as_ref()
    }
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

impl<'a, V, CS, S, M, B> Visitor<ViewNode<StackPage<V>, CS, S, M, B>, S, B>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, B>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, B, View = StackPage<V>>,
    StackPage<V>: View<S, M, B, Children = V::Children, State = StackPageState<V::State>>,
{
    type Context = MessageContext<M>;

    type Output = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<StackPage<V>, CS, S, M, B>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::Output {
        let (view, view_state) = node.state_mut().extract_mut();
        let new_child: &gtk::Widget = view_state.as_ref().unwrap().child_state.as_ref();
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
                    view.force_update(&stack_page);
                    view_state.unwrap().stack_page = Some(stack_page);
                    self.current_child = Some(child);
                    break;
                }
                None => {
                    let stack_page = self.container.add_child(new_child);
                    view.force_update(&stack_page);
                    view_state.unwrap().stack_page = Some(stack_page);
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
