use std::any;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::component::Component;
use crate::context::RenderContext;
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};

use super::{ComponentElement, Element, ElementSeq};

pub struct Provide<E, T> {
    element: E,
    value: T,
}

impl<E, T> Provide<E, T> {
    pub const fn new(element: E, value: T) -> Self {
        Self { element, value }
    }
}

impl<E, T, S, B> Element<S, B> for Provide<E, T>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type View = E::View;

    type Components = E::Components;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let mut node = self.element.render(state, backend, context);
        node.env = Some(Rc::new(self.value));
        node
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let result = self.element.update(scope, state, backend, context);
        *scope.env = Some(Rc::new(self.value));
        result
    }
}

impl<E, T, S, B> ElementSeq<S, B> for Provide<E, T>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type Storage = ViewNode<E::View, E::Components, S, B>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        self.render(state, backend, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.scope(), state, backend, context)
    }
}

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

impl<E, T, S, B> Component<S, B> for Consume<E, T, S, B>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type Element = AsElement<Self>;

    fn render(&self, _state: &S, _backend: &B) -> Self::Element {
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

impl<E, T, S, B> Element<S, B> for AsElement<Consume<E, T, S, B>>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type View = E::View;

    type Components = E::Components;

    fn render(
        self,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, B> {
        let value = context
            .get_env::<T>()
            .unwrap_or_else(|| panic!("get env {}", any::type_name::<T>()));
        let element = (self.inner.render)(value, state, backend);
        element.render(state, backend, context)
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let value = context
            .get_env::<T>()
            .unwrap_or_else(|| panic!("get env {}", any::type_name::<T>()));
        let element = (self.inner.render)(value, state, backend);
        element.update(scope, state, backend, context)
    }
}

impl<E, T, S, B> ElementSeq<S, B> for AsElement<Consume<E, T, S, B>>
where
    E: Element<S, B>,
    T: 'static,
    S: State,
{
    type Storage = ViewNode<E::View, E::Components, S, B>;

    fn render_children(self, state: &S, backend: &B, context: &mut RenderContext) -> Self::Storage {
        self.render(state, backend, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.scope(), state, backend, context)
    }
}
