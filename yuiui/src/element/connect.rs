use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::component_stack::ComponentStack;
use crate::context::{MessageContext, RenderContext};
use crate::event::{EventMask, HasEvent, Lifecycle};
use crate::id::{Depth, IdPath};
use crate::state::Store;
use crate::traversable::Traversable;
use crate::view::View;
use crate::view_node::{CommitMode, ViewNode, ViewNodeMut, ViewNodeSeq};

use super::{Element, ElementSeq};

pub struct Connect<T, FS, FM, SS, SM> {
    target: T,
    store_selector: Arc<FS>,
    message_selector: Arc<FM>,
    _phantom: PhantomData<(SS, SM)>,
}

impl<T, FS, FM, SS, SM> Connect<T, FS, FM, SS, SM> {
    pub fn new(target: T, store_selector: Arc<FS>, message_selector: Arc<FM>) -> Self {
        Self {
            target,
            store_selector,
            message_selector,
            _phantom: PhantomData,
        }
    }
}

impl<T, FS, FM, SS, SM> fmt::Debug for Connect<T, FS, FM, SS, SM>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Connect").field(&self.target).finish()
    }
}

impl<T, FS, FM, SS, SM, S, M, B> Element<S, M, B> for Connect<T, FS, FM, SS, SM>
where
    T: Element<SS, SM, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
{
    type View = Connect<T::View, FS, FM, SS, SM>;

    type Components = Connect<T::Components, FS, FM, SS, SM>;

    const DEPTH: usize = T::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        let sub_node = self.target.render(context, sub_store);
        ViewNode {
            id: sub_node.id,
            state: sub_node.state.map(|state| {
                state.map_view(|view| {
                    Connect::new(
                        view,
                        self.store_selector.clone(),
                        self.message_selector.clone(),
                    )
                })
            }),
            children: Connect::new(
                sub_node.children,
                self.store_selector.clone(),
                self.message_selector.clone(),
            ),
            components: Connect::new(
                sub_node.components,
                self.store_selector,
                self.message_selector,
            ),
            env: sub_node.env,
            event_mask: sub_node.event_mask,
            dirty: sub_node.dirty,
        }
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        with_sub_node(node, |sub_node| {
            self.target.update(sub_node, context, sub_store)
        })
    }
}

impl<T, FS, FM, SS, SM, S, M, B> ElementSeq<S, M, B> for Connect<T, FS, FM, SS, SM>
where
    T: ElementSeq<SS, SM, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
{
    type Storage = Connect<T::Storage, FS, FM, SS, SM>;

    const DEPTH: usize = T::DEPTH;

    fn render_children(self, context: &mut RenderContext, store: &Store<S>) -> Self::Storage {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        Connect::new(
            self.target.render_children(context, sub_store),
            self.store_selector.clone(),
            self.message_selector.clone(),
        )
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        store: &Store<S>,
    ) -> bool {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        self.target
            .update_children(&mut storage.target, context, sub_store)
    }
}

impl<T, FS, FM, SS, SM, S, M, B> ViewNodeSeq<S, M, B> for Connect<T, FS, FM, SS, SM>
where
    T: ViewNodeSeq<SS, SM, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
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
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        let result = self
            .target
            .commit(mode, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
        result
    }
}

impl<T, FS, FM, SS, SM, Visitor, Output, S, B> Traversable<Visitor, RenderContext, Output, S, B>
    for Connect<T, FS, FM, SS, SM>
where
    T: Traversable<Visitor, RenderContext, Output, SS, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Output {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        self.target.for_each(visitor, context, sub_store, backend)
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut RenderContext,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Option<Output> {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        self.target
            .search(id_path, visitor, context, sub_store, backend)
    }
}

impl<T, FS, FM, SS, SM, Visitor, S, M, B> Traversable<Visitor, MessageContext<M>, bool, S, B>
    for Connect<T, FS, FM, SS, SM>
