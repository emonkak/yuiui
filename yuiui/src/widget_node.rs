use std::any::{Any, TypeId};
use std::fmt;

use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::event::{CaptureState, Event, EventMask, InternalEvent};
use crate::id::{Id, IdPath};
use crate::sequence::{NodeVisitor, TraversableSeq, WidgetNodeSeq};
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
    CS: ComponentStack<S, E>,
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

    pub fn event_mask(&self) -> &EventMask {
        &self.event_mask
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        context.begin_widget(self.id);
        context.begin_components();
        self.components.commit(mode, state, env, context);
        context.end_components();
        self.children.commit(mode, state, env, context);
        self.state = match self.state.take().unwrap() {
            WidgetState::Uninitialized(view) => {
                let mut widget = view.build(&self.children, state, env);
                let result = widget.lifecycle(
                    WidgetLifeCycle::Mounted,
                    &self.children,
                    context.id_path(),
                    state,
                    env,
                );
                context.process_result(result);
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Prepared(mut widget, view) => {
                match mode {
                    CommitMode::Mount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Mounted,
                            &self.children,
                            context.id_path(),
                            state,
                            env,
                        );
                        context.process_result(result);
                    }
                    CommitMode::Unmount => {
                        let result = widget.lifecycle(
                            WidgetLifeCycle::Unmounted,
                            &self.children,
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
            WidgetState::Changed(mut widget, view, old_view) => {
                if view.rebuild(&self.children, &old_view, &mut widget, state, env) {
                    let result = widget.lifecycle(
                        WidgetLifeCycle::Updated,
                        &self.children,
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
        context.end_widget();
    }

    pub fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        let mut capture_state = CaptureState::Ignored;
        match self.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Changed(widget, _, _) => {
                context.begin_widget(self.id);
                if self.event_mask.contains(&TypeId::of::<Event>()) {
                    capture_state = self.children.event(event, state, env, context);
                }
                if let Some(event) = <V::Widget as WidgetEvent>::Event::from_static(event) {
                    let result = widget.event(event, &self.children, context.id_path(), state, env);
                    context.process_result(result);
                    capture_state = CaptureState::Captured;
                }
                context.end_widget();
            }
            _ => {}
        }
        capture_state
    }

    pub fn any_event(
        &mut self,
        event: &dyn Any,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        match self.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Changed(widget, _, _) => {
                context.begin_widget(self.id);
                let event = <V::Widget as WidgetEvent>::Event::from_any(event)
                    .expect("cast internal event to widget event");
                context.process_result(widget.event(
                    event,
                    &self.children,
                    context.id_path(),
                    state,
                    env,
                ));
                context.end_widget();
            }
            WidgetState::Uninitialized(_) => {}
        }
    }

    pub fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState {
        match self.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Changed(widget, _, _) => {
                context.begin_widget(self.id);
                let capture_state = if self.id == event.id_path().bottom_id() {
                    let event = <V::Widget as WidgetEvent>::Event::from_any(event.payload())
                        .expect("cast internal event to widget event");
                    context.process_result(widget.event(
                        event,
                        &self.children,
                        context.id_path(),
                        state,
                        env,
                    ));
                    CaptureState::Captured
                } else if event.id_path().starts_with(context.id_path()) {
                    self.children.internal_event(event, state, env, context)
                } else {
                    CaptureState::Ignored
                };
                context.end_widget();
                capture_state
            }
            WidgetState::Uninitialized(_) => CaptureState::Ignored,
        }
    }

    pub fn for_each<Visitor>(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        <V::Widget as Widget<S, E>>::Children: TraversableSeq<Visitor, S, E>,
        Visitor: NodeVisitor<Self, S, E>,
    {
        context.begin_widget(self.id);
        self.children.for_each(visitor, state, env, context);
        visitor.visit(self, state, env, context);
        context.end_widget();
    }

    pub fn search<Visitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool
    where
        <V::Widget as Widget<S, E>>::Children: TraversableSeq<Visitor, S, E>,
        Visitor: NodeVisitor<Self, S, E>,
    {
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
    Changed(W, V, V),
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
