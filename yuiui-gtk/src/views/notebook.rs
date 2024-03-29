use gtk::prelude::*;
use gtk::{gdk, glib};
use std::marker::PhantomData;
use yuiui_core::{
    CommitContext, ComponentStack, Element, ElementSeq, EventTarget, Lifecycle, Traversable, View,
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

impl<Children, S, M, E> View<S, M, E> for Notebook<Children>
where
    Children: ElementSeq<S, M, E>,
    Children::Storage: for<'a, 'context> Traversable<
        ReconcileChildrenVisitor<'a>,
        CommitContext<'context, S, M, E>,
    >,
{
    type Children = Children;

    type State = gtk::Notebook;

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

impl<'event, Children> EventTarget<'event> for Notebook<Children> {
    type Event = ();
}

#[derive(Debug, Clone)]
pub struct NotebookChild<Label, Content> {
    _phantom: PhantomData<(Label, Content)>,
}

impl<Label, Content> NotebookChild<Label, Content> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Label, Content, S, M, E> View<S, M, E> for NotebookChild<Label, Content>
where
    Label: Element<S, M, E>,
    Label::View: View<S, M, E>,
    <Label::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
    Content: Element<S, M, E>,
    Content::View: View<S, M, E>,
    <Content::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
{
    type Children = (Label, Content);

    type State = ();

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        _context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        ()
    }
}

impl<'event, Label, Content> EventTarget<'event> for NotebookChild<Label, Content> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::Notebook,
    current_child: Option<gtk::Widget>,
    index: u32,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::Notebook) -> Self {
        Self {
            container,
            current_child: container.first_child(),
            index: 0,
        }
    }
}

impl<'a, Label, Content, CS, S, M, E, Context>
    Visitor<ViewNode<NotebookChild<Label, Content>, CS, S, M, E>, Context>
    for ReconcileChildrenVisitor<'a>
where
    Label: Element<S, M, E>,
    Label::View: View<S, M, E>,
    <Label::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
    Content: Element<S, M, E>,
    Content::View: View<S, M, E>,
    <Content::View as View<S, M, E>>::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, E, View = NotebookChild<Label, Content>>,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<NotebookChild<Label, Content>, CS, S, M, E>,
        _context: &mut Context,
    ) {
        let new_label: &gtk::Widget = node.children().0.view_state().unwrap().as_ref();
        let new_child: &gtk::Widget = node.children().1.view_state().unwrap().as_ref();

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
                    self.container
                        .insert_page(new_child, Some(new_label), Some(self.index));
                    new_child.insert_before(self.container, Some(&child));
                    self.current_child = Some(child);
                    self.index += 1;
                    break;
                }
                None => {
                    self.container.append_page(new_child, Some(new_label));
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
            self.container.detach_tab(&current_child);
            self.current_child = current_child.next_sibling();
        }
    }
}
