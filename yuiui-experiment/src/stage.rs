use crate::context::Context;
use crate::element::Element;
use crate::view_node::ViewNode;

#[allow(dead_code)]
pub struct Stage<E: Element> {
    node: ViewNode<E::View, E::Components>,
    context: Context,
}

impl<E: Element> Stage<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new();
        let node = element.build(&mut context);
        Self { node, context }
    }

    pub fn node(&self) -> &ViewNode<E::View, E::Components> {
        &self.node
    }
}
