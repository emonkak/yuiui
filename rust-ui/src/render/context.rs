use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::event::{EventType, EventContext, WidgetHandler};
use crate::widget::{StateCell, WidgetId};

pub struct RenderContext<Widget: ?Sized, Renderer, State> {
    widget_id: WidgetId,
    _widget: PhantomData<Widget>,
    _handle: PhantomData<Renderer>,
    _state: PhantomData<State>,
}

impl<Widget, Painter, State> RenderContext<Widget, Painter, State>
where
    Widget: 'static,
    State: 'static,
{
    pub fn new(widget_id: WidgetId) -> Self {
        Self {
            widget_id,
            _widget: PhantomData,
            _handle: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn use_callback<EventType>(
        &self,
        callback: fn(Arc<Widget>, &EventType::Event, StateCell<State>, EventContext),
    ) -> WidgetHandler<EventType::Event, Widget, State>
    where
        EventType: self::EventType + 'static,
    {
        WidgetHandler::new(TypeId::of::<EventType>(), self.widget_id, callback)
    }
}
