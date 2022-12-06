use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::event::{EventTarget, Lifecycle};
use crate::id::Level;
use crate::view::View;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{Element, ElementSeq};

pub struct HookElement<Inner, Callback, S, M, E>
where
    Inner: Element<S, M, E>,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    inner: Inner,
    callback: Callback,
    _phantom: PhantomData<(S, M, E)>,
}

impl<Inner, Callback, S, M, E> HookElement<Inner, Callback, S, M, E>
where
    Inner: Element<S, M, E>,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    pub const fn new(inner: Inner, callback: Callback) -> Self {
        Self {
            inner,
            callback,
            _phantom: PhantomData,
        }
    }
}

impl<Inner, Callback, S, M, E> Element<S, M, E> for HookElement<Inner, Callback, S, M, E>
where
    Inner: Element<S, M, E>,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    type View = Hook<Inner::View, Callback>;

    type Components = Hook<Inner::Components, Callback>;

    fn render(
        self,
        context: &mut RenderContext<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, E> {
        let node = self.inner.render(context);
        ViewNode {
            id: node.id,
            view: Hook::new(node.view, self.callback.clone()),
            pending_view: node
                .pending_view
                .map(|view| Hook::new(view, self.callback.clone())),
            view_state: node.view_state,
            children: node.children,
            components: Hook::new(node.components, self.callback),
            dirty: node.dirty,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, E>,
        context: &mut RenderContext<S>,
    ) -> bool {
        node.view.callback = self.callback.clone();
        node.components.callback = self.callback;
        with_inner_node(node, |mut inner_node| {
            self.inner.update(&mut inner_node, context)
        })
    }
}

impl<Inner, Callback, S, M, E> ElementSeq<S, M, E> for HookElement<Inner, Callback, S, M, E>
where
    Inner: Element<S, M, E>,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    type Storage =
        ViewNode<<Self as Element<S, M, E>>::View, <Self as Element<S, M, E>>::Components, S, M, E>;

    fn render_children(self, context: &mut RenderContext<S>) -> Self::Storage {
        context.render_node(self)
    }

    fn update_children(self, storage: &mut Self::Storage, context: &mut RenderContext<S>) -> bool {
        context.update_node(self, storage)
    }
}

impl<Inner, Callback, S, M, E> fmt::Debug for HookElement<Inner, Callback, S, M, E>
where
    Inner: Element<S, M, E> + fmt::Debug,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("HookElement").field(&self.inner).finish()
    }
}

pub struct Hook<Inner, Callback> {
    inner: Inner,
    callback: Callback,
}

impl<Inner, Callback> Hook<Inner, Callback> {
    fn new(inner: Inner, callback: Callback) -> Self {
        Self { inner, callback }
    }
}

impl<Inner, Callback, S, M, E> View<S, M, E> for Hook<Inner, Callback>
where
    Inner: View<S, M, E>,
    Callback: HookCallback<Inner, S, M, E>,
{
    type Children = Inner::Children;

    type State = Inner::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        let lifecycle = lifecycle.map(|view| view.inner);
        self.callback
            .call(&self.inner, &lifecycle, view_state, children, context);
        self.inner
            .lifecycle(lifecycle, view_state, children, context)
    }

    fn event(
        &self,
        event: <Self as EventTarget>::Event,
        view_state: &mut Self::State,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        self.inner.event(event, view_state, children, context)
    }

    fn build(
        &self,
        children: &mut <Self::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) -> Self::State {
        self.inner.build(children, context)
    }
}

impl<'event, Inner, Callback> EventTarget<'event> for Hook<Inner, Callback>
where
    Inner: EventTarget<'event>,
{
    type Event = Inner::Event;
}

impl<Inner, Callback, S, M, E> ComponentStack<S, M, E> for Hook<Inner, Callback>
where
    Inner: ComponentStack<S, M, E>,
    Callback: HookCallback<Inner::View, S, M, E>,
{
    const LEVEL: Level = Inner::LEVEL;

    type View = Hook<Inner::View, Callback>;

    fn force_update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, E>,
        level: Level,
        context: &mut RenderContext<S>,
    ) -> bool {
        with_inner_node(node, |mut inner_node| {
            Inner::force_update(&mut inner_node, level, context)
        })
    }
}

impl<Inner, Callback> fmt::Debug for Hook<Inner, Callback>
where
    Inner: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Hook").field(&self.inner).finish()
    }
}

fn with_inner_node<F, T, Callback, V, CS, S, M, E>(
    node: &mut ViewNodeMut<Hook<V, Callback>, Hook<CS, Callback>, S, M, E>,
    f: F,
) -> T
where
    F: FnOnce(ViewNodeMut<V, CS, S, M, E>) -> T,
    Callback: HookCallback<V, S, M, E>,
    V: View<S, M, E>,
    CS: ComponentStack<S, M, E, View = V>,
{
    let mut inner_pending_view = node.pending_view.take().map(|view| view.inner);
    let inner_node = ViewNodeMut {
        id: node.id,
        view: &mut node.view.inner,
        pending_view: &mut inner_pending_view,
        view_state: node.view_state,
        children: node.children,
        components: &mut node.components.inner,
        dirty: node.dirty,
    };
    let callback = &node.view.callback;
    let result = f(inner_node);
    *node.pending_view = inner_pending_view.map(|view| Hook::new(view, callback.clone()));
    result
}

pub trait HookCallback<V, S, M, E>: Clone
where
    V: View<S, M, E>,
{
    fn call(
        &self,
        view: &V,
        lifecycle: &Lifecycle<V>,
        view_state: &V::State,
        children: &<V::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    );
}

impl<F, V, S, M, E> HookCallback<V, S, M, E> for F
where
    F: Fn(
            &V,
            &Lifecycle<V>,
            &V::State,
            &<V::Children as ElementSeq<S, M, E>>::Storage,
            &mut CommitContext<S, M, E>,
        ) + Clone,
    V: View<S, M, E>,
{
    fn call(
        &self,
        view: &V,
        lifecycle: &Lifecycle<V>,
        view_state: &V::State,
        children: &<V::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        self(view, lifecycle, view_state, children, context);
    }
}

impl<F, V, S, M, E> HookCallback<V, S, M, E> for Rc<F>
where
    F: Fn(
        &V,
        &Lifecycle<V>,
        &V::State,
        &<V::Children as ElementSeq<S, M, E>>::Storage,
        &mut CommitContext<S, M, E>,
    ),
    V: View<S, M, E>,
{
    fn call(
        &self,
        view: &V,
        lifecycle: &Lifecycle<V>,
        view_state: &V::State,
        children: &<V::Children as ElementSeq<S, M, E>>::Storage,
        context: &mut CommitContext<S, M, E>,
    ) {
        self(view, lifecycle, view_state, children, context);
    }
}
