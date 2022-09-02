use std::fmt;
use std::marker::PhantomData;
use std::mem;

use crate::component::Component;
use crate::context::{EffectContext, RenderContext};
use crate::element::Element;
use crate::event::Lifecycle;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::view::View;
use crate::widget_node::{CommitMode, WidgetNodeScope};

pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) local_state: C::LocalState,
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
            local_state: Default::default(),
            state: PhantomData,
            env: PhantomData,
        }
    }

    pub(crate) fn render(&self, state: &S, env: &E) -> C::Element {
        self.component.render(&self.local_state, state, env)
    }

    pub(crate) fn should_update(&self, other: &C, state: &S, env: &E) -> bool {
        self.component
            .should_update(other, &self.local_state, state, env)
    }

    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EffectContext<S>) {
        let result = match mode {
            CommitMode::Mount => {
                self.component
                    .lifecycle(Lifecycle::Mounted, &mut self.local_state, state, env)
            }
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("take pending component"),
                );
                self.component.lifecycle(
                    Lifecycle::Updated(&old_component),
                    &mut self.local_state,
                    state,
                    env,
                )
            }
            CommitMode::Unmount => {
                self.component
                    .lifecycle(Lifecycle::Unmounted, &mut self.local_state, state, env)
            }
        };
        context.process_result(result);
    }
}

impl<C, S, E> fmt::Debug for ComponentNode<C, S, E>
where
    C: Component<S, E> + fmt::Debug,
    C::LocalState: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .field("local_state", &self.local_state)
            .finish()
    }
}

pub trait ComponentStack<S: State, E>: Sized {
    const LEN: usize;

    type View: View<S, E>;

    fn force_update<'a>(
        scope: WidgetNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    );
}

impl<C, CS, S, E> ComponentStack<S, E> for (ComponentNode<C, S, E>, CS)
where
    C: Component<S, E>,
    C::Element: Element<S, E, Components = CS>,
    CS: ComponentStack<S, E, View = <C::Element as Element<S, E>>::View>,
    S: State,
{
    const LEN: usize = 1 + CS::LEN;

    type View = <C::Element as Element<S, E>>::View;

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
        if target_index <= current_index {
            let element = head.render(state, env);
            element.update(scope, state, env, context)
        } else {
            CS::force_update(scope, target_index, current_index + 1, state, env, context)
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut EffectContext<S>,
    ) {
        if target_index <= context.effect_path().component_index {
            self.0.commit(mode, state, env, context);
        }
        context.increment_component_index();
        self.1.commit(mode, target_index, state, env, context);
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
    const LEN: usize = 0;

    type View = V;

    fn force_update<'a>(
        _scope: WidgetNodeScope<'a, V, Self, S, E>,
        _target_index: ComponentIndex,
        _current_index: ComponentIndex,
        _state: &S,
        _env: &E,
        _context: &mut RenderContext,
    ) -> bool {
        false
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _component_index: ComponentIndex,
        _state: &S,
        _env: &E,
        _context: &mut EffectContext<S>,
    ) {
    }
}
