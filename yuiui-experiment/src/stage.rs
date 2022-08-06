use crate::context::Context;
use crate::element::Element;
use crate::node::UINode;

#[allow(dead_code)]
pub struct Stage<E: Element> {
    node: UINode<E::View, E::Components>,
    context: Context,
}

impl<E: Element> Stage<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new();
        let node = element.build(&mut context);
        Self {
            node,
            context,
        }
    }

    pub fn node(&self) -> &UINode<E::View, E::Components> {
        &self.node
    }
}
