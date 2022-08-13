use std::any::TypeId;
use std::fmt;
use std::mem;

use crate::component::ComponentStack;
use crate::context::{EffectContext, Id};
use crate::event::{EventMask, InternalEvent};
use crate::sequence::{CommitMode, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;

pub trait Widget<S: State> {
    type Children: WidgetNodeSeq<S>;

    type Event: 'static;

    fn event(&self, _event: &Self::Event, _state: &S, _context: &mut EffectContext<S>) {}
}

pub struct WidgetNode<V: View<S>, CS, S: State> {
    pub id: Id,
    pub status: Option<WidgetStatus<V, V::Widget>>,
    pub children: <V::Widget as Widget<S>>::Children,
    pub components: CS,
    pub event_mask: EventMask,
}

impl<V, CS, S> WidgetNode<V, CS, S>
where
    V: View<S>,
    CS: ComponentStack<S>,
    S: State,
{
    pub fn new(
        id: Id,
        view: V,
        children: <V::Widget as Widget<S>>::Children,
        components: CS,
    ) -> Self {
        Self {
            id,
            status: Some(WidgetStatus::Uninitialized(view)),
            children,
            components,
            event_mask: <V::Widget as Widget<S>>::Children::event_mask(),
        }
    }

    pub fn map_components<F, NCS>(self, f: F) -> WidgetNode<V, NCS, S>
    where
        F: FnOnce(CS) -> NCS,
    {
        WidgetNode {
            id: self.id,
            status: self.status,
            children: self.children,
            components: f(self.components),
            event_mask: self.event_mask,
        }
    }

    pub fn scope(&mut self) -> WidgetNodeScope<V, CS, S> {
        WidgetNodeScope {
            id: self.id,
            status: &mut self.status,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>) {
        context.begin_widget(self.id);
        context.begin_components();
        self.components.commit(mode, state, context);
        context.end_components();
        self.children.commit(mode, state, context);
        self.status = match self.status.take().unwrap() {
            WidgetStatus::Uninitialized(view) => {
                let widget = view.build(&self.children, state);
                WidgetStatus::Prepared(widget)
            }
            WidgetStatus::Prepared(widget) => WidgetStatus::Prepared(widget),
            WidgetStatus::Changed(mut widget, view) => {
                view.rebuild(&self.children, &mut widget, state);
                WidgetStatus::Prepared(widget)
            }
        }
        .into();
        context.end_widget();
    }

    pub fn event<E: 'static>(&self, event: &E, state: &S, context: &mut EffectContext<S>) {
        if let WidgetStatus::Prepared(widget) = self.status.as_ref().unwrap() {
            context.begin_widget(self.id);
            if TypeId::of::<E>() == TypeId::of::<<V::Widget as Widget<S>>::Event>() {
                let event = unsafe { mem::transmute(event) };
                widget.event(event, state, context);
            }
            if self.event_mask.contains(&TypeId::of::<E>()) {
                self.children.event(event, state, context);
            }
            context.end_widget();
        }
    }

    pub fn internal_event(&self, event: &InternalEvent, state: &S, context: &mut EffectContext<S>) {
        if let WidgetStatus::Prepared(widget) = self.status.as_ref().unwrap() {
            context.begin_widget(self.id);
            if self.id == event.id_path.id() {
                let event = event
                    .payload
                    .downcast_ref()
                    .expect("cast internal event to widget event");
                widget.event(event, state, context);
            } else if event.id_path.starts_with(context.id_path()) {
                self.children.internal_event(event, state, context);
            }
            context.end_widget();
        }
    }
}

impl<V, CS, S> fmt::Debug for WidgetNode<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
    CS: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetNode")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("children", &self.children)
            .field("components", &self.components)
            .field("event_mask", &self.event_mask)
            .finish()
    }
}

pub struct WidgetNodeScope<'a, V: View<S>, CS, S: State> {
    pub id: Id,
    pub status: &'a mut Option<WidgetStatus<V, V::Widget>>,
    pub children: &'a mut <V::Widget as Widget<S>>::Children,
    pub components: &'a mut CS,
}

#[derive(Debug)]
pub enum WidgetStatus<V, W> {
    Uninitialized(V),
    Prepared(W),
    Changed(W, V),
}
