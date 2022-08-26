use std::fmt;
use std::mem;

use crate::effect::{Effect, EffectContext, EffectPath};
use crate::element::Element;
use crate::event::InternalEvent;
use crate::id::{ComponentIndex, IdContext, IdPath};
use crate::sequence::TraversableSeq;
use crate::state::State;
use crate::view::View;
use crate::widget::Widget;
use crate::widget_node::{
    CommitMode, InternalEventVisitor, RerenderComponent, StaticEventVisitor, SubtreeUpdateVisitor,
    WidgetNode, WidgetNodeScope,
};

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

    pub fn update(&mut self, element: El) -> bool {
        element.update(self.root.scope(), &self.state, &self.env, &mut self.context)
    }

    pub fn update_subtree(&mut self, id_path: &IdPath, component_index: ComponentIndex) -> bool
    where
        for<'widget> WidgetNodeScope<'widget, El::View, El::Components, S, E>:
            RerenderComponent<S, E>,
        <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children:
            TraversableSeq<SubtreeUpdateVisitor, S, E, IdContext>,
    {
        let mut visitor = SubtreeUpdateVisitor::new(component_index);
        let _ = self.root.search(
            id_path,
            &mut visitor,
            &self.state,
            &self.env,
            &mut self.context,
        );
        visitor.has_changed()
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

    pub fn event<Event>(&mut self, event: &Event) -> Vec<(EffectPath, Effect<S>)>
    where
        Event: 'static,
        <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children:
            for<'a> TraversableSeq<StaticEventVisitor<'a, Event>, S, E, EffectContext<S>>,
    {
        let mut visitor = StaticEventVisitor::new(event);
        let mut context = EffectContext::new();
        let _ = self
            .root
            .for_each(&mut visitor, &self.state, &self.env, &mut context);
        context.into_effects()
    }

    pub fn internal_event(&mut self, event: &InternalEvent) -> Vec<(EffectPath, Effect<S>)>
    where
        <<El::View as View<S, E>>::Widget as Widget<S, E>>::Children:
            for<'a> TraversableSeq<InternalEventVisitor<'a>, S, E, EffectContext<S>>,
    {
        let mut visitor = InternalEventVisitor::new(event.payload());
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
