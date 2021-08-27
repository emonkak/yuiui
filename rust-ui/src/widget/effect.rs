use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::event::{EventListener, EventListenerId, EventManager};

use super::element::Children;
use super::widget::{downcast_widget, PolymophicWidget, Widget, StateHolder};
use super::widget_tree::WidgetId;

pub struct Effect<Renderer> {
    callback: EffectCallback<Renderer>,
}

pub type EffectCallback<Renderer> = Box<
    dyn FnOnce(
            &Arc<dyn PolymophicWidget<Renderer>>,
            &Children<Renderer>,
            &StateHolder,
            &EffectContext<Renderer>,
        ) -> Option<EffectFinalizer>
        + Send
        + Sync,
>;

pub type EffectFinalizer = Box<dyn FnOnce()>;

#[derive(Debug)]
pub struct EffectContext<Renderer> {
    widget_id: WidgetId,
    update_sender: Sender<WidgetId>,
    event_manager: Arc<Mutex<EventManager<Renderer>>>,
}

impl<Renderer> Effect<Renderer> {
    pub fn new<Widget, State, EffectFn, FinalizeFn>(effect_fn: EffectFn) -> Self
    where
        Widget: self::Widget<Renderer> + 'static,
        State: 'static,
        EffectFn: FnOnce(
                Arc<Widget>,
                Children<Renderer>,
                &mut State,
                EffectContext<Renderer>,
            ) -> Option<FinalizeFn>
            + Sync
            + Send
            + 'static,
        FinalizeFn: FnOnce() + 'static,
    {
        Self {
            callback: Box::new(move |widget, children, state, context| {
                let finalizer = effect_fn(
                    downcast_widget(widget.clone()),
                    children.clone(),
                    (*state.write().unwrap()).downcast_mut().unwrap(),
                    context.clone(),
                );
                finalizer.map(|f| Box::new(f) as EffectFinalizer)
            }),
        }
    }

    pub fn apply(
        self,
        widget: &Arc<dyn PolymophicWidget<Renderer>>,
        children: &Children<Renderer>,
        state: &StateHolder,
        context: &EffectContext<Renderer>,
    ) -> Option<EffectFinalizer> {
        (self.callback)(widget, children, state, context)
    }
}

impl<Renderer> fmt::Debug for Effect<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Effect {{ .. }}")
    }
}

impl<Renderer> EffectContext<Renderer> {
    pub fn new(
        widget_id: WidgetId,
        update_sender: Sender<WidgetId>,
        event_manager: Arc<Mutex<EventManager<Renderer>>>,
    ) -> Self {
        Self {
            widget_id,
            update_sender,
            event_manager,
        }
    }

    pub fn request_update(&self) {
        self.update_sender.send(self.widget_id).unwrap();
    }

    pub fn add_listener(&self, listener: EventListener<Renderer>) -> EventListenerId {
        self.event_manager.lock().unwrap().add_listener(listener)
    }

    pub fn remove_listener(&self, listener_id: EventListenerId) -> EventListener<Renderer> {
        self.event_manager
            .lock()
            .unwrap()
            .remove_listener(listener_id)
    }
}

impl<Renderer> Clone for EffectContext<Renderer> {
    fn clone(&self) -> Self {
        Self {
            widget_id: self.widget_id,
            update_sender: self.update_sender.clone(),
            event_manager: self.event_manager.clone(),
        }
    }
}
