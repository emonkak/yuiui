use std::fmt;
use std::mem;

use crate::context::{ComponentIndex, EffectContext, IdPath, RenderContext};
use crate::effect::Effect;
use crate::element::Element;
use crate::env::Env;
use crate::event::InternalEvent;
use crate::sequence::CommitMode;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub struct Stage<EL: Element<S, E>, S: State, E: for<'a> Env<'a>> {
    node: WidgetNode<EL::View, EL::Components, S, E>,
    state: S,
    env: E,
    context: RenderContext,
    is_mounted: bool,
}

impl<EL: Element<S, E>, S: State, E> Stage<EL, S, E>
where
    EL: Element<S, E>,
    S: State,
    E: for<'a> Env<'a>,
{
    pub fn new(element: EL, state: S, env: E) -> Self {
        let mut context = RenderContext::new();
        let node = element.render(&state, &env.as_refs(), &mut context);
        Self {
            node,
            state,
            env,
            context,
            is_mounted: false,
        }
    }

    pub fn update(&mut self, element: EL) {
        if element.update(
            self.node.scope(),
            &self.state,
            &self.env.as_refs(),
            &mut self.context,
        ) {
            let mut context = EffectContext::new();
            self.node.commit(
                CommitMode::Update,
                &self.state,
                &self.env.as_refs(),
                &mut context,
            );
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
        self.node
            .commit(mode, &self.state, &self.env.as_refs(), &mut context);
        for (id_path, component_index, effect) in context.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    pub fn event<EV: 'static>(&mut self, event: &EV) {
        let mut context = EffectContext::new();
        self.node
            .event(event, &self.state, &self.env.as_refs(), &mut context);
        for (id_path, component_index, effect) in context.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    pub fn internal_event(&mut self, event: &InternalEvent) {
        let mut context = EffectContext::new();
        self.node
            .internal_event(event, &self.state, &self.env.as_refs(), &mut context);
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
    E: for<'a> Env<'a>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("node", &self.node)
            .field("state", &self.state)
            .field("context", &self.context)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}