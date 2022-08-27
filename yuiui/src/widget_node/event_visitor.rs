use std::any::TypeId;

use crate::component_node::ComponentStack;
use crate::effect::{EffectContext, EffectContextSeq, EffectContextVisitor};
use crate::event::Event;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent};

use super::{WidgetNode, WidgetState};

pub struct EventVisitor<'a, Event> {
    event: &'a Event,
    result: bool,
}

impl<'a, Event: 'static> EventVisitor<'a, Event> {
    pub fn new(event: &'a Event) -> Self {
        Self {
            event,
            result: false,
        }
    }

    pub fn result(&self) -> bool {
        self.result
    }
}

impl<'a, Event: 'static> EffectContextVisitor for EventVisitor<'a, Event> {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E, View = V>,
        S: State,
    {
        match node.state.as_mut().unwrap() {
            WidgetState::Prepared(widget, _) | WidgetState::Dirty(widget, _) => {
                if node.event_mask.contains(&TypeId::of::<Event>()) {
                    EffectContextSeq::for_each(&mut node.children, self, state, env, context);
                }
                if let Some(event) = <V::Widget as WidgetEvent>::Event::from_static(self.event) {
                    let result = widget.event(event, &node.children, context.id_path(), state, env);
                    context.process_result(result);
                    self.result = true;
                }
            }
            _ => {}
        }
    }
}
