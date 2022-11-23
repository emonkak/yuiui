use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::element::Element;
use crate::id::{Depth, IdContext};
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, E> {
    const DEPTH: usize;

    type View: View<S, M, E>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool;

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        state: &S,
        messages: &mut Vec<M>,
        entry_point: &E,
        id_context: &mut IdContext,
    ) -> bool;
}

impl<C, CS, S, M, E> ComponentStack<S, M, E> for (ComponentNode<C, S, M, E>, CS)
where
    C: Component<S, M, E>,
    C::Element: Element<S, M, E, Components = CS>,
    CS: ComponentStack<S, M, E, View = <C::Element as Element<S, M, E>>::View>,
{
    const DEPTH: usize = 1 + CS::DEPTH;

    type View = <C::Element as Element<S, M, E>>::View;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        state: &S,
        id_context: &mut IdContext,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        if depth >= CS::DEPTH {
            id_context.set_depth(Self::DEPTH);
            let element = head_component.render(state, id_context);
            element.update(node, state, id_context)
        } else {
            CS::update(&mut node, depth, state, id_context)
        }
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        state: &S,
        messages: &mut Vec<M>,
        entry_point: &E,
        id_context: &mut IdContext,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            view: node.view,
            pending_view: node.pending_view,
            view_state: node.view_state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        if depth >= CS::DEPTH {
            id_context.set_depth(Self::DEPTH);
            head_component.commit(mode, node, state, id_context, messages, entry_point)
        } else {
            CS::commit(
                &mut node,
                mode,
                depth,
                state,
                messages,
                entry_point,
                id_context,
            )
        }
    }
}

#[derive(Debug)]
pub struct ComponentTermination<V> {
    _phantom: PhantomData<V>,
}

impl<V> ComponentTermination<V> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<V: View<S, M, E>, S, M, E> ComponentStack<S, M, E> for ComponentTermination<V> {
    const DEPTH: usize = 0;

    type View = V;

    fn update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, E>,
        _depth: Depth,
        _state: &S,
        id_context: &mut IdContext,
    ) -> bool {
        id_context.set_depth(Self::DEPTH);
        false
    }

    fn commit<'a>(
        _node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        _mode: CommitMode,
        _depth: Depth,
        _state: &S,
        _messages: &mut Vec<M>,
        _entry_point: &E,
        id_context: &mut IdContext,
    ) -> bool {
        id_context.set_depth(Self::DEPTH);
        false
    }
}
