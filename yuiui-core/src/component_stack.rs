use std::marker::PhantomData;
use std::{fmt, mem};

use crate::component::Component;
use crate::context::{CommitContext, RenderContext};
use crate::element::Element;
use crate::event::Lifecycle;
use crate::id::Depth;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNodeMut};

pub trait ComponentStack<S, M, E> {
    const DEPTH: usize;

    type View: View<S, M, E>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        depth: Depth,
        context: &mut RenderContext<S>,
    ) -> bool;

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
        context: &mut CommitContext<S, M, E>,
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
        if depth >= CS::DEPTH {
            context.id_stack.set_depth(Self::DEPTH);
            let element = head_component.component.render(context);
            element.update(node, context)
        } else {
            CS::update(&mut node, depth, context)
        }
    }

    fn commit<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        mode: CommitMode,
        depth: Depth,
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
        if depth >= CS::DEPTH {
            context.id_stack.set_depth(Self::DEPTH);
            head_component.commit(mode, node, context)
        } else {
            CS::commit(&mut node, mode, depth, context)
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
        context: &mut RenderContext<S>,
    ) -> bool {
        context.id_stack.set_depth(Self::DEPTH);
        false
    }

    fn commit<'a>(
        _node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        _mode: CommitMode,
        _depth: Depth,
        context: &mut CommitContext<S, M, E>,
    ) -> bool {
        context.id_stack.set_depth(Self::DEPTH);
        false
    }
}

pub struct ComponentNode<C: Component<S, M, E>, S, M, E> {
    component: C,
    pending_component: Option<C>,
    is_mounted: bool,
    _phantom: PhantomData<(S, M, E)>,
}

impl<C, S, M, E> ComponentNode<C, S, M, E>
where
    C: Component<S, M, E>,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            is_mounted: false,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn update(&mut self, component: C) {
        self.pending_component = Some(component);
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        view_node: ViewNodeMut<
            '_,
            <C::Element as Element<S, M, E>>::View,
            <C::Element as Element<S, M, E>>::Components,
            S,
            M,
            E,
        >,
        context: &mut CommitContext<S, M, E>,
    ) -> bool {
        match mode {
            CommitMode::Mount => {
                let lifecycle = if self.is_mounted {
                    Lifecycle::Remount
                } else {
                    Lifecycle::Mount
                };
                self.component.lifecycle(lifecycle, view_node, context);
                self.is_mounted = true;
                true
            }
            CommitMode::Update => {
                if let Some(pending_component) = self.pending_component.take() {
                    let old_component = mem::replace(&mut self.component, pending_component);
                    self.component
                        .lifecycle(Lifecycle::Update(old_component), view_node, context);
                    true
                } else {
                    false
                }
            }
            CommitMode::Unmount => {
                self.component
                    .lifecycle(Lifecycle::Unmount, view_node, context);
                true
            }
        }
    }

    pub fn component(&self) -> &C {
        &self.component
    }
}

impl<C, S, M, E> fmt::Debug for ComponentNode<C, S, M, E>
where
    C: Component<S, M, E> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ComponentNode")
            .field("component", &self.component)
            .field("pending_component", &self.pending_component)
            .field("is_mounted", &self.is_mounted)
            .finish()
    }
}
