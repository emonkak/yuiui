use hlist::{HCons, HList, HNil};
use std::mem;
use std::ops::ControlFlow;
use std::rc::Rc;

use crate::component::{Component, ComponentStack};
use crate::context::{EffectContext, RenderContext};
use crate::element::Element;
use crate::event::{EventMask, EventResult, InternalEvent};
use crate::sequence::{CommitMode, ElementSeq, TraversableSeq, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;
use crate::widget::{Widget, WidgetNode, WidgetNodeScope, WidgetState};

pub trait Env<'a>: HList {
    type Output: HList + Clone;

    fn as_refs(&'a self) -> Self::Output;
}

impl<'a, H, T> Env<'a> for HCons<H, T>
where
    H: 'a,
    T: Env<'a>,
{
    type Output = HCons<&'a H, <T as Env<'a>>::Output>;

    fn as_refs(&'a self) -> Self::Output {
        HCons {
            head: &self.head,
            tail: self.tail.as_refs(),
        }
    }
}

impl<'a> Env<'a> for HNil {
    type Output = HNil;

    fn as_refs(&'a self) -> Self::Output {
        HNil
    }
}

#[derive(Debug)]
pub struct WithEnv<T, E> {
    target: T,
    env: Rc<E>,
}

impl<T, SE> WithEnv<T, SE> {
    pub fn new(target: T, env: Rc<SE>) -> Self {
        Self { target, env }
    }
}

impl<T, E, S, ES> Element<S, ES> for WithEnv<T, E>
where
    T: Element<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    type View = WithEnv<T::View, E>;

    type Components = WithEnv<T::Components, E>;

    fn render(
        self,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut RenderContext,
    ) -> WidgetNode<Self::View, Self::Components, S, ES> {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        let sub_node = self.target.render(state, sub_env_ref, context);
        WidgetNode {
            id: sub_node.id,
            state: sub_node.state.map(|state| match state {
                WidgetState::Uninitialized(view) => {
                    WidgetState::Uninitialized(WithEnv::new(view, self.env.clone()))
                }
                WidgetState::Prepared(widget) => {
                    WidgetState::Prepared(WithEnv::new(widget, self.env.clone()))
                }
                WidgetState::Changed(widget, view) => WidgetState::Changed(
                    WithEnv::new(widget, self.env.clone()),
                    WithEnv::new(view, self.env.clone()),
                ),
            }),
            children: WithEnv::new(sub_node.children, self.env.clone()),
            components: WithEnv::new(sub_node.components, self.env),
            event_mask: sub_node.event_mask,
        }
    }

    fn update(
        self,
        scope: WidgetNodeScope<Self::View, Self::Components, S, ES>,
        state: &S,
        env: &<ES as Env>::Output,
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
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        let has_changed = self.target.update(sub_scope, state, sub_env_ref, context);
        *scope.state = sub_widget_state.map(|state| match state {
            WidgetState::Uninitialized(view) => {
                WidgetState::Uninitialized(WithEnv::new(view, self.env))
            }
            WidgetState::Prepared(widget) => {
                WidgetState::Prepared(WithEnv::new(widget, self.env))
            }
            WidgetState::Changed(widget, view) => WidgetState::Changed(
                WithEnv::new(widget, self.env.clone()),
                WithEnv::new(view, self.env),
            ),
        });
        has_changed
    }
}

impl<T, E, S, ES> ElementSeq<S, ES> for WithEnv<T, E>
where
    T: ElementSeq<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    type Store = WithEnv<T::Store, E>;

    fn render(
        self,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut RenderContext,
    ) -> Self::Store {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        WithEnv::new(
            self.target.render(state, sub_env_ref, context),
            self.env,
        )
    }

    fn update(
        self,
        store: &mut Self::Store,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut RenderContext,
    ) -> bool {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target
            .update(&mut store.target, state, sub_env_ref, context)
    }
}

impl<T, E, S, ES> WidgetNodeSeq<S, ES> for WithEnv<T, E>
where
    T: WidgetNodeSeq<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    fn event_mask() -> EventMask {
        T::event_mask()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut EffectContext<S>,
    ) {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target.commit(mode, state, sub_env_ref, context);
    }

    fn event<Event: 'static>(
        &mut self,
        event: &Event,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target.event(event, state, sub_env_ref, context)
    }

    fn internal_event(
        &mut self,
        event: &InternalEvent,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target
            .internal_event(event, state, sub_env_ref, context)
    }
}

impl<T, E, C> TraversableSeq<C> for WithEnv<T, E>
where
    T: TraversableSeq<C>,
{
    fn for_each(&self, callback: &mut C) -> ControlFlow<()> {
        self.target.for_each(callback)
    }
}

impl<T, E, S, ES> Component<S, ES> for WithEnv<T, E>
where
    T: Component<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    type Element = WithEnv<T::Element, E>;

    fn render(&self, state: &S, env: &<ES as Env>::Output) -> Self::Element {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        WithEnv::new(self.target.render(state, sub_env_ref), self.env.clone())
    }

    fn should_update(&self, other: &Self, state: &S, env: &<ES as Env>::Output) -> bool {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target.should_update(&other.target, state, sub_env_ref)
    }
}

impl<T, E, S, ES> ComponentStack<S, ES> for WithEnv<T, E>
where
    T: ComponentStack<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut EffectContext<S>,
    ) {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target.commit(mode, state, sub_env_ref, context);
    }
}

impl<T, E, S, ES> View<S, ES> for WithEnv<T, E>
where
    T: View<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    type Widget = WithEnv<T::Widget, E>;

    type Children = WithEnv<T::Children, E>;

    fn build(
        self,
        children: &<Self::Widget as Widget<S, ES>>::Children,
        state: &S,
        env: &<ES as Env>::Output,
    ) -> Self::Widget {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        WithEnv::new(
            self.target.build(&children.target, state, sub_env_ref),
            self.env,
        )
    }

    fn rebuild(
        self,
        children: &<Self::Widget as Widget<S, ES>>::Children,
        widget: &mut Self::Widget,
        state: &S,
        env: &<ES as Env>::Output,
    ) -> bool {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target
            .rebuild(&children.target, &mut widget.target, state, sub_env_ref)
    }
}

impl<T, E, S, ES> Widget<S, ES> for WithEnv<T, E>
where
    T: Widget<S, HCons<E, ES>>,
    E: for<'a> Env<'a> + 'static,
    S: State,
    ES: for<'a> Env<'a>,
{
    type Children = WithEnv<T::Children, E>;

    type Event = T::Event;

    fn event(
        &mut self,
        event: &Self::Event,
        children: &Self::Children,
        state: &S,
        env: &<ES as Env>::Output,
        context: &mut EffectContext<S>,
    ) -> EventResult {
        let sub_env = env.clone().cons(self.env.as_ref());
        let sub_env_ref = unsafe { mem::transmute(&sub_env) };
        self.target
            .event(event, &children.target, state, sub_env_ref, context)
    }
}