where
    T: Traversable<Visitor, MessageContext<SM>, bool, SS, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
{
    fn for_each(
        &mut self,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        let result = self
            .target
            .for_each(visitor, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
        result
    }

    fn search(
        &mut self,
        id_path: &IdPath,
        visitor: &mut Visitor,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> Option<bool> {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        let result = self
            .target
            .search(id_path, visitor, &mut sub_context, sub_store, backend);
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
        result
    }
}

impl<T, FS, FM, SS, SM, S, M, B> ComponentStack<S, M, B> for Connect<T, FS, FM, SS, SM>
where
    T: ComponentStack<SS, SM, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
{
    const LEN: usize = T::LEN;

    type View = Connect<T::View, FS, FM, SS, SM>;

    fn update<'a>(
        node: &mut ViewNodeMut<'a, Self::View, Self, S, M, B>,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut RenderContext,
        store: &mut Store<S>,
    ) -> bool {
        let store_selector = &node.components.store_selector;
        let sub_store = unsafe { &mut *(store_selector(store) as *const _ as *mut Store<SS>) };
        with_sub_node(node, |sub_node| {
            T::update(sub_node, target_depth, current_depth, context, sub_store)
        })
    }

    fn commit(
        &mut self,
        mode: CommitMode,
        target_depth: Depth,
        current_depth: Depth,
        context: &mut MessageContext<M>,
        store: &mut Store<S>,
        backend: &mut B,
    ) -> bool {
        let sub_store =
            unsafe { &mut *((self.store_selector)(store) as *const _ as *mut Store<SS>) };
        match mode {
            CommitMode::Mount => {
                sub_store.add_subscriber(context.id_path().to_vec(), current_depth);
            }
            CommitMode::Unmount => {
                sub_store.remove_subscriber(context.id_path(), current_depth);
            }
            CommitMode::Update => {}
        }
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        let result = self.target.commit(
            mode,
            target_depth,
            current_depth,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
        result
    }
}

impl<T, FS, FM, SS, SM, S, M, B> View<S, M, B> for Connect<T, FS, FM, SS, SM>
where
    T: View<SS, SM, B>,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    SM: 'static,
    M: 'static,
{
    type Children = Connect<T::Children, FS, FM, SS, SM>;

    type State = T::State;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        state: &S,
        backend: &mut B,
    ) {
        let sub_lifecycle = lifecycle.map(|view| &view.target);
        let sub_store = (self.store_selector)(state);
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        self.target.lifecycle(
            sub_lifecycle,
            view_state,
            &children.target,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
    }

    fn event(
        &self,
        event: <Self as HasEvent>::Event,
        view_state: &mut Self::State,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        context: &mut MessageContext<M>,
        state: &S,
        backend: &mut B,
    ) {
        let sub_store = (self.store_selector)(state);
        let mut sub_context = context.new_sub_context(sub_store.to_subscribers());
        self.target.event(
            event,
            view_state,
            &children.target,
            &mut sub_context,
            sub_store,
            backend,
        );
        context.merge_sub_context(sub_context, self.message_selector.as_ref());
    }

    fn build(
        &self,
        children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        state: &S,
        backend: &mut B,
    ) -> Self::State {
        let sub_store = (self.store_selector)(state);
        self.target.build(&children.target, sub_store, backend)
    }
}

impl<'event, T, FS, FM, SS, SM> HasEvent<'event> for Connect<T, FS, FM, SS, SM>
where
    T: HasEvent<'event>,
{
    type Event = T::Event;
}

fn with_sub_node<Callback, Output, FS, FM, SS, SM, V, CS, S, M, B>(
    node: &mut ViewNodeMut<Connect<V, FS, FM, SS, SM>, Connect<CS, FS, FM, SS, SM>, S, M, B>,
    callback: Callback,
) -> Output
where
    Callback: FnOnce(&mut ViewNodeMut<V, CS, SS, SM, B>) -> Output,
    FS: Fn(&S) -> &Store<SS> + Sync + Send + 'static,
    FM: Fn(SM) -> M + Sync + Send + 'static,
    V: View<SS, SM, B>,
    CS: ComponentStack<SS, SM, B, View = V>,
{
    let store_selector = &node.components.store_selector;
    let message_selector = &node.components.message_selector;
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
    *node.state = sub_node_state.map(|state| {
        state.map_view(|view| Connect::new(view, store_selector.clone(), message_selector.clone()))
    });
    result
}