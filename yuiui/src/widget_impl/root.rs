use crate::widget::{ElementInstance, Widget};

#[derive(Debug)]
pub struct Root;

impl<State, Message> Widget<State, Message> for Root {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }
}

impl<State: 'static, Message: 'static> From<Root> for ElementInstance<State, Message> {
    fn from(widget: Root) -> Self {
        widget.into_rc().into()
    }
}
