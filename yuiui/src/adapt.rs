use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component::{Component, ComponentLifecycle};
use crate::component_node::ComponentStack;
use crate::effect::EffectContext;
use crate::element::Element;
use crate::event::{EventMask, EventResult};
use crate::id::{ComponentIndex, IdContext, IdPath};
use crate::sequence::{ElementSeq, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetEvent, WidgetLifeCycle};
use crate::widget_node::{CommitMode, WidgetNode, WidgetNodeScope, WidgetNodeVisitor, WidgetState};

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
    S: State + 'static,
    SS: State + 'static,
{
    type View = Adapt<T::View, F, SS>;

    type Components = Adapt<T::Components, F, SS>;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let sub_state = (self.selector_fn)(state);
        let sub_node = self.target.render(sub_state, env, context);
        WidgetNode {
            id: sub_node.id,
            state: sub_node
                .state
                .map(|state| lift_widget_state(state, &self.selector_fn)),
            children: Adapt::new(sub_node.children, self.selector_fn.clone()),
            components: Adapt::new(sub_node.components, self.selector_fn),
            event_mask: sub_node.event_mask,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) -> bool {
        let mut sub_widget_state = scope.state.take().map(|state| match state {
            WidgetState::Uninitialized(view) => WidgetState::Uninitialized(view.target),
            WidgetState::Prepared(widget, view) => {
                WidgetState::Prepared(widget.target, view.target)
            }
            WidgetState::Dirty(widget, view) => WidgetState::Dirty(widget.target, view.target),
        });
        let sub_scope = WidgetNodeScope {
            id: scope.id,
            state: &mut sub_widget_state,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
        };
        let sub_state = (self.selector_fn)(state);
        let has_changed = self.target.update(sub_scope, sub_state, env, context);
        *scope.state = sub_widget_state.map(|state| lift_widget_state(state, &self.selector_fn));
        has_changed
    }
}

impl<T, F, SS, S, E> ElementSeq<S, E> for Adapt<T, F, SS>
where
    T: ElementSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Store = Adapt<T::Store, F, SS>;

    fn render(self, state: &S, env: &E, context: &mut IdContext) -> Self::Store {
        let sub_state = (self.selector_fn)(state);
        Adapt::new(
            self.target.render(sub_state, env, context),
            self.selector_fn.clone(),
        )
    }

    fn update(self, store: &mut Self::Store, state: &S, env: &E, context: &mut IdContext) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .update(&mut store.target, sub_state, env, context)
    }
}

impl<T, F, SS, S, E> WidgetNodeSeq<S, E> for Adapt<T, F, SS>
where
    T: WidgetNodeSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        context.merge(sub_context, &self.selector_fn);
    }

    fn for_each<V: WidgetNodeVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .for_each(visitor, sub_state, env, &mut sub_context);
        context.merge(sub_context, &self.selector_fn);
    }

    fn search<V: WidgetNodeVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let found = self
            .target
            .search(id_path, visitor, sub_state, env, &mut sub_context);
        context.merge(sub_context, &self.selector_fn);
        found
    }
}

impl<T, F, SS, S, E> Component<S, E> for Adapt<T, F, SS>
where
    T: Component<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Element = Adapt<T::Element, F, SS>;

    fn lifecycle(&self, lifecycle: ComponentLifecycle<Self>, state: &S, env: &E) -> EventResult<S> {
        let sub_lifecycle = lifecycle.map_component(|component| component.target);
        let sub_state = (self.selector_fn)(state);
        self.target
            .lifecycle(sub_lifecycle, sub_state, env)
            .lift(self.selector_fn.clone())
    }

    fn render(&self, state: &S, env: &E) -> Self::Element {
        Adapt::new(
            self.target.render((self.selector_fn)(state), env),
            self.selector_fn.clone(),
        )
    }

    fn should_update(&self, other: &Self, state: &S, env: &E) -> bool {
        self.target
            .should_update(&other.target, (self.selector_fn)(state), env)
    }
}

impl<T, F, SS, S, E> ComponentStack<S, E> for Adapt<T, F, SS>
where
    T: ComponentStack<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type View = Adapt<T::View, F, SS>;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        context.merge(sub_context, &self.selector_fn);
    }

    fn force_update<'a>(
        scope: WidgetNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut IdContext,
    ) {
        let mut sub_widget_state = scope.state.take().map(|state| match state {
            WidgetState::Uninitialized(view) => WidgetState::Uninitialized(view.target),
            WidgetState::Prepared(widget, view) => {
                WidgetState::Prepared(widget.target, view.target)
            }
            WidgetState::Dirty(widget, view) => WidgetState::Dirty(widget.target, view.target),
        });
        let selector_fn = &scope.components.selector_fn;
        let sub_scope = WidgetNodeScope {
            id: scope.id,
            state: &mut sub_widget_state,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
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
        *scope.state = sub_widget_state.map(|state| lift_widget_state(state, selector_fn));
        has_changed
    }
}

impl<T, F, SS, S, E> View<S, E> for Adapt<T, F, SS>
where
    T: View<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Widget = Adapt<T::Widget, F, SS>;

    type Children = Adapt<T::Children, F, SS>;

    fn build(
        &self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        state: &S,
        env: &E,
    ) -> Self::Widget {
        let sub_state = (self.selector_fn)(state);
        Adapt::new(
            self.target.build(&children.target, sub_state, env),
            self.selector_fn.clone(),
        )
    }

    fn rebuild(
        &self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        widget: &mut Self::Widget,
        state: &S,
        env: &E,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .rebuild(&children.target, &mut widget.target, sub_state, env)
    }
}

impl<T, F, SS, S, E> Widget<S, E> for Adapt<T, F, SS>
where
    T: Widget<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Children = Adapt<T::Children, F, SS>;

    fn lifecycle(
        &mut self,
        lifecycle: WidgetLifeCycle,
        children: &Self::Children,
        id_path: &IdPath,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        let sub_state = (self.selector_fn)(state);
        self.target
            .lifecycle(lifecycle, &children.target, id_path, sub_state, env)
            .lift(self.selector_fn.clone())
    }

    fn event(
        &mut self,
        event: <Self as WidgetEvent>::Event,
        children: &Self::Children,
        id_path: &IdPath,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        let sub_state = (self.selector_fn)(state);
        self.target
            .event(event, &children.target, id_path, sub_state, env)
            .lift(self.selector_fn.clone())
    }
}

impl<'event, T, F, SS> WidgetEvent<'event> for Adapt<T, F, SS>
where
    T: WidgetEvent<'event>,
{
    type Event = T::Event;
}

fn lift_widget_state<V, F, SS, S, E>(
    state: WidgetState<V, V::Widget>,
    f: &Arc<F>,
) -> WidgetState<Adapt<V, F, SS>, Adapt<V::Widget, F, SS>>
where
    V: View<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State,
    SS: State,
{
    match state {
        WidgetState::Uninitialized(view) => WidgetState::Uninitialized(Adapt::new(view, f.clone())),
        WidgetState::Prepared(widget, view) => {
            WidgetState::Prepared(Adapt::new(widget, f.clone()), Adapt::new(view, f.clone()))
        }
        WidgetState::Dirty(widget, view) => {
            WidgetState::Dirty(Adapt::new(widget, f.clone()), Adapt::new(view, f.clone()))
        }
    }
}
