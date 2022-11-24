use std::marker::PhantomData;

use crate::component::Component;
use crate::context::RenderContext;
use crate::element::Element;
use crate::id::Level;
use crate::view::View;
use crate::view_node::ViewNodeMut;

pub trait ComponentStack<S, M, E> {
    const LEVEL: Level;

    type View: View<S, M, E>;

    fn force_update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
    ) -> bool;
}

impl<C, CS, S, M, E> ComponentStack<S, M, E> for (C, CS)
where
    C: Component<S, M, E>,
    C::Element: Element<S, M, E, Components = CS>,
    CS: ComponentStack<S, M, E, View = <C::Element as Element<S, M, E>>::View>,
{
    const LEVEL: Level = 1 + CS::LEVEL;

    type View = <C::Element as Element<S, M, E>>::View;

    fn force_update<'a>(
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
            element.update(&mut node, context)
        } else {
            CS::force_update(&mut node, level, context)
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

impl<V, S, M, E> ComponentStack<S, M, E> for ComponentTermination<V>
where
    V: View<S, M, E>,
{
    const LEVEL: Level = 0;

    type View = V;

    fn force_update<'a>(
        _node: &mut ViewNodeMut<'a, V, Self, S, M, E>,
        _level: Level,
        context: &mut RenderContext<S>,
    ) -> bool {
        context.id_stack.set_level(Self::LEVEL);
        false
    }
}
