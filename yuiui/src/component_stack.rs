use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{CommitContext, RenderContext};
use crate::element::Element;
use crate::id::ComponentIndex;
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S: State, B>: Sized {
    const LEN: usize;

    type View: View<S, B>;

    fn update<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, B>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    );
}

impl<C, CS, S, B> ComponentStack<S, B> for (ComponentNode<C, S, B>, CS)
where
    C: Component<S, B>,
    C::Element: Element<S, B, Components = CS>,
    CS: ComponentStack<S, B, View = <C::Element as Element<S, B>>::View>,
    S: State,
{
    const LEN: usize = 1 + CS::LEN;

    type View = <C::Element as Element<S, B>>::View;

    fn update<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, B>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let (head, tail) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            state: node.state,
            children: node.children,
            components: tail,
            env: node.env,
            dirty: node.dirty,
        };
        if target_index <= current_index {
            let element = head.render(state, backend);
            element.update(&mut node, state, backend, context)
        } else {
            CS::update(
                node,
                target_index,
                current_index + 1,
                state,
                backend,
                context,
            )
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) {
        if target_index <= current_index {
            self.0.commit(mode, current_index, state, backend, context);
        } else {
            self.1.commit(
                mode,
                target_index,
                current_index + 1,
                state,
                backend,
                context,
            );
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

impl<V: View<S, B>, S: State, B> ComponentStack<S, B> for ComponentEnd<V> {
    const LEN: usize = 0;

    type View = V;

    fn update<'a>(
        _node: ViewNodeMut<'a, V, Self, S, B>,
        _target_index: ComponentIndex,
        _current_index: ComponentIndex,
        _state: &S,
        _backend: &B,
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
        _backend: &B,
        _context: &mut CommitContext<S>,
    ) {
    }
}
