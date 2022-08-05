use std::pin::Pin;

use crate::element::Element;
use crate::view::{View, ViewPod};
use crate::widget::WidgetPod;

pub struct VirtualWorld<E: Element> {
    tree: Pin<Box<ViewPod<E::View, E::Components>>>,
}

impl<E: Element> VirtualWorld<E> {
    pub fn new(element: E) -> Self {
        let tree = Box::pin(element.build());
        Self { tree }
    }

    pub fn render(&self) -> RealWorld<E> {
        let tree = Box::pin(E::compile(&self.tree()));
        RealWorld { tree }
    }

    pub fn tree(&self) -> &ViewPod<E::View, E::Components> {
        Pin::get_ref(self.tree.as_ref())
    }
}

pub struct RealWorld<E: Element> {
    tree: Pin<Box<WidgetPod<<E::View as View>::Widget>>>,
}

impl<E: Element> RealWorld<E> {
    pub fn tree(&self) -> &WidgetPod<<E::View as View>::Widget> {
        Pin::get_ref(self.tree.as_ref())
    }
}
