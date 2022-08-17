use std::any::TypeId;
use std::fmt;
use std::mem;

use crate::component::ComponentStack;
use crate::context::{EffectContext, Id};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::sequence::{CommitMode, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;

pub trait Widget<S: State, E> {
    type Children: WidgetNodeSeq<S, E>;

    type Event: 'static;

    fn lifecycle(
        &mut self,
        _lifecycle: WidgetLifeCycle,
        _children: &Self::Children,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) {
    }

    fn event(
        &mut self,
        _event: &Self::Event,
        _children: &Self::Children,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) -> EventResult {
        EventResult::Ignored
    }
}

pub struct WidgetNode<V: View<S, E>, CS, S: State, E> {
    pub id: Id,
    pub state: Option<WidgetState<V, V::Widget>>,
    pub children: <V::Widget as Widget<S, E>>::Children,
    pub components: CS,
    pub event_mask: EventMask,
}

impl<V, CS, S, E> WidgetNode<V, CS, S, E>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    pub fn new(
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

    pub fn scope(&mut self) -> WidgetNodeScope<V, CS, S, E> {
        WidgetNodeScope {
            id: self.id,
            state: &mut self.state,
            children: &mut self.children,
            components: &mut self.components,
        }
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
                widget.lifecycle(
                    WidgetLifeCycle::Mounted,
                    &self.children,
                    state,
                    env,
                    context,
                );
                WidgetState::Prepared(widget)
            }
            WidgetState::Prepared(mut widget) => {
                match mode {
                    CommitMode::Mount => {
                        widget.lifecycle(
                            WidgetLifeCycle::Mounted,
                            &self.children,
                            state,
                            env,
                            context,
                        );
                    }
                    CommitMode::Unmount => {
                        widget.lifecycle(
                            WidgetLifeCycle::Unmounted,
                            &self.children,
                            state,
                            env,
                            context,
                        );
                    }
                    CommitMode::Update => {}
                }
                WidgetState::Prepared(widget)
            }
            WidgetState::Changed(mut widget, view) => {
                if view.rebuild(&self.children, &mut widget, state, env) {
                    widget.lifecycle(
                        WidgetLifeCycle::Updated,
                        &self.children,
                        state,
                        env,
                        context,
                    );
                }
                WidgetState::Prepared(widget)
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
    ) -> EventResult {
        let mut result = EventResult::Ignored;
        match self.state.as_mut().unwrap() {
            WidgetState::Prepared(widget) | WidgetState::Changed(widget, _) => {
                context.begin_widget(self.id);
                if self.event_mask.contains(&TypeId::of::<Event>()) {
                    result = result.merge(self.children.event(event, state, env, context));
                }
                if TypeId::of::<Event>() == TypeId::of::<<V::Widget as Widget<S, E>>::Event>() {
                    let event = unsafe { mem::transmute(event) };
                    result = result.merge(widget.event(event, &self.children, state, env, context));
                }
                context.end_widget();
            }
            _ => {}
        }
        result
    }

    pub fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        match self.state.as_mut().unwrap() {
            WidgetState::Prepared(widget) | WidgetState::Changed(widget, _) => {
                context.begin_widget(self.id);
                let result = if self.id == event.id_path.id() {
                    let event = event
                        .payload
                        .downcast_ref()
                        .expect("cast internal event to widget event");
                    widget.event(event, &self.children, state, env, context)
                } else {
                    self.children.internal_event(event, state, env, context)
                };
                context.end_widget();
                result
            }
            WidgetState::Uninitialized(_) => EventResult::Ignored,
        }
    }
}

impl<V, CS, S, E> fmt::Debug for WidgetNode<V, CS, S, E>
where
    V: View<S, E> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S, E>>::Children: fmt::Debug,
    CS: fmt::Debug,
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
    Prepared(W),
    Changed(W, V),
}

#[derive(Debug)]
pub enum WidgetLifeCycle {
    Mounted,
    Updated,
    Unmounted,
}
