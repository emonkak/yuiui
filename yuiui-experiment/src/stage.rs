use crate::context::Context;
use crate::element::Element;
use crate::view::View;
use crate::widget::WidgetNode;

#[allow(dead_code)]
pub struct Stage<E: Element> {
    node: WidgetNode<<E::View as View>::Widget, E::Components>,
    context: Context,
}

impl<E: Element> Stage<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new();
        let node = element.build(&mut context);
        Self { node, context }
    }

    pub fn node(&self) -> &WidgetNode<<E::View as View>::Widget, E::Components> {
        &self.node
    }
}
