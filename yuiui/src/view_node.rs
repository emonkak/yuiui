mod batch_visitor;
mod commit_visitor;
mod downward_event_visitor;
mod local_event_visitor;
mod update_visitor;
mod upward_event_visitor;

use std::any::Any;
use std::fmt;
use std::rc::Rc;
use std::sync::Once;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, IdContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::{Event, EventMask, HasEvent};
use crate::id::{ComponentIndex, Id, IdPath, IdPathBuf, IdTree};
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;

use batch_visitor::BatchVisitor;
use commit_visitor::CommitVisitor;
use downward_event_visitor::DownwardEventVisitor;
use local_event_visitor::LocalEventVisitor;
use update_visitor::UpdateVisitor;
use upward_event_visitor::UpwardEventVisitor;

pub struct ViewNode<V: View<S, B>, CS: ComponentStack<S, B, View = V>, S: State, B> {
    pub(crate) id: Id,
    pub(crate) state: Option<ViewNodeState<V, V::Widget>>,
    pub(crate) children: <V::Children as ElementSeq<S, B>>::Storage,
    pub(crate) components: CS,
    pub(crate) env: Option<Rc<dyn Any>>,
    pub(crate) event_mask: &'static EventMask,
    pub(crate) dirty: bool,
}

impl<V, CS, S, B> ViewNode<V, CS, S, B>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    pub(crate) fn new(
        id: Id,
        view: V,
        children: <V::Children as ElementSeq<S, B>>::Storage,
        components: CS,
    ) -> Self {
        Self {
            id,
            state: Some(ViewNodeState::Uninitialized(view)),
            children,
            components,
            env: None,
            event_mask: <V::Children as ElementSeq<S, B>>::Storage::event_mask(),
            dirty: true,
        }
    }

    pub(crate) fn scope(&mut self) -> ViewNodeScope<V, CS, S, B> {
        ViewNodeScope {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
            env: &mut self.env,
            dirty: &mut self.dirty,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn state(&self) -> &ViewNodeState<V, V::Widget> {
        self.state.as_ref().unwrap()
    }

    pub fn as_widget(&self) -> Option<&V::Widget> {
        match self.state.as_ref().unwrap() {
            ViewNodeState::Prepared(_, widget) | ViewNodeState::Pending(_, _, widget) => {
                Some(widget)
            }
            ViewNodeState::Uninitialized(_) => None,
        }
    }

    pub fn children(&self) -> &<V::Children as ElementSeq<S, B>>::Storage {
        &self.children
    }

    pub fn components(&self) -> &CS {
        &self.components
    }

    pub fn event_mask(&self) -> &EventMask {
        &self.event_mask
    }

    pub fn update_subtree(
        &mut self,
        id_tree: &IdTree<ComponentIndex>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> Vec<(IdPathBuf, ComponentIndex)> {
        let mut visitor = BatchVisitor::new(id_tree.root(), |_, component_index| {
            UpdateVisitor::new(component_index)
        });
        visitor.visit(self, state, backend, context);
        visitor.into_changed_nodes()
    }

    pub fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        if self.dirty || mode.is_propagatable() {
            let mut visitor = CommitVisitor::new(mode, 0);
            visitor.visit(self, state, backend, context);
            self.dirty = false;
            true
        } else {
            false
        }
    }

    pub fn commit_subtree(
        &mut self,
        id_tree: &IdTree<ComponentIndex>,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut visitor = BatchVisitor::new(id_tree.root(), |_, component_index| {
            CommitVisitor::new(CommitMode::Update, component_index)
        });
        visitor.visit(self, state, backend, context)
    }

    pub fn downward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event);
        self.search(id_path, &mut visitor, state, backend, context)
    }

    pub fn upward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut visitor = UpwardEventVisitor::new(event, id_path);
        visitor.visit(self, state, backend, context)
    }

    pub fn local_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let mut visitor = LocalEventVisitor::new(event);
        self.search(id_path, &mut visitor, state, backend, context)
    }

    pub fn search<Visitor, Context>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool
    where
        <V::Children as ElementSeq<S, B>>::Storage: Traversable<Visitor, Context, S, B>,
        Visitor: TraversableVisitor<Self, Context, S, B>,
        Context: IdContext,
    {
        if self.id == Id::from_top(id_path) {
            visitor.visit(self, state, backend, context)
        } else if self.id == Id::from_bottom(id_path) {
            debug_assert!(id_path.len() > 0);
            let id_path = &id_path[1..];
            self.children
                .search(id_path, visitor, state, backend, context)
        } else {
            false
        }
    }
}

