mod attributes;
mod command;
mod component;
mod component_proxy;
mod element;
mod lifecycle;
mod reconciler;
mod root;
mod widget;
mod widget_proxy;
mod widget_storage;

pub use attributes::{AnyValue, Attributes};
pub use command::Command;
pub use component::{BoxedComponent, Component};
pub use component_proxy::ComponentProxy;
pub use element::{
    attribute, key, Child, ComponentElement, Element, ElementNode, Key, WidgetElement,
};
pub use lifecycle::Lifecycle;
pub use widget::{RcWidget, Widget};
pub use widget_proxy::WidgetProxy;
pub use widget_storage::{DrawContext, LayoutContext, WidgetStorage};

use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn short_type_name_of<'a>(name: &'a str) -> &'a str {
    name.split('<')
        .next()
        .unwrap_or(name)
        .split("::")
        .last()
        .unwrap_or(name)
}
