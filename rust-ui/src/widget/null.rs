use std::any::Any;

use super::element::{Children, Element, ElementId};
use super::state::StateContainer;
use super::widget::{Widget, WidgetSeal};

pub struct Null<Renderer> {
    pub children: Vec<Element<Renderer>>,
}

impl<Renderer: 'static> Widget<Renderer> for Null<Renderer> {
    type State = ();
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_pure_state(())
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
