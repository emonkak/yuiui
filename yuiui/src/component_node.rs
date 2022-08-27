use std::marker::PhantomData;
use std::mem;

use crate::component::{Component, ComponentLifecycle};
use crate::effect::EffectContext;
use crate::element::Element;
use crate::render::{ComponentIndex, RenderContext};
use crate::state::State;
use crate::view::View;
use crate::widget_node::{CommitMode, WidgetNodeScope};

#[derive(Debug)]
pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) state: PhantomData<S>,
    pub(crate) env: PhantomData<E>,
}

impl<C, S, E> ComponentNode<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: PhantomData,
            env: PhantomData,
        }
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let lifecycle = match mode {
            CommitMode::Mount => ComponentLifecycle::Mounted,
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("take pending component"),
                );
                ComponentLifecycle::Updated(old_component)
            }
            CommitMode::Unmount => ComponentLifecycle::Unmounted,
        };
        let result = self.component.lifecycle(lifecycle, state, env);
        context.process_result(result);
        context.next_component();
    }
}

pub trait ComponentStack<S: State, E>: Sized {
    type View: View<S, E>;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>);

    fn force_update<'a>(
        scope: WidgetNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

impl<C, CS, S, E> ComponentStack<S, E> for (ComponentNode<C, S, E>, CS)
where
    C: Component<S, E>,
    C::Element: Element<S, E, Components = CS>,
    CS: ComponentStack<S, E, View = <C::Element as Element<S, E>>::View>,
    S: State,
{
    type View = <C::Element as Element<S, E>>::View;

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        self.0.commit(mode, state, env, context);
        self.1.commit(mode, state, env, context);
    }

    fn force_update<'a>(
        scope: WidgetNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let (head, tail) = scope.components;
        let scope = WidgetNodeScope {
            id: scope.id,
            state: scope.state,
            children: scope.children,
            components: tail,
            dirty: scope.dirty,
        };
        if target_index == current_index {
            let element = head.component.render(state, env);
            element.update(scope, state, env, context)
        } else {
            CS::force_update(scope, target_index, current_index + 1, state, env, context)
        }
    }
}

#[derive(Debug)]
pub struct ComponentEnd<V>(PhantomData<V>);

impl<V> ComponentEnd<V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<V: View<S, E>, S: State, E> ComponentStack<S, E> for ComponentEnd<V> {
    type View = V;

    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EffectContext<S>) {
    }

    fn force_update<'a>(
        _scope: WidgetNodeScope<'a, V, Self, S, E>,
        _target_index: ComponentIndex,
        _current_index: ComponentIndex,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) -> bool {
        true
    }
}
