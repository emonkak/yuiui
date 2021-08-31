use std::any::Any;

use super::widget::{AsAny, Widget};
use super::element::{Element, ElementId, Children};

pub struct Null<Children> {
    pub children: Children,
}

impl<Renderer: 'static> Widget<Renderer> for Null<Vec<Element<Renderer>>> {
    type State = ();
    type Message = ();

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        self.children.clone()
    }
}

impl<Renderer: 'static> AsAny for Null<Renderer> {
    fn as_any(&self) -> &dyn Any {
       self
    }
}
