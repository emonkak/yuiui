use std::any::Any;

use super::element::{Children, Element, ElementId};
use super::widget::{Widget, WidgetSeal};

pub struct Null<Renderer> {
    pub children: Vec<Element<Renderer>>,
}

impl<Renderer: 'static> Widget<Renderer> for Null<Renderer> {
    type State = ();
    type Message = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        self.children.clone()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Renderer> WidgetSeal for Null<Renderer> {}
