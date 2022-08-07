use std::fmt;

use crate::context::Context;
use crate::element::Element;
use crate::view::View;
use crate::widget::{CommitMode, Widget, WidgetNode};

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
        if element.rebuild(self.node.scope(), &mut self.context) {
            self.node.commit(CommitMode::Update, &mut self.context);
        }
    }
}

impl<E: Element> fmt::Debug for Stage<E>
where
    E::View: View + fmt::Debug,
    <E::View as View>::Widget: Widget + fmt::Debug,
    <<E::View as View>::Widget as Widget>::Children: fmt::Debug,
    E::Components: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("node", &self.node)
            .field("context", &self.context)
            .finish()
    }
}
