mod commit_visitor;
mod event_visitor;
mod internal_event_visitor;
mod update_subtree_visitor;

use std::fmt;

use crate::component_node::ComponentStack;
use crate::context::{EffectContext, IdContext, RenderContext};
use crate::event::{Event, EventMask, InternalEvent};
use crate::id::{ComponentIndex, Id, IdPath};
use crate::sequence::{TraversableSeq, TraversableSeqVisitor};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};

use commit_visitor::CommitVisitor;
use event_visitor::EventVisitor;
use internal_event_visitor::InternalEventVisitor;
use update_subtree_visitor::UpdateSubtreeVisitor;

pub trait WidgetNodeSeq<S: State, E>:
    TraversableSeq<CommitVisitor, EffectContext<S>, S, E>
    + TraversableSeq<UpdateSubtreeVisitor, RenderContext, S, E>
    + for<'a> TraversableSeq<EventVisitor<'a>, EffectContext<S>, S, E>
    + for<'a> TraversableSeq<InternalEventVisitor<'a>, EffectContext<S>, S, E>
{
    fn event_mask() -> EventMask;

    fn len(&self) -> usize;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);
}

pub struct WidgetNode<V: View<S, E>, CS: ComponentStack<S, E>, S: State, E> {
    pub(crate) id: Id,
    pub(crate) state: Option<WidgetState<V, V::Widget>>,
    pub(crate) children: <V::Widget as Widget<S, E>>::Children,
    pub(crate) components: CS,
    pub(crate) event_mask: EventMask,
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
        children: <V::Widget as Widget<S, E>>::Children,
        components: CS,
    ) -> Self {
        Self {
            id,
            state: Some(WidgetState::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Widget as Widget<S, E>>::Children::event_mask(),
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

    pub fn children(&self) -> &<V::Widget as Widget<S, E>>::Children {
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
        component_index: Option<ComponentIndex>,
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
        mode: CommitMode,
        id_path: &IdPath,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let mut visitor = CommitVisitor::new(mode);
        self.search(id_path, &mut visitor, state, env, context);
    }

    pub fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = EventVisitor::new(event);
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
        visitor.result()
    }

    pub fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = InternalEventVisitor::new(event.payload());
        self.search(event.id_path(), &mut visitor, state, env, context)
    }
}

impl<V, CS, S, E> WidgetNodeSeq<S, E> for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn event_mask() -> EventMask {
        let mut event_mask = <V::Widget as Widget<S, E>>::Children::event_mask();
        event_mask.extend(<V::Widget as WidgetEvent>::Event::allowed_types());
        event_mask
    }

    fn len(&self) -> usize {
        1
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        if self.dirty || mode.is_propagatable() {
            let mut visitor = CommitVisitor::new(mode);
            context.begin_widget(self.id);
            visitor.visit(self, state, env, context);
            context.end_widget();
            self.dirty = false;
        }
    }
}

impl<V, CS, Visitor, Context, S, E> TraversableSeq<Visitor, Context, S, E>
    for WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    <V::Widget as Widget<S, E>>::Children: TraversableSeq<Visitor, Context, S, E>,
    CS: ComponentStack<S, E, View = V>,
    Visitor: TraversableSeqVisitor<Self, Context, S, E>,
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
    <V::Widget as Widget<S, E>>::Children: fmt::Debug,
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
            .finish()
    }
}

pub struct WidgetNodeScope<'a, V: View<S, E>, CS, S: State, E> {
    pub id: Id,
    pub state: &'a mut Option<WidgetState<V, V::Widget>>,
    pub children: &'a mut <V::Widget as Widget<S, E>>::Children,
    pub components: &'a mut CS,
    pub dirty: &'a mut bool,
}

#[derive(Debug)]
pub enum WidgetState<V, W> {
    Uninitialized(V),
    Prepared(W, V),
    Dirty(W, V),
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
