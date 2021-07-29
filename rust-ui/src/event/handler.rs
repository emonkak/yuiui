use std::any::{Any, TypeId};

use crate::tree::NodeId;
use crate::widget::{WidgetPod, WidgetTree};

use super::{EventContext, EventHandler, EventType};

pub struct WidgetHandler<EventType, Event, Widget, State> {
    _event_type: EventType,
    callback: fn(&Widget, &Event, &mut State, &mut EventContext),
    node_id: NodeId,
}

impl<EventType, Widget, State> WidgetHandler<EventType, EventType::Event, Widget, State>
where
    EventType: self::EventType + 'static,
    Widget: 'static,
    State: 'static,
{
    pub fn new(
        event_type: EventType,
        callback: fn(&Widget, &EventType::Event, &mut State, &mut EventContext),
        node_id: NodeId,
    ) -> Self {
        Self {
            _event_type: event_type,
            callback,
            node_id,
        }
    }
}

impl<EventType, Widget, Handle, State> EventHandler<Handle>
    for WidgetHandler<EventType, EventType::Event, Widget, State>
where
    Widget: 'static,
    EventType: self::EventType + Send + 'static,
    State: 'static,
{
    fn dispatch(
        &self,
        tree: &WidgetTree<Handle>,
        event: &Box<dyn Any>,
        context: &mut EventContext,
    ) {
        let WidgetPod { widget, state, .. } = &*tree[self.node_id];
        (self.callback)(
            widget.as_any().downcast_ref::<Widget>().unwrap(),
            event.downcast_ref::<EventType::Event>().unwrap(),
            state.lock().unwrap().downcast_mut::<State>().unwrap(),
            context,
        )
    }

    fn subscribed_type(&self) -> TypeId {
        TypeId::of::<EventType>()
    }

    fn as_ptr(&self) -> *const () {
        self.callback as *const ()
    }
}

pub struct GlobalHandler<EventType, Event> {
    _event_type: EventType,
    callback: fn(&Event, &mut EventContext),
}

impl<EventType> GlobalHandler<EventType, EventType::Event>
where
    EventType: self::EventType + 'static,
{
    pub fn new(event_type: EventType, callback: fn(&EventType::Event, &mut EventContext)) -> Self {
        Self {
            _event_type: event_type,
            callback,
        }
    }
}

impl<EventType, Handle> EventHandler<Handle> for GlobalHandler<EventType, EventType::Event>
where
    EventType: self::EventType + Send + 'static,
{
    fn dispatch(
        &self,
        _tree: &WidgetTree<Handle>,
        event: &Box<dyn Any>,
        context: &mut EventContext,
    ) {
        (self.callback)(event.downcast_ref::<EventType::Event>().unwrap(), context)
    }

    fn subscribed_type(&self) -> TypeId {
        TypeId::of::<EventType>()
    }

    fn as_ptr(&self) -> *const () {
        self.callback as *const ()
    }
}
