use std::any::{self, Any};
use std::fmt;
use std::rc::Rc;

use super::{short_type_name_of, AsAny, Children, ComponentProxy, Element};

pub type RcComponent<Message> = Rc<dyn Component<Message, dyn Any, State = Box<dyn Any>>>;

pub trait Component<Message, Own: ?Sized = Self>: AsAny {
    type State;

    fn initial_state(&self) -> Self::State;

    fn should_update(
        &self,
        _new_component: &Own,
        _old_children: &Vec<Element<Message>>,
        _new_children: &Vec<Element<Message>>,
        _state: &Self::State,
    ) -> bool {
        true
    }

    fn render(&self, children: &Children<Message>, state: &Self::State) -> Element<Message>;

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn into_rc(self) -> RcComponent<Message>
    where
        Self: 'static + Sized + Component<Message>,
        <Self as Component<Message>>::State: 'static,
        Message: 'static,
    {
        Rc::new(ComponentProxy::new(self))
    }
}

impl<M, O: ?Sized, S> fmt::Debug for dyn Component<M, O, State = S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
