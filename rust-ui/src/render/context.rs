use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::event::handler::{EventContext, WidgetHandler};
use crate::event::EventType;
use crate::support::tree::NodeId;
use crate::widget::StateCell;

pub struct RenderContext<Widget: ?Sized, Renderer, State> {
    node_id: NodeId,
    _widget: PhantomData<Widget>,
    _handle: PhantomData<Renderer>,
    _state: PhantomData<State>,
}

impl<Widget, Painter, State> RenderContext<Widget, Painter, State>
where
    Widget: 'static,
    State: 'static,
{
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id: node_id,
            _widget: PhantomData,
            _handle: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn use_callback<EventType>(
        &self,
        callback: fn(Arc<Widget>, &EventType::Event, StateCell<State>, &mut EventContext),
    ) -> WidgetHandler<EventType::Event, Widget, State>
    where
        EventType: self::EventType + 'static,
    {
        WidgetHandler::new(TypeId::of::<EventType>(), self.node_id, callback)
    }
}
