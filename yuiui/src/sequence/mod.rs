mod array;
mod either;
mod hlist;
mod option;
mod vec;
mod widget_node;

use std::ops::ControlFlow;

use crate::context::{EffectContext, RenderContext};
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::state::State;

pub trait ElementSeq<S: State> {
    type Store: WidgetNodeSeq<S>;

    fn render(self, state: &S, context: &mut RenderContext) -> Self::Store;

    fn update(self, store: &mut Self::Store, state: &S, context: &mut RenderContext) -> bool;
}

pub trait WidgetNodeSeq<S: State> {
    fn event_mask() -> EventMask;

    fn commit(&mut self, mode: CommitMode, state: &S, context: &mut EffectContext<S>);

    fn event<E: 'static>(
        &mut self,
        event: &E,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult;

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        context: &mut EffectContext<S>,
    ) -> EventResult;
}

pub trait TraversableSeq<C> {
    fn for_each(&self, callback: &mut C) -> ControlFlow<()>;
}

pub trait SeqCallback<T> {
    fn call(&mut self, value: &T) -> ControlFlow<()>;
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
