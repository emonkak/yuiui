use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component_stack::ComponentStack;
use crate::context::{EffectContext, RenderContext};
use crate::effect::EffectOps;
use crate::event::{EventMask, HasEvent, Lifecycle};
use crate::id::{Depth, IdPath};
use crate::state::State;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct Scope<T, F, SS> {
    target: T,
    selector_fn: Arc<F>,
    sub_state: PhantomData<SS>,
}

impl<T, F, SS> Scope<T, F, SS> {
    pub fn new(target: T, selector_fn: Arc<F>) -> Self {
        Self {
            target,
            selector_fn,
            sub_state: PhantomData,
        }
    }
}

impl<T, F, SS> fmt::Debug for Scope<T, F, SS>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Scope").field(&self.target).finish()
    }
}

impl<T, F, SS, S, B> Element<S, B> for Scope<T, F, SS>
where
    T: Element<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type View = Scope<T::View, F, SS>;

    type Components = Scope<T::Components, F, SS>;

    const DEPTH: usize = T::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let sub_state = (self.selector_fn)(state);
        let sub_node = self.target.render(context, sub_state, backend);
        ViewNode {
            id: sub_node.id,
            state: sub_node
                .state
                .map(|state| state.map_view(|view| Scope::new(view, self.selector_fn.clone()))),
            children: Scope::new(sub_node.children, self.selector_fn.clone()),
            components: Scope::new(sub_node.components, self.selector_fn),
            env: sub_node.env,
            event_mask: sub_node.event_mask,
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        with_sub_node(node, |sub_node| {
            self.target.update(sub_node, context, sub_state, backend)
        })
    }
}

impl<T, F, SS, S, B> ElementSeq<S, B> for Scope<T, F, SS>
where
    T: ElementSeq<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Storage = Scope<T::Storage, F, SS>;

    const DEPTH: usize = T::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        let sub_state = (self.selector_fn)(state);
        Scope::new(
            self.target.render_children(context, sub_state, backend),
            self.selector_fn.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .update_children(&mut storage.target, context, sub_state, backend)
    }
}

impl<T, F, SS, S, B> ViewNodeSeq<S, B> for Scope<T, F, SS>
where
    T: ViewNodeSeq<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    fn event_mask() -> &'static EventMask {
        T::event_mask()
    }

    fn len(&self) -> usize {
        self.target.len()
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .commit(mode, &mut sub_context, sub_state, backend)
            .lift(&self.selector_fn)
    }
}

impl<T, F, SS, Visitor, Output, S, B> Traversable<Visitor, RenderContext, Output, S, B>
    for Scope<T, F, SS>
where
    T: Traversable<Visitor, RenderContext, Output, SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> Output {
        let sub_state = (self.selector_fn)(state);
        self.target.for_each(visitor, context, sub_state, backend)
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> Option<Output> {
        let sub_state = (self.selector_fn)(state);
        self.target
            .search(id_path, visitor, context, sub_state, backend)
    }
}

impl<T, F, SS, Visitor, S, B> Traversable<Visitor, EffectContext, EffectOps<S>, S, B>
    for Scope<T, F, SS>
where
    T: Traversable<Visitor, EffectContext, EffectOps<SS>, SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .for_each(visitor, &mut sub_context, sub_state, backend)
            .lift(&self.selector_fn)
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> Option<EffectOps<S>> {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .search(id_path, visitor, &mut sub_context, sub_state, backend)
            .map(|result| result.lift(&self.selector_fn))
    }
}

impl<T, F, SS, S, B> ComponentStack<S, B> for Scope<T, F, SS>
where
    T: ComponentStack<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    const LEN: usize = T::LEN;

    type View = Scope<T::View, F, SS>;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target
            .commit(
                mode,
                target_depth,
                current_depth,
                &mut sub_context,
                sub_state,
                backend,
            )
            .lift(&self.selector_fn)
    }

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let sub_state = (node.components.selector_fn)(state);
        with_sub_node(node, |sub_node| {
            T::update(
                sub_node,
                target_depth,
                current_depth,
                context,
                sub_state,
                backend,
            )
        })
    }
}

impl<T, F, SS, S, B> View<S, B> for Scope<T, F, SS>
where
    T: View<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Children = Scope<T::Children, F, SS>;

    type State = T::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        context: &EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let sub_lifecycle = lifecycle.map(|view| &view.target);
        let sub_context = context.new_sub_context();
        let sub_state = (self.selector_fn)(state);
        self.target
            .lifecycle(
                sub_lifecycle,
                view_state,
                &children.target,
                &sub_context,
                sub_state,
                backend,
            )
            .lift(&self.selector_fn)
    }

    fn event(
        &self,
        event: <Self as HasEvent>::Event,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        context: &EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        let sub_context = context.new_sub_context();
        let sub_state = (self.selector_fn)(state);
        self.target
            .event(
                event,
                view_state,
                &children.target,
                &sub_context,
                sub_state,
                backend,
            )
            .lift(&self.selector_fn)
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        state: &S,
        backend: &B,
    ) -> Self::State {
        let sub_state = (self.selector_fn)(state);
        self.target.build(&children.target, sub_state, backend)
    }
}

impl<'event, T, F, SS> HasEvent<'event> for Scope<T, F, SS>
where
    T: HasEvent<'event>,
{
    type Event = T::Event;
}

fn with_sub_node<Callback, Output, F, SS, V, CS, S, B>(
    node: &mut ViewNodeMut<Scope<V, F, SS>, Scope<CS, F, SS>, S, B>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(&mut ViewNodeMut<V, CS, SS, B>) -> Output,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    V: View<SS, B>,
    CS: ComponentStack<SS, B, View = V>,
    S: State,
{
    let selector_fn = &node.components.selector_fn;
    let mut sub_node_state = node
        .state
        .take()
        .map(|state| state.map_view(|view| view.target));
    let mut sub_node = ViewNodeMut {
        id: node.id,
        state: &mut sub_node_state,
        children: &mut node.children.target,
        components: &mut node.components.target,
        env: &mut node.env,
        dirty: &mut node.dirty,
    };
    let result = callback(&mut sub_node);
    *node.state =
        sub_node_state.map(|state| state.map_view(|view| Scope::new(view, selector_fn.clone())));
    result
}
