use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{CommitContext, RenderContext};
use crate::element::Element;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeScope};

pub trait ComponentStack<S: State, E>: Sized {
    const LEN: usize;

    type View: View<S, E>;

    fn update<'a>(
        scope: ViewNodeScope<'a, Self::View, Self, S, E>,
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
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut CommitContext<S>,
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

    fn update<'a>(
        scope: ViewNodeScope<'a, Self::View, Self, S, E>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let (head, tail) = scope.components;
        let scope = ViewNodeScope {
            id: scope.id,
            state: scope.state,
            children: scope.children,
            components: tail,
            dirty: scope.dirty,
        };
        if target_index <= current_index {
            let element = head.render();
            element.update(scope, state, env, context)
        } else {
            CS::update(scope, target_index, current_index + 1, state, env, context)
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        env: &E,
        context: &mut CommitContext<S>,
    ) {
        if target_index <= current_index {
            self.0.commit(mode, target_index, state, env, context);
        } else {
            self.1
                .commit(mode, target_index, current_index + 1, state, env, context);
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
    const LEN: usize = 0;

    type View = V;

    fn update<'a>(
        _scope: ViewNodeScope<'a, V, Self, S, E>,
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
        _target_index: ComponentIndex,
        _current_index: ComponentIndex,
        _state: &S,
        _env: &E,
        _context: &mut CommitContext<S>,
    ) {
    }
}
