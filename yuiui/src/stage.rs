use std::fmt;
use std::mem;

use crate::context::{ComponentIndex, EffectContext, IdPath, RenderContext};
use crate::effect::Effect;
use crate::element::Element;
use crate::event::InternalEvent;
use crate::sequence::CommitMode;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub struct Stage<EL: Element<S, E>, S: State, E> {
    root: WidgetNode<EL::View, EL::Components, S, E>,
    state: S,
    env: E,
    context: RenderContext,
    is_mounted: bool,
}

impl<EL: Element<S, E>, S: State, E> Stage<EL, S, E>
where
    EL: Element<S, E>,
    S: State,
{
    pub fn new(element: EL, state: S, env: E) -> Self {
        let mut context = RenderContext::new();
        let root = element.render(&state, &env, &mut context);
        Self {
            root,
            state,
            env,
            context,
            is_mounted: false,
        }
    }

    pub fn update(&mut self, element: EL) {
        if element.update(self.root.scope(), &self.state, &self.env, &mut self.context) {
            let mut context = EffectContext::new();
            self.root
                .commit(CommitMode::Update, &self.state, &self.env, &mut context);
            for (id_path, component_index, effect) in context.effects {
                self.run_effect(id_path, component_index, effect);
            }
        }
    }

    pub fn commit(&mut self) {
        let mode = if mem::replace(&mut self.is_mounted, true) {
            CommitMode::Update
        } else {
            CommitMode::Mount
        };
        let mut context = EffectContext::new();
        self.root.commit(mode, &self.state, &self.env, &mut context);
        for (id_path, component_index, effect) in context.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    pub fn event<EV: 'static>(&mut self, event: &EV) {
        let mut context = EffectContext::new();
        self.root.event(event, &self.state, &self.env, &mut context);
        for (id_path, component_index, effect) in context.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    pub fn internal_event(&mut self, event: &InternalEvent) {
        let mut context = EffectContext::new();
        self.root
            .internal_event(event, &self.state, &self.env, &mut context);
        for (id_path, component_index, effect) in context.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    fn run_effect(
        &mut self,
        _id_path: IdPath,
        _component_index: Option<ComponentIndex>,
        effect: Effect<S>,
    ) -> bool {
        match effect {
            Effect::Message(message) => self.state.reduce(message),
            Effect::Mutation(mut mutation) => mutation.apply(&mut self.state),
        }
    }
}

impl<EL, S, E> fmt::Debug for Stage<EL, S, E>
where
    EL: Element<S, E>,
    EL::View: View<S, E> + fmt::Debug,
    <EL::View as View<S, E>>::Widget: Widget<S, E> + fmt::Debug,
    <<EL::View as View<S, E>>::Widget as Widget<S, E>>::Children: fmt::Debug,
    EL::Components: fmt::Debug,
    S: State + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("root", &self.root)
            .field("state", &self.state)
            .field("context", &self.context)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}
