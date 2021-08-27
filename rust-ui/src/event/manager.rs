use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::support::slot_vec::SlotVec;
use crate::widget::element::Children;
use crate::widget::{
    downcast_widget, PolymophicWidget, StateHolder, Widget, WidgetId, WidgetPod, WidgetTree,
};

use super::event::{EventType, GenericEvent};

#[derive(Debug)]
pub struct EventManager<Renderer> {
    listeners: SlotVec<EventListener<Renderer>>,
    listeners_by_type: HashMap<TypeId, Vec<EventListenerId>>,
}

pub struct EventListener<Renderer> {
    pub widget_id: WidgetId,
    pub type_id: TypeId,
    pub callback: EventCallback<Renderer>,
}

pub struct EventContext {
    widget_id: WidgetId,
    update_sender: Sender<WidgetId>,
}

pub type EventCallback<Renderer> = Box<
    dyn Fn(
            Arc<dyn PolymophicWidget<Renderer>>,
            Children<Renderer>,
            StateHolder,
            &Box<dyn Any>,
            EventContext,
        ) + Send
        + Sync,
>;

pub type EventListenerId = usize;

impl<Renderer> EventManager<Renderer> {
    pub fn new() -> Self {
        Self {
            listeners: SlotVec::new(),
            listeners_by_type: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, listener: EventListener<Renderer>) -> EventListenerId {
        let type_id = listener.type_id;
        let listener_id = self.listeners.insert(listener);
        self.listeners_by_type
            .entry(type_id)
            .or_default()
            .push(listener_id);
        listener_id
    }

    pub fn remove_listener(&mut self, listener_id: EventListenerId) -> EventListener<Renderer> {
        let listener = self.listeners.remove(listener_id);
        let listener_ids = self.listeners_by_type.get_mut(&listener.type_id).unwrap();
        if let Some(index) = listener_ids.iter().position(|&id| id == listener_id) {
            listener_ids.swap_remove(index);
        }
        listener
    }

    pub fn dispatch_event(
        &mut self,
        event: &GenericEvent,
        update_sender: &Sender<WidgetId>,
        widget_tree: &WidgetTree<Renderer>,
    ) {
        if let Some(listener_ids) = self.listeners_by_type.get(&event.type_id) {
            for &listener_id in listener_ids {
                let listener = &mut self.listeners[listener_id];
                let WidgetPod {
                    widget,
                    children,
                    state,
                    ..
                } = (*widget_tree[listener.widget_id]).clone();
                let context = EventContext::new(listener.widget_id, update_sender.clone());
                (listener.callback)(widget, children, state, &event.payload, context);
            }
        }
    }
}

impl<Renderer> fmt::Debug for EventListener<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventListener")
            .field("widget_id", &self.widget_id)
            .field("type_id", &self.type_id)
            .field("callback", &"{ .. }")
            .finish()
    }
}

impl<Renderer> EventListener<Renderer> {
    pub fn new<Widget, State, EventType, ListenerFn>(
        widget_id: WidgetId,
        event_type: EventType,
        listener_fn: ListenerFn,
    ) -> Self
    where
        Widget: self::Widget<Renderer> + 'static,
        State: 'static,
        EventType: self::EventType + 'static,
        EventType::Event: 'static,
        ListenerFn: Fn(
                Arc<Widget>,
                Children<Renderer>,
                &mut State,
                &EventType::Event,
                EventContext,
            ) + Sync
            + Send
            + 'static,
    {
        Self {
            widget_id,
            type_id: event_type.type_id(),
            callback: Box::new(move |widget, children, state, event, context| {
                listener_fn(
                    downcast_widget(widget),
                    children,
                    (*state.write().unwrap()).downcast_mut().unwrap(),
                    event.downcast_ref::<EventType::Event>().unwrap(),
                    context,
                )
            }),
        }
    }
}

impl EventContext {
    pub fn new(widget_id: WidgetId, update_sender: Sender<WidgetId>) -> Self {
        Self {
            widget_id,
            update_sender,
        }
    }

    pub fn request_update(&self) {
        self.update_sender.send(self.widget_id).unwrap();
    }
}
