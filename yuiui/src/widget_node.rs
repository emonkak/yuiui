mod commit_visitor;
mod downward_event_visitor;
mod internal_event_visitor;
mod update_subtree_visitor;
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
use internal_event_visitor::InternalEventVisitor;
use update_subtree_visitor::UpdateSubtreeVisitor;
use upward_event_visitor::UpwardEventVisitor;

pub trait WidgetNodeSeq<S: State, E>:
    Traversable<CommitVisitor, EffectContext<S>, S, E>
    + Traversable<UpdateSubtreeVisitor, RenderContext, S, E>
    + for<'a> Traversable<DownwardEventVisitor<'a>, EffectContext<S>, S, E>
    + for<'a> Traversable<InternalEventVisitor<'a>, EffectContext<S>, S, E>
    + for<'a> Traversable<UpwardEventVisitor<'a>, EffectContext<S>, S, E>
{
    fn event_mask() -> &'static EventMask;

    fn len(&self) -> usize;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

pub struct WidgetNode<V: View<S, E>, CS: ComponentStack<S, E, View = V>, S: State, E> {
    pub(crate) id: Id,
    pub(crate) state: Option<WidgetState<V, V::Widget>>,
    pub(crate) children: <V::Children as ElementSeq<S, E>>::Store,
    pub(crate) components: CS,
    pub(crate) event_mask: &'static EventMask,
    pub(crate) dirty: bool,
}

impl<V, CS, S, E> WidgetNode<V, CS, S, E>
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
            state: Some(WidgetState::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Children as ElementSeq<S, E>>::Store::event_mask(),
            dirty: true,
        }
    }

    pub(crate) fn scope(&mut self) -> WidgetNodeScope<V, CS, S, E> {
        WidgetNodeScope {
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

    pub fn state(&self) -> &WidgetState<V, <V as View<S, E>>::Widget> {
        self.state.as_ref().unwrap()
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
        let mut visitor = UpdateSubtreeVisitor::new(component_index);
        self.search(id_path, &mut visitor, state, env, context);
        visitor.result()
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
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
        visitor.result()
    }

    pub fn internal_event(
        &mut self,
        event: &dyn Any,
        id_path: &IdPath,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = InternalEventVisitor::new(event);
        self.search(id_path, &mut visitor, state, env, context)
    }
}

impl<V, CS, S, E> WidgetNodeSeq<S, E> for WidgetNode<V, CS, S, E>
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
                EVENT_MASK.add_all(&<V as HasEvent>::Event::allowed_types());
            });
        }

        unsafe { &EVENT_MASK }
    }

    fn len(&self) -> usize {
        1
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            let mut visitor = CommitVisitor::new(mode, 0);
            context.begin_widget(self.id);
            visitor.visit(self, state, env, context);
            context.end_widget();
            self.dirty = false;
        }
    }
}

impl<V, CS, Visitor, Context, S, E> Traversable<Visitor, Context, S, E> for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    <V::Children as ElementSeq<S, E>>::Store: Traversable<Visitor, Context, S, E>,
    CS: ComponentStack<S, E, View = V>,
    Visitor: TraversableVisitor<Self, Context, S, E>,
    Context: IdContext,
    S: State,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut Context) {
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut Context,
    ) -> bool {
        context.begin_widget(self.id);
        let result = if self.id == id_path.bottom_id() {
            visitor.visit(self, state, env, context);
            true
        } else if id_path.starts_with(context.id_path()) {
            self.children.search(id_path, visitor, state, env, context)
        } else {
            false
        };
        context.end_widget();
        result
    }
}

impl<V, CS, S, E> fmt::Debug for WidgetNode<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq<S, E>>::Store: fmt::Debug,
    CS: ComponentStack<S, E, View = V> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WidgetNode")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("event_mask", &self.event_mask)
            .field("dirty", &self.dirty)
            .finish()
    }
}

pub struct WidgetNodeScope<'a, V: View<S, E>, CS, S: State, E> {
    pub id: Id,
    pub state: &'a mut Option<WidgetState<V, V::Widget>>,
    pub children: &'a mut <V::Children as ElementSeq<S, E>>::Store,
    pub components: &'a mut CS,
    pub dirty: &'a mut bool,
}

#[derive(Debug)]
pub enum WidgetState<V, W> {
    Uninitialized(V),
    Prepared(W, V),
    Dirty(W, V),
    Pending(W, V, V),
}

impl<V, W> WidgetState<V, W> {
    pub fn map_view<F, NV>(self, f: F) -> WidgetState<NV, W>
    where
        F: Fn(V) -> NV,
    {
        match self {
            Self::Uninitialized(view) => WidgetState::Uninitialized(f(view)),
            Self::Prepared(widget, view) => WidgetState::Prepared(widget, f(view)),
            Self::Dirty(widget, view) => WidgetState::Dirty(widget, f(view)),
            Self::Pending(widget, view, pending_view) => {
                WidgetState::Pending(widget, f(view), f(pending_view))
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
