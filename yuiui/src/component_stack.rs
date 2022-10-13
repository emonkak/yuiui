use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::element::Element;
use crate::id::{Depth, IdContext};
use crate::store::Store;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, B> {
    const LEN: usize;

    type View: View<S, M, B>;

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>) -> Depth;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
    ) -> bool;

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        mode: CommitMode,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
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

    fn depth<'a>(node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>) -> Depth {
        node.components.0.depth()
    }

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        depth: Depth,
        id_context: &mut IdContext,
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
            element.update(node, id_context, store)
        } else {
            CS::update(&mut node, depth, id_context, store)
        }
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        mode: CommitMode,
        depth: Depth,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &mut B,
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
            head_component.commit(mode, node, id_context, store, messages, backend)
        } else {
            CS::commit(&mut node, mode, depth, id_context, store, messages, backend)
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

impl<V: View<S, M, B>, S, M, B> ComponentStack<S, M, B> for ComponentTermination<V> {
    const LEN: usize = 0;

    type View = V;

    fn depth<'a>(node: &mut ViewNodeMut<'a, V, Self, S, M, B>) -> Depth {
        node.depth
    }

    fn update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, B>,
        _depth: Depth,
        _id_context: &mut IdContext,
        _store: &Store<S>,
    ) -> bool {
        false
    }

    fn commit<'a>(
        _node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        _mode: CommitMode,
        _depth: Depth,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _backend: &mut B,
    ) -> bool {
        false
    }
}
