mod array;
mod either;
mod hlist;
mod option;
mod vec;
mod widget_node;

use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::event::{CaptureState, EventMask, InternalEvent};
use crate::state::State;

pub trait ElementSeq<S: State, E> {
    type Store: WidgetNodeSeq<S, E>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store;

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait WidgetNodeSeq<S: State, E> {
    fn event_mask() -> EventMask;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState;

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> CaptureState;
}

pub trait TraversableSeq<C> {
    fn for_each(self, callback: &mut C) -> ControlFlow<()>;
}

pub trait CallbackMut<T> {
    fn call(&mut self, value: T) -> ControlFlow<()>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount | Self::Unmount => true,
            Self::Update => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
