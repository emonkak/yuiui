use std::convert::Infallible;

use crate::view::View;
use crate::component::Component;

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
