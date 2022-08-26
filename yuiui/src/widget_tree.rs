use std::any::Any;
use std::fmt;
use std::mem;

use crate::component_node::ComponentStack;
use crate::effect::{Effect, EffectContext, EffectPath};
use crate::element::Element;
use crate::event::InternalEvent;
use crate::id::IdContext;
use crate::sequence::{NodeVisitor, TraversableSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::Widget;
use crate::widget_node::{CommitMode, WidgetNode};

pub struct WidgetTree<El: Element<S, E>, S: State, E> {
    root: WidgetNode<El::View, El::Components, S, E>,
    state: S,
    env: E,
    context: IdContext,
    is_mounted: bool,
}

impl<El: Element<S, E>, S: State, E> WidgetTree<El, S, E>
where
    El: Element<S, E>,
    S: State,
{
    pub fn new(element: El, state: S, env: E) -> Self {
        let mut context = IdContext::new();
        let root = element.render(&state, &env, &mut context);
        Self {
            root,
            state,
            env,
            context,
            is_mounted: false,
        }
    }

    pub fn update(&mut self, element: El) -> Vec<(EffectPath, Effect<S>)> {
        let mut context = EffectContext::new();
        if element.update(self.root.scope(), &self.state, &self.env, &mut self.context) {
            self.root
                .commit(CommitMode::Update, &self.state, &self.env, &mut context);
        }
        context.into_effects()
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

    pub fn event<Event: 'static>(&mut self, event: &Event) -> Vec<(EffectPath, Effect<S>)> {
        let mut context = EffectContext::new();
        let _ = self.root.event(event, &self.state, &self.env, &mut context);
        context.into_effects()
    }

    pub fn internal_event(&mut self, event: &InternalEvent) -> Vec<(EffectPath, Effect<S>)>
    where
        <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children:
            for<'a> TraversableSeq<InternalEventVisitor<'a>, S, E>,
    {
        let mut visitor = InternalEventVisitor {
            event: event.payload(),
        };
        let mut context = EffectContext::new();
        let _ = self.root.search(
            event.id_path(),
            &mut visitor,
            &self.state,
            &self.env,
            &mut context,
        );
        context.into_effects()
    }
}

impl<El, S, E> fmt::Debug for WidgetTree<El, S, E>
where
    El: Element<S, E>,
    El::View: View<S, E> + fmt::Debug,
    <El::View as View<S, E>>::Widget: Widget<S, E> + fmt::Debug,
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

pub struct InternalEventVisitor<'a> {
    event: &'a dyn Any,
}

impl<'a, V, CS, S, E> NodeVisitor<WidgetNode<V, CS, S, E>, S, E> for InternalEventVisitor<'a>
where
    V: View<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    fn visit(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        node.any_event(self.event, state, env, context)
    }
}
