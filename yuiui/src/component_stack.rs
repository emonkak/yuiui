use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::element::Element;
use crate::id::{Depth, IdStack};
use crate::store::Store;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, E> {
    const LEN: usize;

    type View: View<S, M, E>;

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>) -> Depth;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
    ) -> bool;

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool;
}

impl<C, CS, S, M, E> ComponentStack<S, M, E> for (ComponentNode<C, S, M, E>, CS)
where
    C: Component<S, M, E>,
    C::Element: Element<S, M, E, Components = CS>,
    CS: ComponentStack<S, M, E, View = <C::Element as Element<S, M, E>>::View>,
{
    const LEN: usize = 1 + CS::LEN;

    type View = <C::Element as Element<S, M, E>>::View;

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>) -> Depth {
        node.components.0.depth()
    }

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            depth: node.depth,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        if depth <= head_component.depth() {
            let element = head_component.component().render(store);
            element.update(node, id_stack, store)
        } else {
            CS::update(&mut node, depth, id_stack, store)
        }
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        id_stack: &mut IdStack,
        store: &Store<S>,
        messages: &mut Vec<M>,
        entry_point: &E,
    ) -> bool {
        let (head_component, tail_components) = node.components;
        let mut node = ViewNodeMut {
            id: node.id,
            depth: node.depth,
            view: node.view,
            pending_view: node.pending_view,
            state: node.state,
            children: node.children,
            components: tail_components,
            dirty: node.dirty,
        };
        if depth <= head_component.depth() {
            head_component.commit(mode, node, id_stack, store, messages, entry_point)
        } else {
            CS::commit(
                &mut node,
                mode,
                depth,
                id_stack,
                store,
                messages,
                entry_point,
            )
        }
    }
}

#[derive(Debug)]
pub struct ComponentTermination<V>(PhantomData<V>);

impl<V> ComponentTermination<V> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<V: View<S, M, E>, S, M, E> ComponentStack<S, M, E> for ComponentTermination<V> {
    const LEN: usize = 0;

    type View = V;

    fn depth<'a>(node: &mut ViewNodeMut<'a, V, Self, S, M, E>) -> Depth {
        node.depth
    }

    fn update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, E>,
        _depth: Depth,
        _id_stack: &mut IdStack,
        _store: &Store<S>,
    ) -> bool {
        false
    }

    fn commit<'a>(
        _node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        _mode: CommitMode,
        _depth: Depth,
        _id_stack: &mut IdStack,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _entry_point: &E,
    ) -> bool {
        false
    }
}
