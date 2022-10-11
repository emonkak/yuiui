use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui::{
    CommitContext, ComponentStack, ElementSeq, IdContext, Lifecycle, Store, Traversable, View,
    ViewNode, ViewNodeSeq, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::Notebook)]
pub struct Notebook<Children> {
    enable_popup: Option<bool>,
    group_name: Option<String>,
    page: Option<i32>,
    scrollable: Option<bool>,
    show_border: Option<bool>,
    show_tabs: Option<bool>,
    tab_pos: Option<gtk::PositionType>,
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

impl<Children, S, M, R> View<S, M, R> for Notebook<Children>
where
    Children: ElementSeq<S, M, R>,
    Children::Storage: for<'a, 'context> Traversable<
        ReconcileChildrenVisitor<'a>,
        CommitContext<'context, S, M, R>,
        S,
        M,
        R,
    >,
{
    type Children = Children;

    type State = gtk::Notebook;

    type Event = ();

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
            let mut context = CommitContext {
                store,
                messages,
                renderer,
            };
            children.for_each(&mut visitor, &mut context, id_context);
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
pub struct NotebookChild<Child> {
    child: Child,
    child_type: NotebookChildType,
}

impl<Child> NotebookChild<Child> {
    pub fn from_tab(child: Child) -> Self {
        Self {
            child,
            child_type: NotebookChildType::TabLabel,
        }
    }

    pub fn from_content(child: Child) -> Self {
        Self {
            child,
            child_type: NotebookChildType::Content,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotebookChildType {
    TabLabel,
    Content,
}

impl<Child, S, M, R> View<S, M, R> for NotebookChild<Child>
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
    container: &'a gtk::Notebook,
    current_child: Option<gtk::Widget>,
    current_tab: Option<gtk::Widget>,
    index: u32,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::Notebook) -> Self {
        Self {
            container,
            current_child: container.first_child(),
            current_tab: None,
            index: 0,
        }
    }
}

impl<'a, V, CS, S, M, R, Context> Visitor<ViewNode<NotebookChild<V>, CS, S, M, R>, Context, S, M, R>
    for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, R>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, R, View = NotebookChild<V>>,
    NotebookChild<V>: View<S, M, R, Children = V::Children, State = V::State>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<NotebookChild<V>, CS, S, M, R>,
        _context: &mut Context,
        _id_context: &mut IdContext,
    ) {
        match node.view().child_type {
            NotebookChildType::TabLabel => {
                let new_child: &gtk::Widget = node.state().unwrap().as_ref();
                self.current_tab = Some(new_child.clone());
            }
            NotebookChildType::Content => {
                let new_child: &gtk::Widget = node.state().unwrap().as_ref();
                loop {
                    match self.current_child.take() {
                        Some(child) if new_child == &child => {
                            self.current_child = child.next_sibling();
                            self.index += 1;
                            break;
                        }
                        Some(child) if new_child.parent().is_some() => {
                            self.current_child = child.next_sibling();
                            self.container.detach_tab(&child);
                            self.index += 1;
                        }
                        Some(child) => {
                            let new_tab = self.current_tab.take();
                            self.container.insert_page(
                                new_child,
                                new_tab.as_ref(),
                                Some(self.index),
                            );
                            new_child.insert_before(self.container, Some(&child));
                            self.current_child = Some(child);
                            self.index += 1;
                            break;
                        }
                        None => {
                            let new_tab = self.current_tab.take();
                            self.container.append_page(new_child, new_tab.as_ref());
                            self.index += 1;
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Drop for ReconcileChildrenVisitor<'a> {
    fn drop(&mut self) {
        while let Some(current_child) = self.current_child.take() {
            self.container.detach_tab(&current_child);
            self.current_child = current_child.next_sibling();
        }
    }
}
