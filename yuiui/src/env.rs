use std::marker::PhantomData;
use std::rc::Rc;

use crate::component::Component;
use crate::context::RenderContext;
use crate::element::{ComponentElement, Element, ElementSeq};
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};

pub struct Provide<El, T> {
    element: El,
    value: T,
}

impl<El, T> Provide<El, T> {
    pub const fn new(element: El, value: T) -> Self {
        Self { element, value }
    }
}

impl<El, T, S, E> Element<S, E> for Provide<El, T>
where
    El: Element<S, E>,
    T: 'static,
    S: State,
{
    type View = El::View;

    type Components = El::Components;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let mut node = self.element.render(state, env, context);
        node.env = Some(Rc::new(self.value));
        node
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let result = self.element.update(scope, state, env, context);
        *scope.env = Some(Rc::new(self.value));
        result
    }
}

impl<El, T, S, E> ElementSeq<S, E> for Provide<El, T>
where
    El: Element<S, E>,
    T: 'static,
    S: State,
{
    type Storage =
        ViewNode<El::View, El::Components, S, E>;

    fn render_children(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        self.render(state, env, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.scope(), state, env, context)
    }
}

pub struct Consume<El, T> {
    render: fn(&T) -> El,
    _phantom: PhantomData<T>,
}

impl<El, T> Consume<El, T> {
    pub const fn new(render: fn(&T) -> El) -> ComponentElement<Self> {
        let connect = Self {
            render,
            _phantom: PhantomData,
        };
        ComponentElement::new(connect)
    }
}

impl<El, T> Clone for Consume<El, T> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<El, T, S, E> Component<S, E> for Consume<El, T>
where
    El: Element<S, E>,
    T: 'static,
    S: State,
{
    type Element = AsElement<Self>;

    fn render(&self) -> Self::Element {
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

impl<El, T, S, E> Element<S, E> for AsElement<Consume<El, T>>
where
    El: Element<S, E>,
    T: 'static,
    S: State,
{
    type View = El::View;

    type Components = El::Components;

    fn render(
        self,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> ViewNode<Self::View, Self::Components, S, E> {
        let value = context.get_env::<T>().unwrap();
        let element = (self.inner.render)(value);
        element.render(state, env, context)
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let value = context.get_env::<T>().unwrap();
        let element = (self.inner.render)(value);
        element.update(scope, state, env, context)
    }
}

impl<El, T, S, E> ElementSeq<S, E> for AsElement<Consume<El, T>>
where
    El: Element<S, E>,
    T: 'static,
    S: State,
{
    type Storage = ViewNode<El::View, El::Components, S, E>;

    fn render_children(self, state: &S, env: &E, context: &mut RenderContext) -> Self::Storage {
        self.render(state, env, context)
    }

    fn update_children(
        self,
        storage: &mut Self::Storage,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        self.update(&mut storage.scope(), state, env, context)
    }
}
