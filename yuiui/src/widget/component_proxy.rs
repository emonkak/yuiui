use std::any::Any;
use std::marker::PhantomData;

use super::{Component, Element};

pub struct ComponentProxy<C, S> {
    component: C,
    state_type: PhantomData<S>,
}

impl<C, S> ComponentProxy<C, S> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            state_type: PhantomData,
        }
    }
}

impl<C> Component<dyn Any> for ComponentProxy<C, C::State>
where
    C: 'static + Component,
    C::State: 'static,
{
    type State = Box<dyn Any>;

    fn initial_state(&self) -> Self::State {
        Box::new(self.component.initial_state())
    }

    fn should_update(
        &self,
        new_component: &dyn Any,
        old_children: &Vec<Element>,
        new_children: &Vec<Element>,
        state: &Self::State,
    ) -> bool {
        self.component.should_update(
            new_component.downcast_ref().unwrap(),
            old_children,
            new_children,
            state.downcast_ref().unwrap(),
        )
    }

    fn render(&self, children: &Vec<Element>, state: &Self::State) -> Element {
        self.component
            .render(children, state.downcast_ref().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.component.type_name()
    }
}
