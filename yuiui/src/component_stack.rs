use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{EffectContext, RenderContext};
use crate::effect::EffectOps;
use crate::element::Element;
use crate::id::Depth;
use crate::state::State;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S: State, B>: Sized {
    const LEN: usize;

    type View: View<S, B>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S>;
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
        node: &mut ViewNodeMut<'a, Self::View, Self, S, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
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
        if target_depth <= current_depth {
            let element = head.render(state, backend);
            element.update(&mut node, context, state, backend)
        } else {
            CS::update(
                &mut node,
                target_depth,
                current_depth + 1,
                context,
                state,
                backend,
            )
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        if target_depth <= current_depth {
            self.0.commit(mode, current_depth, context, state, backend)
        } else {
            self.1.commit(
                mode,
                target_depth,
                current_depth + 1,
                context,
                state,
                backend,
            )
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
        _node: &mut ViewNodeMut<'a, V, Self, S, B>,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut RenderContext,
        _state: &S,
        _backend: &B,
    ) -> bool {
        false
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EffectOps<S> {
        EffectOps::nop()
    }
}
