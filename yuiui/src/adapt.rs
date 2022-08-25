use std::fmt;
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::sync::Arc;

use crate::component::{Component, ComponentLifecycle, ComponentStack};
use crate::element::Element;
use crate::event::{CaptureState, EventContext, EventMask, EventResult, InternalEvent};
use crate::id::{IdContext, IdPath};
use crate::sequence::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::{
    Widget, WidgetEvent, WidgetLifeCycle, WidgetNode, WidgetNodeScope, WidgetState,
};

pub struct Adapt<T, F, S, SS> {
    target: T,
    selector_fn: Arc<F>,
    state: PhantomData<S>,
    sub_state: PhantomData<SS>,
}

impl<T, F, S, SS> Adapt<T, F, S, SS> {
    pub fn new(target: T, selector_fn: Arc<F>) -> Self {
        Self {
            target,
            selector_fn,
            state: PhantomData,
            sub_state: PhantomData,
        }
    }
}

impl<T, F, S, SS> fmt::Debug for Adapt<T, F, S, SS>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.target).finish()
    }
}

impl<T, F, S, SS, E> Element<S, E> for Adapt<T, F, S, SS>
where
    T: Element<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type View = Adapt<T::View, F, S, SS>;

    type Components = Adapt<T::Components, F, S, SS>;

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
            WidgetState::Changed(widget, view, old_view) => {
                WidgetState::Changed(widget.target, view.target, old_view.target)
            }
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

impl<T, F, S, SS, E> ElementSeq<S, E> for Adapt<T, F, S, SS>
where
    T: ElementSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Store = Adapt<T::Store, F, S, SS>;

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

impl<T, F, S, SS, E> WidgetNodeSeq<S, E> for Adapt<T, F, S, SS>
where
    T: WidgetNodeSeq<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EventContext<S>) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        context.merge(sub_context, |sub_message| {
            sub_message.lift(self.selector_fn.clone())
        });
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &E,
        context: &mut EventContext<S>,
    ) -> CaptureState {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let capture_state = self.target.event(event, sub_state, env, &mut sub_context);
        context.merge(sub_context, |sub_message| {
            sub_message.lift(self.selector_fn.clone())
        });
        capture_state
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &E,
        context: &mut EventContext<S>,
    ) -> CaptureState {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let capture_state = self
            .target
            .internal_event(event, sub_state, env, &mut sub_context);
        context.merge(sub_context, |sub_message| {
            sub_message.lift(self.selector_fn.clone())
        });
        capture_state
    }
}

impl<T, F, S, SS, C> TraversableSeq<C> for &Adapt<T, F, S, SS>
where
    for<'a> &'a T: TraversableSeq<C>,
{
    fn for_each(self, callback: &mut C) -> ControlFlow<()> {
        self.target.for_each(callback)
    }
}

impl<T, F, S, SS, E> Component<S, E> for Adapt<T, F, S, SS>
where
    T: Component<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Element = Adapt<T::Element, F, S, SS>;

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

impl<T, F, S, SS, E> ComponentStack<S, E> for Adapt<T, F, S, SS>
where
    T: ComponentStack<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EventContext<S>) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        context.merge(sub_context, |sub_message| {
            sub_message.lift(self.selector_fn.clone())
        });
    }
}

impl<T, F, S, SS, E> View<S, E> for Adapt<T, F, S, SS>
where
    T: View<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Widget = Adapt<T::Widget, F, S, SS>;

    type Children = Adapt<T::Children, F, S, SS>;

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
        old_view: &Self,
        widget: &mut Self::Widget,
        state: &S,
        env: &E,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target.rebuild(
            &children.target,
            &old_view.target,
            &mut widget.target,
            sub_state,
            env,
        )
    }
}

impl<T, F, S, SS, E> Widget<S, E> for Adapt<T, F, S, SS>
where
    T: Widget<SS, E>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    S: State + 'static,
    SS: State + 'static,
{
    type Children = Adapt<T::Children, F, S, SS>;

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

impl<'event, T, F, S, SS> WidgetEvent<'event> for Adapt<T, F, S, SS>
where
    T: WidgetEvent<'event>,
{
    type Event = T::Event;
}

fn lift_widget_state<V, F, S, SS, E>(
    state: WidgetState<V, V::Widget>,
    f: &Arc<F>,
) -> WidgetState<Adapt<V, F, S, SS>, Adapt<V::Widget, F, S, SS>>
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
        WidgetState::Changed(widget, view, old_view) => WidgetState::Changed(
            Adapt::new(widget, f.clone()),
            Adapt::new(view, f.clone()),
            Adapt::new(old_view, f.clone()),
        ),
    }
}
