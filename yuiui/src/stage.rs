use std::fmt;
use std::mem;

use crate::context::{ComponentIndex, EffectContext, IdPath, RenderContext};
use crate::effect::Effect;
use crate::element::Element;
use crate::event::{CaptureState, InternalEvent};
use crate::sequence::CommitMode;
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode};

pub struct Stage<El: Element<S, E>, S: State, E> {
    root: WidgetNode<El::View, El::Components, S, E>,
    state: S,
    env: E,
    context: RenderContext,
    is_mounted: bool,
}

impl<El: Element<S, E>, S: State, E> Stage<El, S, E>
where
    El: Element<S, E>,
    S: State,
{
    pub fn new(element: El, state: S, env: E) -> Self {
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

    pub fn update(&mut self, element: El) {
        if element.update(self.root.scope(), &self.state, &self.env, &mut self.context) {
            let mut context = EffectContext::new();
            self.root
                .commit(CommitMode::Update, &self.state, &self.env, &mut context);
            let unit_of_work = context.into_unit_of_work();
            for (id_path, component_index, effect) in unit_of_work.effects {
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
        let unit_of_work = context.into_unit_of_work();
        for (id_path, component_index, effect) in unit_of_work.effects {
            self.run_effect(id_path, component_index, effect);
        }
    }

    pub fn event<EV: 'static>(&mut self, event: &EV) -> CaptureState {
        let mut context = EffectContext::new();
        let capture_state = self.root.event(event, &self.state, &self.env, &mut context);
        let unit_of_work = context.into_unit_of_work();
        for (id_path, component_index, effect) in unit_of_work.effects {
            self.run_effect(id_path, component_index, effect);
        }
        capture_state
    }

    pub fn internal_event(&mut self, event: &InternalEvent) -> CaptureState {
        let mut context = EffectContext::new();
        let capture_state = self
            .root
            .internal_event(event, &self.state, &self.env, &mut context);
        let unit_of_work = context.into_unit_of_work();
        for (id_path, component_index, effect) in unit_of_work.effects {
            self.run_effect(id_path, component_index, effect);
        }
        capture_state
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
            Effect::Command(_) => todo!(),
        }
    }
}

impl<El, S, E> fmt::Debug for Stage<El, S, E>
where
    El: Element<S, E>,
    El::View: View<S, E> + fmt::Debug,
    <El::View as View<S, E>>::Widget: Widget<S, E> + fmt::Debug,
    <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children: fmt::Debug,
    El::Components: fmt::Debug,
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
