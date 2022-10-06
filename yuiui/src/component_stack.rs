use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{MessageContext, RenderContext};
use crate::element::Element;
use crate::id::Depth;
use crate::state::Store;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, R> {
    const LEN: usize;

    type View: View<S, M, R>;

    fn update<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool;

    fn commit<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool;
}

impl<C, CS, S, M, R> ComponentStack<S, M, R> for (ComponentNode<C, S, M, R>, CS)
where
    C: Component<S, M, R>,
    C::Element: Element<S, M, R, Components = CS>,
    CS: ComponentStack<S, M, R, View = <C::Element as Element<S, M, R>>::View>,
{
    const LEN: usize = 1 + CS::LEN;

    type View = <C::Element as Element<S, M, R>>::View;

    fn update<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let (head, tail) = node.components;
        let node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail,
            dirty: node.dirty,
        };
        if target_depth <= current_depth {
            let element = head.component().render(store);
            element.update(node, context, store)
        } else {
            CS::update(node, target_depth, current_depth + 1, context, store)
        }
    }

    fn commit<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) -> bool {
        let (head, tail) = node.components;
        let node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail,
            dirty: node.dirty,
        };
        if target_depth <= current_depth {
            head.commit(mode, node.into(), context, store, renderer)
        } else {
            CS::commit(
                node,
                mode,
                target_depth,
                current_depth + 1,
                context,
                store,
                renderer,
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

impl<V: View<S, M, R>, S, M, R> ComponentStack<S, M, R> for ComponentEnd<V> {
    const LEN: usize = 0;

    type View = V;

    fn update<'a>(
        _node: ViewNodeMut<'a, V, Self, S, M, R>,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut RenderContext,
        _store: &Store<S>,
    ) -> bool {
        false
    }

    fn commit<'a>(
        _node: ViewNodeMut<'a, Self::View, Self, S, M, R>,
        _mode: CommitMode,
        _target_depth: Depth,
        _current_depth: Depth,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> bool {
        false
    }
}
