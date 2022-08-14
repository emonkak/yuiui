use std::any::Any;
use std::marker::PhantomData;

use super::{Children, Component, Effect, Element, Event, Lifecycle};

pub struct ComponentProxy<Inner, State, Message, LocalState> {
    inner: Inner,
    state_type: PhantomData<State>,
    message_type: PhantomData<Message>,
    local_state_type: PhantomData<LocalState>,
}

impl<Inner, State, Message, LocalState> ComponentProxy<Inner, State, Message, LocalState> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            state_type: PhantomData,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<Inner, State, Message> Component<State, Message, dyn Any>
    for ComponentProxy<Inner, State, Message, Inner::LocalState>
where
    Inner: 'static + Component<State, Message>,
    State: 'static,
    Message: 'static,
    Inner::LocalState: 'static,
{
    type LocalState = Box<dyn Any>;

    fn initial_state(&self) -> Self::LocalState {
        Box::new(self.inner.initial_state())
    }

    fn should_update(
        &self,
        new_component: &dyn Any,
        old_children: &Vec<Element<State, Message>>,
        new_children: &Vec<Element<State, Message>>,
        state: &Self::LocalState,
    ) -> bool {
        self.inner.should_update(
            &new_component.downcast_ref::<Self>().unwrap().inner,
            old_children,
            new_children,
            state.downcast_ref().unwrap(),
        )
    }

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn Any>,
        state: &mut Self::LocalState,
    ) -> Effect<Message> {
        self.inner.on_lifecycle(
            lifecycle.map(|component| &component.downcast_ref::<Self>().unwrap().inner),
            state.downcast_mut().unwrap(),
        )
    }

    fn on_event(&self, event: Event<State>, state: &mut Self::LocalState) -> Effect<Message> {
        self.inner.on_event(event, state.downcast_mut().unwrap())
    }

    fn render(
        &self,
        children: &Children<State, Message>,
        state: &Self::LocalState,
    ) -> Element<State, Message> {
        self.inner.render(children, state.downcast_ref().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }
}
