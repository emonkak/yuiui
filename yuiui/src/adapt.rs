use std::fmt;
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::rc::Rc;

use crate::component::{Component, ComponentStack};
use crate::context::{EffectContext, RenderContext};
use crate::effect::{Effect, Mutation};
use crate::element::Element;
use crate::env::Env;
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::sequence::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode, WidgetNodeScope, WidgetState};

pub struct Adapt<T, F, SS> {
    target: T,
    selector_fn: Rc<F>,
    sub_state: PhantomData<SS>,
}

impl<T, F, SS> Adapt<T, F, SS> {
    pub fn new(target: T, selector_fn: Rc<F>) -> Self {
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
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    type View = Adapt<T::View, F, SS>;

    type Components = Adapt<T::Components, F, SS>;

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, E> {
        let sub_state = (self.selector_fn)(state);
        let sub_node = self.target.render(sub_state, env, context);
        WidgetNode {
            id: sub_node.id,
            state: sub_node.state.map(|state| match state {
                WidgetState::Uninitialized(view) => {
                    WidgetState::Uninitialized(Adapt::new(view, self.selector_fn.clone()))
                }
                WidgetState::Prepared(widget) => {
                    WidgetState::Prepared(Adapt::new(widget, self.selector_fn.clone()))
                }
                WidgetState::Changed(widget, view) => WidgetState::Changed(
                    Adapt::new(widget, self.selector_fn.clone()),
                    Adapt::new(view, self.selector_fn.clone()),
                ),
            }),
            children: Adapt::new(sub_node.children, self.selector_fn.clone()),
            components: Adapt::new(sub_node.components, self.selector_fn),
            event_mask: sub_node.event_mask,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let mut sub_widget_state = scope.state.take().map(|state| match state {
            WidgetState::Uninitialized(view) => WidgetState::Uninitialized(view.target),
            WidgetState::Prepared(widget) => WidgetState::Prepared(widget.target),
            WidgetState::Changed(widget, view) => WidgetState::Changed(widget.target, view.target),
        });
        let sub_scope = WidgetNodeScope {
            id: scope.id,
            state: &mut sub_widget_state,
            children: &mut scope.children.target,
            components: &mut scope.components.target,
        };
        let sub_state = (self.selector_fn)(state);
        let has_changed = self.target.update(sub_scope, sub_state, env, context);
        *scope.state = sub_widget_state.map(|state| match state {
            WidgetState::Uninitialized(view) => {
                WidgetState::Uninitialized(Adapt::new(view, self.selector_fn.clone()))
            }
            WidgetState::Prepared(widget) => {
                WidgetState::Prepared(Adapt::new(widget, self.selector_fn.clone()))
            }
            WidgetState::Changed(widget, view) => WidgetState::Changed(
                Adapt::new(widget, self.selector_fn.clone()),
                Adapt::new(view, self.selector_fn.clone()),
            ),
        });
        has_changed
    }
}

impl<T, F, SS, S, E> ElementSeq<S, E> for Adapt<T, F, SS>
where
    T: ElementSeq<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    type Store = Adapt<T::Store, F, SS>;

    fn render(
        self,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> Self::Store {
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
        env: &<E as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .update(&mut store.target, sub_state, env, context)
    }
}

impl<T, F, SS, S, E> WidgetNodeSeq<S, E> for Adapt<T, F, SS>
where
    T: WidgetNodeSeq<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        for (id, component_index, sub_effect) in sub_context.effects {
            let effect = map_effect(sub_effect, self.selector_fn.clone());
            context.effects.push((id, component_index, effect));
        }
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let result = self.target.event(event, sub_state, env, &mut sub_context);
        for (id, component_index, sub_effect) in sub_context.effects {
            let effect = map_effect(sub_effect, self.selector_fn.clone());
            context.effects.push((id, component_index, effect));
        }
        result
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let result = self
            .target
            .internal_event(event, sub_state, env, &mut sub_context);
        for (id, component_index, sub_effect) in sub_context.effects {
            let effect = map_effect(sub_effect, self.selector_fn.clone());
            context.effects.push((id, component_index, effect));
        }
        result
    }
}

impl<T, F, SS, C> TraversableSeq<C> for Adapt<T, F, SS>
where
    T: TraversableSeq<C>,
{
    fn for_each(&self, callback: &mut C) -> ControlFlow<()> {
        self.target.for_each(callback)
    }
}

impl<T, F, SS, S, E> Component<S, E> for Adapt<T, F, SS>
where
    T: Component<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    type Element = Adapt<T::Element, F, SS>;

    fn render(&self, state: &S, env: &<E as Env>::Output) -> Self::Element {
        Adapt::new(
            self.target.render((self.selector_fn)(state), env),
            self.selector_fn.clone(),
        )
    }

    fn should_update(&self, other: &Self, state: &S, env: &<E as Env>::Output) -> bool {
        self.target
            .should_update(&other.target, (self.selector_fn)(state), env)
    }
}

impl<T, F, SS, S, E> ComponentStack<S, E> for Adapt<T, F, SS>
where
    T: ComponentStack<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(mode, sub_state, env, &mut sub_context);
        for (id, component_index, sub_effect) in sub_context.effects {
            let effect = map_effect(sub_effect, self.selector_fn.clone());
            context.effects.push((id, component_index, effect));
        }
    }
}

impl<T, F, SS, S, E> View<S, E> for Adapt<T, F, SS>
where
    T: View<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    type Widget = Adapt<T::Widget, F, SS>;

    type Children = Adapt<T::Children, F, SS>;

    fn build(
        self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        state: &S,
        env: &<E as Env>::Output,
    ) -> Self::Widget {
        let sub_state = (self.selector_fn)(state);
        Adapt::new(
            self.target.build(&children.target, sub_state, env),
            self.selector_fn.clone(),
        )
    }

    fn rebuild(
        self,
        children: &<Self::Widget as Widget<S, E>>::Children,
        widget: &mut Self::Widget,
        state: &S,
        env: &<E as Env>::Output,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .rebuild(&children.target, &mut widget.target, sub_state, env)
    }
}

impl<T, F, SS, S, E> Widget<S, E> for Adapt<T, F, SS>
where
    T: Widget<SS, E>,
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
    E: for<'a> Env<'a>,
{
    type Children = Adapt<T::Children, F, SS>;

    type Event = T::Event;

    fn event(
        &mut self,
        event: &Self::Event,
        children: &Self::Children,
        state: &S,
        env: &<E as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let result = self
            .target
            .event(event, &children.target, sub_state, env, &mut sub_context);
        for (id, component_index, sub_effect) in sub_context.effects {
            let effect = map_effect(sub_effect, self.selector_fn.clone());
            context.effects.push((id, component_index, effect));
        }
        result
    }
}

impl<T, F, SS, S> Mutation<S> for Adapt<T, F, SS>
where
    T: Mutation<SS>,
    F: Fn(&S) -> &SS,
{
    fn apply(&mut self, state: &mut S) -> bool {
        let sub_state = unsafe { &mut *((self.selector_fn)(state) as *const _ as *mut _) };
        self.target.apply(sub_state)
    }
}

fn map_effect<F, SS, S>(effect: Effect<SS>, f: Rc<F>) -> Effect<S>
where
    F: Fn(&S) -> &SS + 'static,
    SS: State + 'static,
    S: State,
{
    match effect {
        Effect::Message(message) => Effect::Mutation(Box::new(Adapt::new(Some(message), f))),
        Effect::Mutation(mutation) => Effect::Mutation(Box::new(Adapt::new(mutation, f))),
    }
}
