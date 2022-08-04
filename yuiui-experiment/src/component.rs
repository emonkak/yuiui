use std::any::{self, Any};
use std::convert::Infallible;

use crate::element::{AnyElement, Element};
use crate::view::View;

pub trait Component: 'static + AnyComponent {
    type View: View;

    type Component: Component;

    fn render(&self) -> Element<Self::View, Self::Component>;

    fn should_update(&self, _other: &Self) -> bool {
        true
    }
}

impl Component for Infallible {
    type View = Infallible;

    type Component = Infallible;

    fn render(&self) -> Element<Self::View, Self::Component> {
        unreachable!()
    }
}

pub trait AnyComponent {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn render(&self) -> AnyElement;
}

impl<T: Component> AnyComponent for T {
    fn render(&self) -> AnyElement {
        Component::render(self).into()
    }

    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
