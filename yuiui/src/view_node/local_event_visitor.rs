use std::any::Any;

use crate::component_node::ComponentStack;
use crate::context::EffectContext;
use crate::event::{Event, HasEvent};
use crate::state::State;
use crate::traversable::TraversableVisitor;
use crate::view::View;

use super::{ViewNode, ViewNodeState};

pub struct LocalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a> LocalEventVisitor<'a> {
    pub fn new(event: &'a dyn Any) -> Self {
        Self { event }
    }
}

impl<'a, V, CS, S, E> TraversableVisitor<ViewNode<V, CS, S, E>, EffectContext<S>, S, E>
    for LocalEventVisitor<'a>
where
    V: View<S, E>,
    CS: ComponentStack<S, E, View = V>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut ViewNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        context.set_component_index(CS::LEN);
        match node.state.as_mut().unwrap() {
            ViewNodeState::Prepared(view, widget) | ViewNodeState::Pending(view, _, widget) => {
                let event = <V as HasEvent>::Event::from_any(self.event)
                    .expect("cast any event to view event");
                let result = view.event(
                    event,
                    widget,
                    &node.children,
                    context.effect_path(),
                    state,
                    env,
                );
                context.process_result(result);
            }
            ViewNodeState::Uninitialized(_) => {}
        }
    }
}
