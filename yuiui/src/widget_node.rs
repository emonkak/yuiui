use std::any::{Any, TypeId};
use std::fmt;

use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::event::{Event, EventMask, InternalEvent};
use crate::id::{ComponentIndex, Id, IdContext, IdPath};
use crate::sequence::WidgetNodeSeq;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent, WidgetLifeCycle};

pub struct WidgetNode<V: View<S, E>, CS: ComponentStack<S, E>, S: State, E> {
    pub(crate) id: Id,
    pub(crate) state: Option<WidgetState<V, V::Widget>>,
    pub(crate) children: <V::Widget as Widget<S, E>>::Children,
    pub(crate) components: CS,
    pub(crate) event_mask: EventMask,
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
        }
    }

    pub(crate) fn scope(&mut self) -> WidgetNodeScope<V, CS, S, E> {
        WidgetNodeScope {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
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

    pub fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let mut visitor = CommitVisitor::new(mode);
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
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

    pub fn force_update(
        &mut self,
        component_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) {
        let scope = self.scope();
        CS::force_update(scope, component_index, 0, state, env, context);
    }

    pub fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let mut visitor = StaticEventVisitor::new(event);
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
        visitor.captured
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

    pub fn for_each<Visitor: WidgetNodeVisitor>(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        context.begin_widget(self.id);
        visitor.visit(self, state, env, context);
        context.end_widget();
    }

    pub fn search<Visitor: WidgetNodeVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
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
    CS: ComponentStack<S, E> + fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

pub trait WidgetNodeVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State;
}

struct CommitVisitor {
    mode: CommitMode,
}

impl CommitVisitor {
    fn new(mode: CommitMode) -> Self {
        Self { mode }
    }
}

impl WidgetNodeVisitor for CommitVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State,
    {
        context.begin_components();
        node.components.commit(self.mode, state, env, context);
        context.end_components();
        node.children.commit(self.mode, state, env, context);
        node.state = match node.state.take().unwrap() {
            WidgetState::Uninitialized(view) => {
                let mut widget = view.build(&node.children, state, env);
                let result = widget.lifecycle(
                    WidgetLifeCycle::Mounted,
                    &node.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Prepared(mut widget, view) => {
                match self.mode {
                    CommitMode::Mount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Mounted,
                            &node.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Unmount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Unmounted,
                            &node.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Update => {}
                }
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Dirty(mut widget, view) => {
                if view.rebuild(&node.children, &mut widget, state, env) {
                    let result = widget.lifecycle(
                        WidgetLifeCycle::Updated,
                        &node.children,
                        context.id_path(),
                        state,
                        env,
                    );
                    context.process_result(result);
                }
                WidgetState::Prepared(widget, view)
            }
        }
        .into();
    }
}

struct StaticEventVisitor<'a, Event> {
    event: &'a Event,
    captured: bool,
}

impl<'a, Event: 'static> StaticEventVisitor<'a, Event> {
    fn new(event: &'a Event) -> Self {
        Self {
            event,
            captured: false,
        }
    }
}

impl<'a, Event: 'static> WidgetNodeVisitor for StaticEventVisitor<'a, Event> {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State,
    {
        match node.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Dirty(widget, _) => {
                if node.event_mask.contains(&TypeId::of::<Event>()) {
                    node.children.for_each(self, state, env, context);
                }
                if let Some(event) = <V::Widget as WidgetEvent>::Event::from_static(self.event) {
                    let result = widget.event(event, &node.children, context.id_path(), state, env);
                    context.process_result(result);
                    self.captured = true;
                }
            }
            _ => {}
        }
    }
}

struct InternalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> InternalEventVisitor<'a> {
    fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a> WidgetNodeVisitor for InternalEventVisitor<'a> {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E>,
        S: State,
    {
        match node.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Dirty(widget, _) => {
                let event = <V::Widget as WidgetEvent>::Event::from_any(self.event)
                    .expect("cast any event to widget event");
                let result = widget.event(event, &node.children, context.id_path(), state, env);
                context.process_result(result);
            }
            WidgetState::Uninitialized(_) => {}
        }
    }
}
