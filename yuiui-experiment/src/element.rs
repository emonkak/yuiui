use std::any::Any;
use std::convert::Infallible;

use crate::component::{AnyComponent, Component};
use crate::view::{AnyView, View};

#[derive(Debug)]
pub enum Element<V: View, C: Component> {
    View(V, V::Children),
    Component(C),
}

pub fn view<V: View>(node: V, children: V::Children) -> Element<V, Infallible> {
    Element::View(node, children)
}

pub fn component<C: Component>(node: C) -> Element<Infallible, C> {
    Element::Component(node)
}

pub enum AnyElement {
    View(Box<dyn AnyView>, Box<dyn Any>),
    Component(Box<dyn AnyComponent>),
}

impl<V: View, C: Component> From<Element<V, C>> for AnyElement {
    fn from(element: Element<V, C>) -> AnyElement {
        match element {
            Element::View(view, children) => AnyElement::View(Box::new(view), Box::new(children)),
            Element::Component(component) => AnyElement::Component(Box::new(component)),
        }
    }
}
