use crate::widget::{ElementInstance, Widget};

#[derive(Debug)]
pub struct Null;

impl<State, Message> Widget<State, Message> for Null {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }
}

impl<State: 'static, Message: 'static> From<Null> for ElementInstance<State, Message> {
    fn from(widget: Null) -> Self {
        widget.into_rc().into()
    }
}
