use std::marker::PhantomData;
use std::sync::Arc;

use crate::event::{EventContext, EventListener, EventType};
use crate::widget::element::Children;
use crate::widget::{Effect, EffectContext, Widget, WidgetId};

pub struct RenderContext<'a, Widget: ?Sized, Renderer> {
    widget_id: WidgetId,
    widget_type: PhantomData<Widget>,
    effects: &'a mut Vec<Effect<Renderer>>,
}

impl<'a, Widget, Renderer> RenderContext<'a, Widget, Renderer> {
    pub fn new(widget_id: WidgetId, effects: &'a mut Vec<Effect<Renderer>>) -> Self {
        Self {
            widget_id,
            widget_type: PhantomData,
            effects,
        }
    }

    pub fn downcast<'b, T>(&'b mut self) -> RenderContext<'b, T, Renderer> {
        RenderContext {
            widget_id: self.widget_id,
            widget_type: PhantomData,
            effects: self.effects,
        }
    }
}

impl<'a, Widget, Renderer> RenderContext<'a, Widget, Renderer>
where
    Widget: self::Widget<Renderer> + 'static,
    Widget::State: 'static,
    Renderer: 'static,
{
    pub fn use_effect<EffectFn, FinalizeFn>(&mut self, effect_fn: EffectFn)
    where
        EffectFn: Fn(
                Arc<Widget>,
                Children<Renderer>,
                &mut Widget::State,
                EffectContext<Renderer>,
            ) -> Option<FinalizeFn>
            + Sync
            + Send
            + 'static,
        FinalizeFn: Fn() + 'static,
    {
        self.effects.push(Effect::new(effect_fn));
    }

    pub fn use_listener<EventType, ListenerFn>(
        &mut self,
        event_type: EventType,
        listener_fn: ListenerFn,
    ) where
        EventType: self::EventType + 'static,
        ListenerFn: Fn(
                Arc<Widget>,
                Children<Renderer>,
                &mut Widget::State,
                &EventType::Event,
                EventContext,
            ) + Sync
            + Send
            + 'static,
    {
        let widget_id = self.widget_id;
        self.effects.push(Effect::new(
            move |_widget: Arc<Widget>,
                  _children,
                  _state: &mut Widget::State,
                  context: EffectContext<Renderer>| {
                let listener = EventListener::new(widget_id, event_type, listener_fn);
                let listener_id = context.add_listener(listener);
                Some(move || {
                    context.remove_listener(listener_id);
                })
            },
        ));
    }
}
