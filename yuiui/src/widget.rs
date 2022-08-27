use crate::event::{Event, EventResult};
use crate::id::IdPath;
use crate::state::State;
use crate::widget_node::WidgetNodeSeq;

pub trait Widget<S: State, E>: for<'event> WidgetEvent<'event> {
    type Children: WidgetNodeSeq<S, E>;

    fn lifecycle(
        &mut self,
        _lifecycle: WidgetLifeCycle,
        _children: &Self::Children,
        _id_path: &IdPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn event(
        &mut self,
        _event: <Self as WidgetEvent>::Event,
        _children: &Self::Children,
        _id_path: &IdPath,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }
}

pub trait WidgetEvent<'event> {
    type Event: Event<'event>;
}

#[derive(Debug)]
pub enum WidgetLifeCycle {
    Mounted,
    Updated,
    Unmounted,
}
