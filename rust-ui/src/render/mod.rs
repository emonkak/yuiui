pub mod tree;

use std::marker::PhantomData;

use crate::event::handler::{EventContext, WidgetHandler};
use crate::event::EventType;
use crate::tree::NodeId;

pub struct RenderContext<Widget: ?Sized, Handle, State> {
    node_id: NodeId,
    _widget: PhantomData<Widget>,
    _handle: PhantomData<Handle>,
    _state: PhantomData<State>,
}

impl<Widget, Handle, State> RenderContext<Widget, Handle, State>
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

    pub fn use_handler<EventType>(
        &self,
        event_type: EventType,
        callback: fn(&Widget, &EventType::Event, &mut State, &mut EventContext),
    ) -> WidgetHandler<EventType, EventType::Event, Widget, State>
    where
        EventType: self::EventType + 'static,
    {
        WidgetHandler::new(event_type, self.node_id, callback)
    }
}
