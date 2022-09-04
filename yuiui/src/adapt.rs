use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component_node::ComponentStack;
use crate::context::{EffectContext, RenderContext};
use crate::effect::EffectPath;
use crate::element::{Element, ElementSeq};
use crate::event::{EventMask, EventResult, HasEvent, Lifecycle};
use crate::id::{ComponentIndex, IdPath};
use crate::state::State;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeScope, ViewNodeSeq};

pub struct Adapt<T, F, SS> {
    target: T,
    selector_fn: Arc<F>,
    sub_state: PhantomData<SS>,
}

impl<T, F, SS> Adapt<T, F, SS> {
    pub fn new(target: T, selector_fn: Arc<F>) -> Self {
        Self {
            target,
            selector_fn,
            sub_state: PhantomData,
        }
    }
}

impl<T, F, SS> fmt::Debug for Adapt<T, F, SS>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.target).finish()
    }
}

impl<T, F, SS, S, E> Element<S, E> for Adapt<T, F, SS>
where
    T: Element<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type View = Adapt<T::View, F, SS>;

    type Components = Adapt<T::Components, F, SS>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let sub_state = (self.selector_fn)(state);
        let sub_node = self.target.render(sub_state, env, context);
        ViewNode {
            id: sub_node.id,
            state: sub_node
                .state
                .map(|state| state.map_view(|view| Adapt::new(view, self.selector_fn.clone()))),
            children: Adapt::new(sub_node.children, self.selector_fn.clone()),
            components: Adapt::new(sub_node.components, self.selector_fn),
            event_mask: sub_node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut sub_view_node_state = scope
            .state
            .take()
            .map(|state| state.map_view(|view| view.target));
        let sub_scope = ViewNodeScope {
            id: scope.id,
            state: &mut sub_view_node_state,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
            dirty: scope.dirty,
        };
        let sub_state = (self.selector_fn)(state);
        let has_changed = self.target.update(sub_scope, sub_state, env, context);
        *scope.state = sub_view_node_state
            .map(|state| state.map_view(|view| Adapt::new(view, self.selector_fn.clone())));
        has_changed
    }
}

impl<T, F, SS, S, E> ElementSeq<S, E> for Adapt<T, F, SS>
where
    T: ElementSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Store = Adapt<T::Store, F, SS>;

    fn render(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Store {
        let sub_state = (self.selector_fn)(state);
        Adapt::new(
            self.target.render(sub_state, env, context),
            self.selector_fn.clone(),
        )
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .update(&mut store.target, sub_state, env, context)
    }
}

impl<T, F, SS, S, E> ViewNodeSeq<S, E> for Adapt<T, F, SS>
where
    T: ViewNodeSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        self.target.len()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
    }
}

impl<T, F, SS, Visitor, S, E> Traversable<Visitor, RenderContext, S, E> for Adapt<T, F, SS>
where
    T: Traversable<Visitor, RenderContext, SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
{
    fn for_each(&mut self, visitor: &mut Visitor, state: &S, env: &E, context: &mut RenderContext) {
        let sub_state = (self.selector_fn)(state);
        self.target.for_each(visitor, sub_state, env, context);
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .search(id_path, visitor, sub_state, env, context)
    }
}

impl<T, F, SS, Visitor, S, E> Traversable<Visitor, EffectContext<S>, S, E> for Adapt<T, F, SS>
where
    T: Traversable<Visitor, EffectContext<SS>, SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .for_each(visitor, sub_state, env, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let found = self
            .target
            .search(id_path, visitor, sub_state, env, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
        found
    }
}

impl<T, F, SS, S, E> ComponentStack<S, E> for Adapt<T, F, SS>
where
    T: ComponentStack<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    const LEN: usize = T::LEN;

    type View = Adapt<T::View, F, SS>;

    fn commit(
        &mut self,
        mode: CommitMode,
        component_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .commit(mode, component_index, sub_state, env, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
    }

    fn force_update<'a>(
        scope: ViewNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let mut sub_view_node_state = scope
            .state
            .take()
            .map(|state| state.map_view(|view| view.target));
        let selector_fn = &scope.components.selector_fn;
        let sub_scope = ViewNodeScope {
            id: scope.id,
            state: &mut sub_view_node_state,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
            dirty: scope.dirty,
        };
        let sub_state = selector_fn(state);
        let has_changed = T::force_update(
            sub_scope,
            target_index,
            current_index,
            sub_state,
            env,
            context,
        );
        *scope.state = sub_view_node_state
            .map(|state| state.map_view(|view| Adapt::new(view, selector_fn.clone())));
        has_changed
    }
}

impl<T, F, SS, S, E> View<S, E> for Adapt<T, F, SS>
where
    T: View<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Widget = T::Widget;

    type Children = Adapt<T::Children, F, SS>;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        widget: &mut Self::Widget,
        children: &<Self::Children as ElementSeq<S, E>>::Store,
        effect_path: &EffectPath,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        let sub_lifecycle = lifecycle.map(|view| &view.target);
        let sub_state = (self.selector_fn)(state);
        self.target
            .lifecycle(
                sub_lifecycle,
                widget,
                &children.target,
                effect_path,
                sub_state,
                env,
            )
            .lift(&self.selector_fn)
    }

    fn event(
        &self,
        event: <Self as HasEvent>::Event,
        widget: &mut Self::Widget,
        children: &<Self::Children as ElementSeq<S, E>>::Store,
        effect_path: &EffectPath,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        let sub_state = (self.selector_fn)(state);
        self.target
            .event(event, widget, &children.target, effect_path, sub_state, env)
            .lift(&self.selector_fn)
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, E>>::Store,
        state: &S,
        env: &E,
    ) -> Self::Widget {
        let sub_state = (self.selector_fn)(state);
        self.target.build(&children.target, sub_state, env)
    }
}

impl<'event, T, F, SS> HasEvent<'event> for Adapt<T, F, SS>
where
    T: HasEvent<'event>,
{
    type Event = T::Event;
}
