use std::marker::PhantomData;

use crate::component::Component;
use crate::component_node::ComponentNode;
use crate::context::{CommitContext, RenderContext};
use crate::element::Element;
use crate::id::Level;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, E> {
    const LEVEL: Level;

    type View: View<S, M, E>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
    ) -> bool;

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        level: Level,
        context: &mut CommitContext<S, M, E>,
    ) -> bool;
}

impl<C, CS, S, M, E> ComponentStack<S, M, E> for (ComponentNode<C, S, M, E>, CS)
where
    C: Component<S, M, E>,
    C::Element: Element<S, M, E, Components = CS>,
    CS: ComponentStack<S, M, E, View = <C::Element as Element<S, M, E>>::View>,
{
    const LEVEL: Level = 1 + CS::LEVEL;

    type View = <C::Element as Element<S, M, E>>::View;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
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
        if level >= CS::LEVEL {
            context.id_stack.set_level(Self::LEVEL);
            let element = head_component.render(context);
            element.update(node, context)
        } else {
            CS::update(&mut node, level, context)
        }
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        level: Level,
        context: &mut CommitContext<S, M, E>,
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
        if level >= CS::LEVEL {
            context.id_stack.set_level(Self::LEVEL);
            head_component.commit(mode, node, context)
        } else {
            CS::commit(&mut node, mode, level, context)
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
    const LEVEL: Level = 0;

    type View = V;

    fn update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, E>,
        _level: Level,
        context: &mut RenderContext<S>,
    ) -> bool {
        context.id_stack.set_level(Self::LEVEL);
        false
    }

    fn commit<'a>(
        _node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        _mode: CommitMode,
        _level: Level,
        context: &mut CommitContext<S, M, E>,
    ) -> bool {
        context.id_stack.set_level(Self::LEVEL);
        false
    }
}
