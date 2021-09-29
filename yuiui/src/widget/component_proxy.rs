use std::any::Any;
use std::marker::PhantomData;

use super::{Children, Component, Element};

pub struct ComponentProxy<C, M, LS> {
    component: C,
    message_type: PhantomData<M>,
    local_state_type: PhantomData<LS>,
}

impl<C, M, LS> ComponentProxy<C, M, LS> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<C, M> Component<M, dyn Any> for ComponentProxy<C, M, C::LocalState>
where
    C: 'static + Component<M>,
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
        old_children: &Vec<Element<M>>,
        new_children: &Vec<Element<M>>,
        state: &Self::LocalState,
    ) -> bool {
        self.component.should_update(
            &new_component.downcast_ref::<Self>().unwrap().component,
            old_children,
            new_children,
            state.downcast_ref().unwrap(),
        )
    }

    fn render(&self, children: &Children<M>, state: &Self::LocalState) -> Element<M> {
        self.component
            .render(children, state.downcast_ref().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.component.type_name()
    }
}
