use std::any;
use std::marker::PhantomData;

use crate::component::Component;
use crate::context::RenderContext;
use crate::view_node::{ViewNode, ViewNodeMut};

use super::{ComponentElement, Element, ElementSeq};

pub struct Consume<E, T, S, B> {
    render: fn(&T, &S, &B) -> E,
    _phantom: PhantomData<T>,
}

impl<E, T, S, B> Consume<E, T, S, B> {
    pub const fn new(render: fn(&T, &S, &B) -> E) -> ComponentElement<Self> {
        let connect = Self {
            render,
            _phantom: PhantomData,
        };
        ComponentElement::new(connect)
    }
}

impl<E, T, S, B> Clone for Consume<E, T, S, B> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<E, T, S, M, B> Component<S, M, B> for Consume<E, T, S, B>
where
    E: Element<S, M, B>,
    T: 'static,
{
    type Element = AsElement<Self>;

    type State = ();

    fn render(&self, _local_state: &Self::State, _state: &S, _backend: &B) -> Self::Element {
        AsElement::new(self.clone())
    }
}

pub struct AsElement<T> {
    inner: T,
}

impl<T> AsElement<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<E, T, S, M, B> Element<S, M, B> for AsElement<Consume<E, T, S, B>>
where
    E: Element<S, M, B>,
    T: 'static,
{
    type View = E::View;

    type Components = E::Components;

    const DEPTH: usize = E::DEPTH;

    fn render(
        self,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> ViewNode<Self::View, Self::Components, S, M, B> {
        let value = context
            .get_env::<T>()
            .unwrap_or_else(|| panic!("get env {}", any::type_name::<T>()));
        let element = (self.inner.render)(value, state, backend);
        element.render(context, state, backend)
    }

    fn update(
        self,
        node: &mut ViewNodeMut<Self::View, Self::Components, S, M, B>,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        let value = context
            .get_env::<T>()
            .unwrap_or_else(|| panic!("get env {}", any::type_name::<T>()));
        let element = (self.inner.render)(value, state, backend);
        element.update(node, context, state, backend)
    }
}

impl<E, T, S, M, B> ElementSeq<S, M, B> for AsElement<Consume<E, T, S, B>>
where
    E: Element<S, M, B>,
    T: 'static,
{
    type Storage = ViewNode<E::View, E::Components, S, M, B>;

    const DEPTH: usize = E::DEPTH;

    fn render_children(self, context: &mut RenderContext, state: &S, backend: &B) -> Self::Storage {
        self.render(context, state, backend)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        context: &mut RenderContext,
        state: &S,
        backend: &B,
    ) -> bool {
        self.update(&mut storage.borrow_mut(), context, state, backend)
    }
}
