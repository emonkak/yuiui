use std::any::{Any, TypeId};
use std::sync::mpsc::Sender;

use crate::tree::NodeId;
use crate::widget::tree::{WidgetPod, WidgetTree};

use super::{EventHandler, EventType};

pub struct WidgetHandler<EventType, Event, Widget, State> {
    _event_type: EventType,
    node_id: NodeId,
    callback: fn(&Widget, &Event, &mut State, &mut EventContext),
}

pub struct EventContext<'a> {
    node_id: NodeId,
    update_notifier: &'a Sender<NodeId>,
}

impl<EventType, Widget, State> WidgetHandler<EventType, EventType::Event, Widget, State>
where
    EventType: self::EventType + 'static,
    Widget: 'static,
    State: 'static,
{
    pub fn new(
        event_type: EventType,
        node_id: NodeId,
        callback: fn(&Widget, &EventType::Event, &mut State, &mut EventContext),
    ) -> Self {
        Self {
            _event_type: event_type,
            node_id,
            callback,
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
        update_notifier: &Sender<NodeId>,
    ) {
        let WidgetPod { widget, state, .. } = &*tree[self.node_id];
        (self.callback)(
            widget.as_any().downcast_ref::<Widget>().unwrap(),
            event.downcast_ref::<EventType::Event>().unwrap(),
            state.lock().unwrap().downcast_mut::<State>().unwrap(),
            &mut EventContext {
                node_id: self.node_id,
                update_notifier,
            },
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
    root_id: NodeId,
    callback: fn(&Event, &mut EventContext),
}

impl<EventType> GlobalHandler<EventType, EventType::Event>
where
    EventType: self::EventType + 'static,
{
    pub fn new(
        event_type: EventType,
        root_id: NodeId,
        callback: fn(&EventType::Event, &mut EventContext),
    ) -> Self {
        Self {
            _event_type: event_type,
            root_id,
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
        update_notifier: &Sender<NodeId>,
    ) {
        (self.callback)(
            event.downcast_ref::<EventType::Event>().unwrap(),
            &mut EventContext {
                node_id: self.root_id,
                update_notifier,
            },
        )
    }

    fn subscribed_type(&self) -> TypeId {
        TypeId::of::<EventType>()
    }

    fn as_ptr(&self) -> *const () {
        self.callback as *const ()
    }
}

impl<'a> EventContext<'a> {
    pub fn notify_changes(&self) {
        self.update_notifier.send(self.node_id).unwrap();
    }
}
