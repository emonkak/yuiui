use std::any::TypeId;
use std::fmt;
use std::mem;

use crate::component::ComponentStack;
use crate::effect::EffectContext;
use crate::event::{CaptureState, EventMask, EventResult, InternalEvent};
use crate::id::Id;
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
    ) -> EventResult<S> {
        EventResult::Nop
    }

    fn event(
        &mut self,
        _event: &Self::Event,
        _children: &Self::Children,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::Nop
    }
}

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
                context.process(widget.lifecycle(
                    WidgetLifeCycle::Mounted,
                    &self.children,
                    state,
                    env,
                ));
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Prepared(mut widget, view) => {
                match mode {
                    CommitMode::Mount => {
                        context.process(widget.lifecycle(
                            WidgetLifeCycle::Mounted,
                            &self.children,
                            state,
                            env,
                        ));
                    }
                    CommitMode::Unmount => {
                        context.mark_unmounted();
                        context.process(widget.lifecycle(
                            WidgetLifeCycle::Unmounted,
                            &self.children,
                            state,
                            env,
                        ));
                    }
                    CommitMode::Update => {}
                }
                WidgetState::Prepared(widget, view)
            }
            WidgetState::Changed(mut widget, view, old_view) => {
                if view.rebuild(&self.children, &old_view, &mut widget, state, env) {
                    context.process(widget.lifecycle(
                        WidgetLifeCycle::Updated,
                        &self.children,
                        state,
                        env,
                    ));
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
                    self.children.event(event, state, env, context);
                    capture_state = CaptureState::Captured;
                }
                if TypeId::of::<Event>() == TypeId::of::<<V::Widget as Widget<S, E>>::Event>() {
                    let event = unsafe { mem::transmute(event) };
                    context.process(widget.event(event, &self.children, state, env));
                    capture_state = CaptureState::Captured;
                }
                context.end_widget();
            }
            _ => {}
        }
        capture_state
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
                if self.id == event.id_path.bottom_id() {
                    let event = event
                        .payload
                        .downcast_ref()
                        .expect("cast internal event to widget event");
                    context.process(widget.event(event, &self.children, state, env));
                } else {
                    self.children.internal_event(event, state, env, context);
                }
                context.end_widget();
                CaptureState::Captured
            }
            WidgetState::Uninitialized(_) => CaptureState::Ignored,
        }
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

#[derive(Debug)]
pub enum WidgetLifeCycle {
    Mounted,
    Updated,
    Unmounted,
}
