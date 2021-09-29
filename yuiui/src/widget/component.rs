use std::any::{self, Any};
use std::fmt;
use std::rc::Rc;

use super::{short_type_name_of, AsAny, Children, ComponentProxy, Element};

pub type RcComponent<State, Message> = Rc<dyn Component<State, Message, dyn Any, LocalState = Box<dyn Any>>>;

pub trait Component<State, Message, Own: ?Sized = Self>: AsAny {
    type LocalState;

    fn initial_state(&self) -> Self::LocalState;

    fn should_update(
        &self,
        _new_component: &Own,
        _old_children: &Vec<Element<State, Message>>,
        _new_children: &Vec<Element<State, Message>>,
        _state: &Self::LocalState,
    ) -> bool {
        true
    }

    fn render(&self, children: &Children<State, Message>, state: &Self::LocalState) -> Element<State, Message>;

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn into_rc(self) -> RcComponent<State, Message>
    where
        Self: 'static + Sized + Component<State, Message>,
        <Self as Component<State, Message>>::LocalState: 'static,
        State: 'static,
        Message: 'static,
    {
        Rc::new(ComponentProxy::new(self))
    }
}

impl<State, Message, Own: ?Sized, LocalState> fmt::Debug for dyn Component<State, Message, Own, LocalState = LocalState> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
