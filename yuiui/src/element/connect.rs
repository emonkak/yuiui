use std::marker::PhantomData;

use crate::component::Component;
use crate::context::RenderContext;
use crate::event::{EventResult, Lifecycle};
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};
use crate::Effect;

use super::{ComponentElement, Element, ElementSeq};

pub struct Connect<E, S> {
    render: fn(&S) -> E,
    _phantom: PhantomData<S>,
}

impl<E, S> Connect<E, S> {
    pub const fn new(render: fn(&S) -> E) -> ComponentElement<Self> {
        let connect = Self {
            render,
            _phantom: PhantomData,
        };
        ComponentElement::new(connect)
    }
}

impl<E, S> Clone for Connect<E, S> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<E, S, B> Component<S, B> for Connect<E, S>
where
    E: Element<S, B>,
    S: State,
{
    type Element = AsElement<Self>;

    fn lifecycle(&self, lifecycle: Lifecycle<&Self>, _state: &S, _backend: &B) -> EventResult<S> {
        match lifecycle {
            Lifecycle::Mounted => EventResult::from(Effect::SubscribeState),
            Lifecycle::Unmounted => EventResult::from(Effect::UnsubscribeState),
            _ => EventResult::nop(),
        }
    }

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

impl<E, S, B> Element<S, B> for AsElement<Connect<E, S>>
where
    E: Element<S, B>,
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
        let element = (self.inner.render)(state);
        element.render(state, backend, context)
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, B>,
        state: &S,
        backend: &B,
        context: &mut RenderContext,
    ) -> bool {
        let element = (self.inner.render)(state);
        element.update(scope, state, backend, context)
    }
}

impl<E, S, B> ElementSeq<S, B> for AsElement<Connect<E, S>>
where
    E: Element<S, B>,
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
