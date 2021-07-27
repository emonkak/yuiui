use std::any::Any;
use std::marker::PhantomData;

use crate::event::handler::{GlobalHandler, WidgetHandler};
use crate::event::{EventContext, EventType};
use crate::tree::NodeId;
use crate::widget::BoxedWidget;
use crate::widget::element::{Children, Key};

#[derive(Debug)]
pub struct RenderState<Handle> {
    pub children: Option<Children<Handle>>,
    pub deleted_children: Vec<(NodeId, BoxedWidget<Handle>)>,
    pub state: Box<dyn Any>,
    pub pending_widget: Option<BoxedWidget<Handle>>,
    pub dirty: bool,
    pub mounted: bool,
    pub key: Option<Key>,
}

impl<Handle> RenderState<Handle> {
    pub fn new(
        state: Box<dyn Any>,
        children: Children<Handle>,
        key: Option<Key>,
    ) -> Self {
        Self {
            children: Some(children),
            deleted_children: Vec::new(),
            state,
            pending_widget: None,
            dirty: true,
            mounted: false,
            key,
        }
    }
}

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
        WidgetHandler::new(event_type, callback, self.node_id)
    }

    pub fn use_global_handler<EventType>(
        &self,
        event_type: EventType,
        callback: fn(&EventType::Event, &mut EventContext),
    ) -> GlobalHandler<EventType, EventType::Event>
    where
        EventType: self::EventType + 'static,
    {
        GlobalHandler::new(event_type, callback)
    }
}
