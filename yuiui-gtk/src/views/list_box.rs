use gtk::{gdk, glib, prelude::*};
use std::marker::PhantomData;
use yuiui::{
    ComponentStack, ElementSeq, EventListener, Lifecycle, MessageContext, Store, Traversable, View,
    ViewNode, ViewNodeSeq, Visitor,
};
use yuiui_gtk_derive::WidgetBuilder;

#[derive(Clone, Debug, WidgetBuilder)]
#[widget(gtk::ListBox)]
pub struct ListBox<Children> {
    accept_unpaired_release: Option<bool>,
    activate_on_single_click: Option<bool>,
    selection_mode: Option<gtk::SelectionMode>,
    show_separators: Option<bool>,
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

impl<Children, S, M, R> View<S, M, R> for ListBox<Children>
where
    Children: ElementSeq<S, M, R>,
    Children::Storage:
        for<'a> Traversable<ReconcileChildrenVisitor<'a>, MessageContext<M>, (), S, M, R>,
{
    type Children = Children;

    type State = gtk::ListBox;

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

impl<'event, Children> EventListener<'event> for ListBox<Children> {
    type Event = ();
}

pub struct ReconcileChildrenVisitor<'a> {
    container: &'a gtk::ListBox,
    current_child: Option<gtk::Widget>,
    index: i32,
}

impl<'a> ReconcileChildrenVisitor<'a> {
    fn new(container: &'a gtk::ListBox) -> Self {
        Self {
            container,
            current_child: container.first_child(),
            index: 0,
        }
    }
}

impl<'a, V, CS, S, M, R> Visitor<ViewNode<V, CS, S, M, R>, S, R> for ReconcileChildrenVisitor<'a>
where
    V: View<S, M, R>,
    V::State: AsRef<gtk::Widget>,
    CS: ComponentStack<S, M, R, View = V>,
{
    type Context = MessageContext<M>;

    type Output = ();

    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, M, R>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::Output {
        let new_widget: &gtk::Widget = node.state().as_view_state().unwrap().as_ref();
        loop {
            match self.current_child.take() {
                Some(child) if new_widget == &child => {
                    self.current_child = child.next_sibling();
                    self.index += 1;
                    break;
                }
                Some(child) if new_widget.parent().is_some() => {
                    self.current_child = child.next_sibling();
                    self.container.remove(&child);
                    self.index += 1;
                }
                Some(child) => {
                    self.container
                        .insert(new_widget, self.index);
                    self.current_child = Some(child);
                    self.index += 1;
                    break;
                }
                None => {
                    self.container.append(new_widget);
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

