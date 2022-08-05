use std::any::{self, Any};

use crate::element::Element;

pub trait Component: 'static + AnyComponent {
    type Element: Element;

    fn render(&self) -> Self::Element;

    fn should_update(&self, _other: &Self) -> bool {
        true
    }
}

pub trait AnyComponent {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;
}

impl<T: Component> AnyComponent for T {
    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
