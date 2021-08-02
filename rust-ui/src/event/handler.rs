use std::any::{Any, TypeId};
use std::sync::mpsc::Sender;

use crate::tree::NodeId;
use crate::widget::tree::{WidgetPod, WidgetTree};

use super::EventHandler;

pub struct WidgetHandler<Event, Widget, State> {
    type_id: TypeId,
    node_id: NodeId,
    callback: fn(&Widget, &Event, &mut State, &mut EventContext),
}

pub struct EventContext<'a> {
    node_id: NodeId,
    update_notifier: &'a Sender<NodeId>,
}

impl<Event, Widget, State> WidgetHandler<Event, Widget, State>
where
    Event: 'static,
    Widget: 'static,
    State: 'static,
{
    pub fn new(
        type_id: TypeId,
        node_id: NodeId,
        callback: fn(&Widget, &Event, &mut State, &mut EventContext),
    ) -> Self {
        Self {
            type_id,
            node_id,
            callback,
        }
    }
}

impl<Event, Widget, Painter, State> EventHandler<Painter> for WidgetHandler<Event, Widget, State>
where
    Event: 'static,
    Widget: 'static,
    State: 'static,
{
    fn dispatch(
        &self,
        tree: &WidgetTree<Painter>,
        event: &Box<dyn Any>,
        update_notifier: &Sender<NodeId>,
    ) {
        let WidgetPod { widget, state, .. } = &*tree[self.node_id];
        (self.callback)(
            widget.as_any().downcast_ref::<Widget>().unwrap(),
            event.downcast_ref::<Event>().unwrap(),
            state.lock().unwrap().downcast_mut::<State>().unwrap(),
            &mut EventContext {
                node_id: self.node_id,
                update_notifier,
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

impl<'a> EventContext<'a> {
    pub fn notify_changes(&self) {
        self.update_notifier.send(self.node_id).unwrap();
    }
}