pub struct ViewNodeScope<'a, V: View<S, B>, CS, S: State, B> {
    pub(crate) id: Id,
    pub(crate) state: &'a mut Option<ViewNodeState<V, V::Widget>>,
    pub(crate) children: &'a mut <V::Children as ElementSeq<S, B>>::Storage,
    pub(crate) components: &'a mut CS,
    pub(crate) env: &'a mut Option<Rc<dyn Any>>,
    pub(crate) dirty: &'a mut bool,
}

pub trait ViewNodeSeq<S: State, B>:
    Traversable<CommitVisitor, CommitContext<S>, S, B>
    + Traversable<UpdateVisitor, RenderContext, S, B>
    + for<'a> Traversable<BatchVisitor<'a, CommitVisitor>, CommitContext<S>, S, B>
    + for<'a> Traversable<BatchVisitor<'a, UpdateVisitor>, RenderContext, S, B>
    + for<'a> Traversable<DownwardEventVisitor<'a>, CommitContext<S>, S, B>
    + for<'a> Traversable<LocalEventVisitor<'a>, CommitContext<S>, S, B>
    + for<'a> Traversable<UpwardEventVisitor<'a>, CommitContext<S>, S, B>
{
    fn event_mask() -> &'static EventMask;

    fn len(&self) -> usize;

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool;
}

impl<V, CS, S, B> ViewNodeSeq<S, B> for ViewNode<V, CS, S, B>
where
    V: View<S, B>,
    CS: ComponentStack<S, B, View = V>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let children_mask = <V::Children as ElementSeq<S, B>>::Storage::event_mask();

            INIT.call_once(|| unsafe {
                EVENT_MASK.merge(children_mask);
                let mut types = Vec::new();
                <V as HasEvent>::Event::collect_types(&mut types);
                EVENT_MASK.add_all(&types);
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        1
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        context.begin_view(self.id, &self.env);
        let result = self.commit(mode, state, backend, context);
        context.end_view();
        result
    }
}

impl<V, CS, Visitor, Context, S, B> Traversable<Visitor, Context, S, B> for ViewNode<V, CS, S, B>
where
    V: View<S, B>,
    <V::Children as ElementSeq<S, B>>::Storage: Traversable<Visitor, Context, S, B>,
    CS: ComponentStack<S, B, View = V>,
    Visitor: TraversableVisitor<Self, Context, S, B>,
    Context: IdContext,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        context.begin_view(self.id, &self.env);
        let result = visitor.visit(self, state, backend, context);
        context.end_view();
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut Context,
    ) -> bool {
        context.begin_view(self.id, &self.env);
        let result = self.search(id_path, visitor, state, backend, context);
        context.end_view();
        result
    }
}

impl<V, CS, S, B> fmt::Debug for ViewNode<V, CS, S, B>
where
    V: View<S, B> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq<S, B>>::Storage: fmt::Debug,
    CS: ComponentStack<S, B, View = V> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ViewNode")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("event_mask", &self.event_mask)
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[derive(Debug)]
pub enum ViewNodeState<V, W> {
    Uninitialized(V),
    Prepared(V, W),
    Pending(V, V, W),
}

impl<V, W> ViewNodeState<V, W> {
    pub fn map_view<F, NewView>(self, f: F) -> ViewNodeState<NewView, W>
    where
        F: Fn(V) -> NewView,
    {
        match self {
            Self::Uninitialized(view) => ViewNodeState::Uninitialized(f(view)),
            Self::Prepared(view, widget) => ViewNodeState::Prepared(f(view), widget),
            Self::Pending(view, pending_view, widget) => {
                ViewNodeState::Pending(f(view), f(pending_view), widget)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    pub fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount | Self::Unmount => true,
            Self::Update => false,
        }
    }
}
