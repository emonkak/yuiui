use std::any::Any;
use std::marker::PhantomData;

use super::{Component, Element};

pub struct ComponentProxy<C, M, S> {
    component: C,
    message_type: PhantomData<M>,
    state_type: PhantomData<S>,
}

impl<C, M, S> ComponentProxy<C, M, S> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            message_type: PhantomData,
            state_type: PhantomData,
        }
    }
}

impl<C, M> Component<M, dyn Any> for ComponentProxy<C, M, C::State>
where
    C: 'static + Component<M>,
    M: 'static,
    C::State: 'static,
{
    type State = Box<dyn Any>;

    fn initial_state(&self) -> Self::State {
        Box::new(self.component.initial_state())
    }

    fn should_update(
        &self,
        new_component: &dyn Any,
        old_children: &Vec<Element<M>>,
        new_children: &Vec<Element<M>>,
    ) -> bool {
        self.component.should_update(
            new_component.downcast_ref().unwrap(),
            old_children,
            new_children,
        )
    }

    fn render(&self, children: &Vec<Element<M>>, state: &Self::State) -> Element<M> {
        self.component
            .render(children, state.downcast_ref().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.component.type_name()
    }
}
