use std::any::{self, Any};
use std::fmt;
use std::rc::Rc;

use super::{short_type_name_of, AsAny, Attributes, ComponentProxy, Element};

pub type BoxedComponent = Rc<dyn Component<dyn Any, State = Box<dyn Any>>>;

pub trait Component<Own: ?Sized = Self>: AsAny {
    type State;

    fn initial_state(&self) -> Self::State;

    fn should_update(
        &self,
        _new_component: &Own,
        _old_attributes: &Attributes,
        _new_attributes: &Attributes,
        _old_children: &Vec<Element>,
        _new_children: &Vec<Element>,
        _state: &Self::State,
    ) -> bool {
        true
    }

    fn render(&self, children: &Vec<Element>, state: &Self::State) -> Element;

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn into_boxed(self) -> BoxedComponent
    where
        Self: 'static + Sized + Component<Self>,
    {
        Rc::new(ComponentProxy::new(self))
    }
}

impl<O: ?Sized, S> fmt::Debug for dyn Component<O, State = S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
