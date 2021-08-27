use std::any::{Any, TypeId};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::support::tree::NodeId;
use crate::widget::{downcast_widget, StateCell, WidgetPod, WidgetTree};

use super::EventHandler;

pub struct WidgetHandler<Event, Widget, State> {
    type_id: TypeId,
    node_id: NodeId,
    callback: fn(Arc<Widget>, &Event, StateCell<State>, &mut EventContext),
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
        callback: fn(Arc<Widget>, &Event, StateCell<State>, &mut EventContext),
    ) -> Self {
        Self {
            type_id,
            node_id,
            callback,
        }
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
        update_notifier: &Sender<NodeId>,
    ) {
        let WidgetPod { widget, state, .. } = &*tree[self.node_id];
        (self.callback)(
            downcast_widget(widget.clone()).unwrap(),
            event.downcast_ref::<Event>().unwrap(),
            StateCell::new(state.clone()),
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
