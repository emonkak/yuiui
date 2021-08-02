pub mod tree;

use std::marker::PhantomData;

use crate::event::handler::{EventContext, WidgetHandler};
use crate::event::EventType;
use crate::tree::NodeId;

pub struct RenderContext<Widget: ?Sized, Painter, State> {
    node_id: NodeId,
    _widget: PhantomData<Widget>,
    _handle: PhantomData<Painter>,
    _state: PhantomData<State>,
}

#[derive(Debug)]
pub enum RenderCycle<Widget, Children> {
    WillMount(Children),
    WillUpdate(Children, Widget, Children),
    WillUnmount(Children),
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

impl<Widget, Children> RenderCycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> RenderCycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            RenderCycle::WillMount(children) => RenderCycle::WillMount(children),
            RenderCycle::WillUpdate(children, new_widget, new_children) => {
                RenderCycle::WillUpdate(children, f(new_widget), new_children)
            }
            RenderCycle::WillUnmount(children) => RenderCycle::WillUnmount(children),
        }
    }

    pub fn without_params(&self) -> RenderCycle<(), ()> {
        match self {
            RenderCycle::WillMount(_) => RenderCycle::WillMount(()),
            RenderCycle::WillUpdate(_, _, _) => RenderCycle::WillUpdate((), (), ()),
            RenderCycle::WillUnmount(_) => RenderCycle::WillUnmount(()),
        }
    }
}
