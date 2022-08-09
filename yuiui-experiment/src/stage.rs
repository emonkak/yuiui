use std::fmt;
use std::mem;

use crate::context::{BuildContext, Id, RenderContext};
use crate::element::Element;
use crate::sequence::CommitMode;
use crate::state::{Effect, State};
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub struct Stage<E: Element<S>, S: State> {
    node: WidgetNode<E::View, E::Components, S>,
    state: S,
    context: RenderContext,
    is_mounted: bool,
}

impl<E: Element<S>, S: State> Stage<E, S> {
    pub fn new(element: E, state: S) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&state, &mut context);
        Self {
            node,
            state,
            context,
            is_mounted: false,
        }
    }

    pub fn update(&mut self, element: E) {
        if element.update(self.node.scope(), &self.state, &mut self.context) {
            let mut context = BuildContext::new();
            self.node
                .commit(CommitMode::Update, &self.state, &mut context);
            for (id_path, effect) in context.effects {
                self.run_effect(id_path, effect);
            }
        }
    }

    pub fn commit(&mut self) {
        let mode = if mem::replace(&mut self.is_mounted, true) {
            CommitMode::Update
        } else {
            CommitMode::Mount
        };
        let mut context = BuildContext::new();
        self.node.commit(mode, &self.state, &mut context);
        for (id_path, effect) in context.effects {
            self.run_effect(id_path, effect);
        }
    }

    fn run_effect(&mut self, _id_path: Vec<Id>, effect: Effect<S>) -> bool {
        match effect {
            Effect::Message(message) => self.state.reduce(message),
            Effect::Mutation(mut mutation) => mutation.apply(&mut self.state),
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
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("node", &self.node)
            .field("context", &self.context)
            .field("state", &self.state)
            .finish()
    }
}
