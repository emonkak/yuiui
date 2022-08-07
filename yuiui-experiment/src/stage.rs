use crate::context::Context;
use crate::element::Element;
use crate::widget::WidgetNode;

pub struct Stage<E: Element> {
    node: WidgetNode<E::View, E::Components>,
    context: Context,
}

impl<E: Element> Stage<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new();
        let node = element.build(&mut context);
        Self { node, context }
    }

    pub fn update(&mut self, element: E) {
        element.rebuild(self.node.scope(), &mut self.context);
    }

    pub fn node(&self) -> &WidgetNode<E::View, E::Components> {
        &self.node
    }
}
