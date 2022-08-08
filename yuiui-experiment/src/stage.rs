use std::fmt;

use crate::context::Context;
use crate::element::Element;
use crate::sequence::CommitMode;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub struct Stage<E: Element<S>, S: State> {
    node: WidgetNode<E::View, E::Components, S>,
    state: S,
    context: Context,
}

impl<E: Element<S>, S: State> Stage<E, S> {
    pub fn new(element: E, state: S) -> Self {
        let mut context = Context::new();
        let node = element.build(&state, &mut context);
        Self {
            node,
            state,
            context,
        }
    }

    pub fn update(&mut self, element: E) {
        if element.rebuild(self.node.scope(), &self.state, &mut self.context) {
            self.node
                .commit(CommitMode::Update, &self.state, &mut self.context);
        }
    }
}

impl<E, S> fmt::Debug for Stage<E, S>
where
    E: Element<S>,
    E::View: View<S> + fmt::Debug,
    <E::View as View<S>>::Widget: Widget<S> + fmt::Debug,
    <<E::View as View<S>>::Widget as Widget<S>>::Children: fmt::Debug,
    E::Components: fmt::Debug,
    S: State + fmt::Debug,
    S::Message: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("node", &self.node)
            .field("context", &self.context)
            .field("state", &self.state)
            .finish()
    }
}
