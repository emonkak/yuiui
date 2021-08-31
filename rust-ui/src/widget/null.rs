use std::any::Any;

use super::element::{Children, Element, ElementId};
use super::widget::{AsAny, ShouldRender, Widget};

pub struct Null<Children> {
    pub children: Children,
}

impl<Renderer: 'static> Widget<Renderer> for Null<Vec<Element<Renderer>>> {
    type State = ();
    type Message = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        self.children.clone()
    }
}

impl<Renderer> ShouldRender<Self> for Null<Renderer> {}

impl<Renderer: 'static> AsAny for Null<Renderer> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
