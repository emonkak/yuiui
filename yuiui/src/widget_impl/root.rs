use crate::widget::{ElementInstance, Widget};

#[derive(Debug)]
pub struct Root;

impl<Message> Widget<Message> for Root {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }
}

impl<Message: 'static> From<Root> for ElementInstance<Message> {
    fn from(widget: Root) -> Self {
        widget.into_rc().into()
    }
}
