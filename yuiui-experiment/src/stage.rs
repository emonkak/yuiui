use crate::context::Context;
use crate::element::Element;
use crate::node::{UINode, VNode};
use crate::view::View;

pub struct Stage<E: Element> {
    v_node: VNode<E::View, E::Components>,
    ui_node: UINode<<E::View as View>::Widget>,
    context: Context,
}

impl<E: Element> Stage<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new(E::depth());
        let v_node = element.build(&mut context);
        let ui_node = E::render(&v_node);
        Self {
            v_node,
            ui_node,
            context,
        }
    }

    pub fn v_node(&self) -> &VNode<E::View, E::Components> {
        &self.v_node
    }

    pub fn ui_node(&self) -> &UINode<<E::View as View>::Widget> {
        &self.ui_node
    }
}
