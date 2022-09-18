use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{MessageContext, RenderContext};
use crate::element::Element;
use crate::id::Depth;
use crate::state::Store;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, B> {
    const LEN: usize;

    type View: View<S, M, B>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool;
}

impl<C, CS, S, M, B> ComponentStack<S, M, B> for (ComponentNode<C, S, M, B>, CS)
where
    C: Component<S, M, B>,
    C::Element: Element<S, M, B, Components = CS>,
    CS: ComponentStack<S, M, B, View = <C::Element as Element<S, M, B>>::View>,
{
    const LEN: usize = 1 + CS::LEN;

    type View = <C::Element as Element<S, M, B>>::View;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        let (head, tail) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            state: node.state,
            children: node.children,
            components: tail,
            dirty: node.dirty,
        };
        if target_depth <= current_depth {
            let element = head.render(store.state());
            element.update(&mut node, context, store)
        } else {
            CS::update(&mut node, target_depth, current_depth + 1, context, store)
        }
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        if target_depth <= current_depth {
            self.0.commit(mode, context, store, backend)
        } else {
            self.1.commit(
                mode,
                target_depth,
                current_depth + 1,
                context,
                store,
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

impl<V: View<S, M, B>, S, M, B> ComponentStack<S, M, B> for ComponentEnd<V> {
    const LEN: usize = 0;

    type View = V;

    fn update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, B>,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut RenderContext,
        _store: &mut Store<S>,
    ) -> bool {
        false
    }

    fn commit(
        &mut self,
        _mode: CommitMode,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut MessageContext<M>,
        _store: &mut Store<S>,
        _backend: &mut B,
    ) -> bool {
        false
    }
}
