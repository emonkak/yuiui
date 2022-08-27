use std::fmt;
use std::mem;

use crate::effect::{Effect, EffectContext, EffectPath};
use crate::element::Element;
use crate::event::InternalEvent;
use crate::render::{ComponentIndex, IdPath, RenderContext};
use crate::sequence::CommitMode;
use crate::state::State;
use crate::view::View;
use crate::widget::Widget;
use crate::widget_node::WidgetNode;

pub struct WidgetTree<El: Element<S, E>, S: State, E> {
    pub root: WidgetNode<El::View, El::Components, S, E>,
    state: S,
    env: E,
    context: RenderContext,
    is_mounted: bool,
}

impl<El: Element<S, E>, S: State, E> WidgetTree<El, S, E>
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

    pub fn update(&mut self, element: El) -> bool {
        element.update(self.root.scope(), &self.state, &self.env, &mut self.context)
    }

    pub fn update_subtree(&mut self, _id_path: &IdPath, _component_index: ComponentIndex) -> bool {
        false
    }

    pub fn commit(&mut self) -> Vec<(EffectPath, Effect<S>)> {
        let mode = if mem::replace(&mut self.is_mounted, true) {
            CommitMode::Update
        } else {
            CommitMode::Mount
        };
        let mut context = EffectContext::new();
        self.root.commit(mode, &self.state, &self.env, &mut context);
        context.into_effects()
    }

    pub fn commit_subtree(&mut self, id_path: &IdPath) -> Vec<(EffectPath, Effect<S>)> {
        let mut context = EffectContext::new();
        self.root.commit_subtree(
            CommitMode::Update,
            id_path,
            &self.state,
            &self.env,
            &mut context,
        );
        context.into_effects()
    }

    pub fn event<Event: 'static>(&mut self, event: &Event) -> Vec<(EffectPath, Effect<S>)> {
        let mut context = EffectContext::new();
        self.root.event(event, &self.state, &self.env, &mut context);
        context.into_effects()
    }

    pub fn internal_event(&mut self, event: &InternalEvent) -> Vec<(EffectPath, Effect<S>)> {
        let mut context = EffectContext::new();
        self.root
            .internal_event(event, &self.state, &self.env, &mut context);
        context.into_effects()
    }
}

impl<El, S, E> fmt::Debug for WidgetTree<El, S, E>
where
    El: Element<S, E>,
    El::View: fmt::Debug,
    <El::View as View<S, E>>::Widget: fmt::Debug,
    <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children: fmt::Debug,
    El::Components: fmt::Debug,
    S: State + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetTree")
            .field("root", &self.root)
            .field("state", &self.state)
            .field("context", &self.context)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}
