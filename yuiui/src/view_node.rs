mod commit_visitor;
mod downward_event_visitor;
mod local_event_visitor;
mod update_visitor;
mod upward_event_visitor;

use std::any::Any;
use std::fmt;
use std::sync::Once;

use crate::component_node::ComponentStack;
use crate::context::{EffectContext, IdContext, RenderContext};
use crate::element::ElementSeq;
use crate::event::{Event, EventMask, HasEvent};
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;
use crate::traversable::{Traversable, TraversableVisitor};
use crate::view::View;

use commit_visitor::CommitVisitor;
use downward_event_visitor::DownwardEventVisitor;
use local_event_visitor::LocalEventVisitor;
use update_visitor::UpdateVisitor;
use upward_event_visitor::UpwardEventVisitor;

pub trait ViewNodeSeq<S: State, E>:
    Traversable<CommitVisitor, EffectContext<S>, S, E>
    + Traversable<UpdateVisitor, RenderContext, S, E>
    + for<'a> Traversable<DownwardEventVisitor<'a>, EffectContext<S>, S, E>
    + for<'a> Traversable<LocalEventVisitor<'a>, EffectContext<S>, S, E>
    + for<'a> Traversable<UpwardEventVisitor<'a>, EffectContext<S>, S, E>
{
    fn event_mask() -> &'static EventMask;

    fn len(&self) -> usize;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

pub struct ViewNode<V: View<S, E>, CS: ComponentStack<S, E, View = V>, S: State, E> {
    pub(crate) id: Id,
    pub(crate) state: Option<ViewNodeState<V, V::Widget>>,
    pub(crate) children: <V::Children as ElementSeq<S, E>>::Store,
    pub(crate) components: CS,
    pub(crate) event_mask: &'static EventMask,
    pub(crate) dirty: bool,
}

impl<V, CS, S, E> ViewNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    pub(crate) fn new(
        id: Id,
        view: V,
        children: <V::Children as ElementSeq<S, E>>::Store,
        components: CS,
    ) -> Self {
        Self {
            id,
            state: Some(ViewNodeState::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Children as ElementSeq<S, E>>::Store::event_mask(),
            dirty: true,
        }
    }

    pub(crate) fn scope(&mut self) -> ViewNodeScope<V, CS, S, E> {
        ViewNodeScope {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
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

    pub fn children(&self) -> &<V::Children as ElementSeq<S, E>>::Store {
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
        id_path: &IdPath,
        component_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut visitor = UpdateVisitor::new(component_index);
        self.search(id_path, &mut visitor, state, env, context);
        visitor.result()
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            let mut visitor = CommitVisitor::new(mode, 0);
            visitor.visit(self, state, env, context);
            self.dirty = false;
        }
    }

    pub fn commit_subtree(
        &mut self,
        id_path: &IdPath,
        component_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let mut visitor = CommitVisitor::new(CommitMode::Update, component_index);
        self.search(id_path, &mut visitor, state, env, context);
    }

    pub fn downward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = DownwardEventVisitor::new(event);
        self.search(id_path, &mut visitor, state, env, context);
        visitor.result()
    }

    pub fn upward_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = UpwardEventVisitor::new(event, id_path);
        visitor.visit(self, state, env, context);
        visitor.result()
    }

    pub fn local_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = LocalEventVisitor::new(event);
        self.search(id_path, &mut visitor, state, env, context)
    }

    pub fn search<Visitor, Context>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool
    where
        <V::Children as ElementSeq<S, E>>::Store: Traversable<Visitor, Context, S, E>,
        Visitor: TraversableVisitor<Self, Context, S, E>,
        Context: IdContext,
    {
        if self.id == id_path.bottom_id() {
            visitor.visit(self, state, env, context);
            true
        } else if id_path.starts_with(context.id_path()) {
            self.children.search(id_path, visitor, state, env, context)
        } else {
            false
        }
    }
}

impl<V, CS, S, E> ViewNodeSeq<S, E> for ViewNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        static INIT: Once = Once::new();
        static mut EVENT_MASK: EventMask = EventMask::new();

        if !INIT.is_completed() {
            let children_mask = <V::Children as ElementSeq<S, E>>::Store::event_mask();

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

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        context.begin_view(self.id);
        self.commit(mode, state, env, context);
        context.end_view();
    }
}

impl<V, CS, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for ViewNode<V, CS, S, E>
where
    V: View<S, E>,
    <V::Children as ElementSeq<S, E>>::Store: Traversable<Visitor, Context, S, E>,
    CS: ComponentStack<S, E, View = V>,
    Visitor: TraversableVisitor<Self, Context, S, E>,
    Context: IdContext,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        context.begin_view(self.id);
        visitor.visit(self, state, env, context);
        context.end_view();
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        context.begin_view(self.id);
        let result = self.search(id_path, visitor, state, env, context);
        context.end_view();
        result
    }
}

impl<V, CS, S, E> fmt::Debug for ViewNode<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq<S, E>>::Store: fmt::Debug,
    CS: ComponentStack<S, E, View = V> + fmt::Debug,
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

pub struct ViewNodeScope<'a, V: View<S, E>, CS, S: State, E> {
    pub id: Id,
    pub state: &'a mut Option<ViewNodeState<V, V::Widget>>,
    pub children: &'a mut <V::Children as ElementSeq<S, E>>::Store,
    pub components: &'a mut CS,
    pub dirty: &'a mut bool,
}

#[derive(Debug)]
pub enum ViewNodeState<V, W> {
    Uninitialized(V),
    Prepared(V, W),
    Pending(V, V, W),
}

impl<V, W> ViewNodeState<V, W> {
    pub fn map_view<F, NV>(self, f: F) -> ViewNodeState<NV, W>
    where
        F: Fn(V) -> NV,
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
