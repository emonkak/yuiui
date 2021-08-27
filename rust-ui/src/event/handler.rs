use std::any::{Any, TypeId};
use std::fmt;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::widget::{downcast_widget, StateCell, WidgetId, WidgetPod, WidgetTree};

#[derive(Clone, Debug)]
pub struct EventContext {
    widget_id: WidgetId,
    update_notifier: Sender<WidgetId>,
}

pub struct WidgetHandler<Event, Widget, State> {
    type_id: TypeId,
    widget_id: WidgetId,
    callback: fn(Arc<Widget>, &Event, StateCell<State>, EventContext),
}

pub trait EventHandler<Renderer>: Send + Sync {
    fn dispatch(
        &self,
        tree: &WidgetTree<Renderer>,
        event: &Box<dyn Any>,
        update_notifier: &Sender<WidgetId>,
    );

    fn subscribed_type(&self) -> TypeId;

    fn as_ptr(&self) -> *const ();
}

pub type HandlerId = usize;

impl EventContext {
    pub fn notify_changes(&self) {
        self.update_notifier.send(self.widget_id).unwrap();
    }
}

impl<Event, Widget, Renderer, State> EventHandler<Renderer> for WidgetHandler<Event, Widget, State>
where
    Event: 'static,
    Widget: 'static + Any + Send + Sync,
    State: 'static,
{
    fn dispatch(
        &self,
        tree: &WidgetTree<Renderer>,
        event: &Box<dyn Any>,
        update_notifier: &Sender<WidgetId>,
    ) {
        let WidgetPod { widget, state, .. } = &*tree[self.widget_id];
        (self.callback)(
            downcast_widget(widget.clone()),
            event.downcast_ref::<Event>().unwrap(),
            StateCell::new(state.clone()),
            EventContext {
                widget_id: self.widget_id,
                update_notifier: update_notifier.clone(),
            },
        )
    }

    fn subscribed_type(&self) -> TypeId {
        self.type_id
    }

    fn as_ptr(&self) -> *const () {
        self.callback as *const ()
    }
}

impl<Renderer> PartialEq for dyn EventHandler<Renderer> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<Renderer> Eq for dyn EventHandler<Renderer> {}

impl<Renderer> fmt::Debug for dyn EventHandler<Renderer> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_tuple("EventHandler")
            .field(&self.as_ptr())
            .finish()
    }
}

impl<Event, Widget, State> WidgetHandler<Event, Widget, State>
where
    Event: 'static,
    Widget: 'static,
    State: 'static,
{
    pub fn new(
        type_id: TypeId,
        widget_id: WidgetId,
        callback: fn(Arc<Widget>, &Event, StateCell<State>, EventContext),
    ) -> Self {
        Self {
            type_id,
            widget_id,
            callback,
        }
    }
}
