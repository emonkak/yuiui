use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component_stack::ComponentStack;
use crate::context::{CommitContext, RenderContext};
use crate::event::{EventMask, EventResult, HasEvent, Lifecycle};
use crate::id::{ComponentIndex, IdPath};
use crate::state::State;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct Adapt<T, F, SS> {
    target: T,
    selector_fn: Arc<F>,
    sub_state: PhantomData<SS>,
}

impl<T, F, SS> Adapt<T, F, SS> {
    pub fn new(target: T, selector_fn: Arc<F>) -> Self {
        Self {
            target,
            selector_fn,
            sub_state: PhantomData,
        }
    }
}

impl<T, F, SS> fmt::Debug for Adapt<T, F, SS>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Adapt").field(&self.target).finish()
    }
}

impl<T, F, SS, S, B> Element<S, B> for Adapt<T, F, SS>
where
    T: Element<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type View = Adapt<T::View, F, SS>;

    type Components = Adapt<T::Components, F, SS>;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let sub_state = (self.selector_fn)(state);
        let sub_node = self.target.render(sub_state, backend, context);
        ViewNode {
            id: sub_node.id,
            state: sub_node
                .state
                .map(|state| state.map_view(|view| Adapt::new(view, self.selector_fn.clone()))),
            children: Adapt::new(sub_node.children, self.selector_fn.clone()),
            components: Adapt::new(sub_node.components, self.selector_fn),
            env: sub_node.env,
            event_mask: sub_node.event_mask,
            dirty: true,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let mut sub_node_state = node
            .state
            .take()
            .map(|state| state.map_view(|view| view.target));
        let mut sub_node = ViewNodeMut {
            id: node.id,
            state: &mut sub_node_state,
            children: &mut node.children.target,
            components: &mut node.components.target,
            env: node.env,
            dirty: node.dirty,
        };
        let sub_state = (self.selector_fn)(state);
        let has_changed = self
            .target
            .update(&mut sub_node, sub_state, backend, context);
        *node.state = sub_node_state
            .map(|state| state.map_view(|view| Adapt::new(view, self.selector_fn.clone())));
        has_changed
    }
}

impl<T, F, SS, S, B> ElementSeq<S, B> for Adapt<T, F, SS>
where
    T: ElementSeq<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Storage = Adapt<T::Storage, F, SS>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        let sub_state = (self.selector_fn)(state);
        Adapt::new(
            self.target.render_children(sub_state, backend, context),
            self.selector_fn.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .update_children(&mut storage.target, sub_state, backend, context)
    }
}

impl<T, F, SS, S, B> ViewNodeSeq<S, B> for Adapt<T, F, SS>
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
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let has_changed = self
            .target
            .commit(mode, sub_state, backend, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
        has_changed
    }
}

impl<T, F, SS, Visitor, S, B> Traversable<Visitor, RenderContext, S, B> for Adapt<T, F, SS>
where
    T: Traversable<Visitor, RenderContext, SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target.for_each(visitor, sub_state, backend, context)
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        self.target
            .search(id_path, visitor, sub_state, backend, context)
    }
}

impl<T, F, SS, Visitor, S, B> Traversable<Visitor, CommitContext<S>, S, B> for Adapt<T, F, SS>
where
    T: Traversable<Visitor, CommitContext<SS>, SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let result = self
            .target
            .for_each(visitor, sub_state, backend, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) -> bool {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        let found = self
            .target
            .search(id_path, visitor, sub_state, backend, &mut sub_context);
        context.merge_sub_context(sub_context, &self.selector_fn);
        found
    }
}

impl<T, F, SS, S, B> ComponentStack<S, B> for Adapt<T, F, SS>
where
    T: ComponentStack<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    const LEN: usize = T::LEN;

    type View = Adapt<T::View, F, SS>;

    fn commit(
        &mut self,
        mode: CommitMode,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut CommitContext<S>,
    ) {
        let sub_state = (self.selector_fn)(state);
        let mut sub_context = context.new_sub_context();
        self.target.commit(
            mode,
            target_index,
            current_index,
            sub_state,
            backend,
            &mut sub_context,
        );
        context.merge_sub_context(sub_context, &self.selector_fn);
    }

    fn update<'a>(
        node: ViewNodeMut<'a, Self::View, Self, S, B>,
        target_index: ComponentIndex,
        current_index: ComponentIndex,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let mut sub_node_state = node
            .state
            .take()
            .map(|state| state.map_view(|view| view.target));
        let selector_fn = &node.components.selector_fn;
        let sub_node = ViewNodeMut {
            id: node.id,
            state: &mut sub_node_state,
            children: &mut node.children.target,
            components: &mut node.components.target,
            env: node.env,
            dirty: node.dirty,
        };
        let sub_state = selector_fn(state);
        let has_changed = T::update(
            sub_node,
            target_index,
            current_index,
            sub_state,
            backend,
            context,
        );
        *node.state = sub_node_state
            .map(|state| state.map_view(|view| Adapt::new(view, selector_fn.clone())));
        has_changed
    }
}

impl<T, F, SS, S, B> View<S, B> for Adapt<T, F, SS>
where
    T: View<SS, B>,
    F: Fn(&S) -> &SS + Sync + Send + 'static,
    SS: State,
    S: State,
{
    type Widget = T::Widget;

    type Children = Adapt<T::Children, F, SS>;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        widget: &mut Self::Widget,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        id_path: &IdPath,
        state: &S,
        backend: &B,
    ) -> EventResult<S> {
        let sub_lifecycle = lifecycle.map(|view| &view.target);
        let sub_state = (self.selector_fn)(state);
        self.target
            .lifecycle(
                sub_lifecycle,
                widget,
                &children.target,
                id_path,
                sub_state,
                backend,
            )
            .lift(&self.selector_fn)
    }

    fn event(
        &self,
        event: <Self as HasEvent>::Event,
        widget: &mut Self::Widget,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        id_path: &IdPath,
        state: &S,
        backend: &B,
    ) -> EventResult<S> {
        let sub_state = (self.selector_fn)(state);
        self.target
            .event(event, widget, &children.target, id_path, sub_state, backend)
            .lift(&self.selector_fn)
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, B>>::Storage,
        state: &S,
        backend: &B,
    ) -> Self::Widget {
        let sub_state = (self.selector_fn)(state);
        self.target.build(&children.target, sub_state, backend)
    }
}

impl<'event, T, F, SS> HasEvent<'event> for Adapt<T, F, SS>
where
    T: HasEvent<'event>,
{
    type Event = T::Event;
}
