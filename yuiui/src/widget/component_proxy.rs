use std::any::Any;
use std::marker::PhantomData;

use super::{Children, Component, Event, Element, Effect, Lifecycle};

pub struct ComponentProxy<C, S, M, LS> {
    component: C,
    state_type: PhantomData<S>,
    message_type: PhantomData<M>,
    local_state_type: PhantomData<LS>,
}

impl<C, S, M, LS> ComponentProxy<C, S, M, LS> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            state_type: PhantomData,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<C, S, M> Component<S, M, dyn Any> for ComponentProxy<C, S, M, C::LocalState>
where
    C: 'static + Component<S, M>,
    S: 'static,
    M: 'static,
    C::LocalState: 'static,
{
    type LocalState = Box<dyn Any>;

    fn initial_state(&self) -> Self::LocalState {
        Box::new(self.component.initial_state())
    }

    fn should_update(
        &self,
        new_component: &dyn Any,
        old_children: &Vec<Element<S, M>>,
        new_children: &Vec<Element<S, M>>,
        state: &Self::LocalState,
    ) -> bool {
        self.component.should_update(
            &new_component.downcast_ref::<Self>().unwrap().component,
            old_children,
            new_children,
            state.downcast_ref().unwrap(),
        )
    }

    fn on_event(&self, _event: &Event<S>, _state: &mut Self::LocalState) -> Effect<M> {
        Effect::None
    }

    fn on_lifecycle(
        &self,
        _lifecycle: Lifecycle<&dyn Any>,
        _state: &mut Self::LocalState,
    ) -> Effect<M> {
        Effect::None
    }

    fn render(&self, children: &Children<S, M>, state: &Self::LocalState) -> Element<S, M> {
        self.component
            .render(children, state.downcast_ref().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.component.type_name()
    }
}
