use std::marker::PhantomData;

use crate::component::Component;
use crate::context::RenderContext;
use crate::element::{ComponentElement, Element, ElementSeq};
use crate::event::{EventResult, Lifecycle};
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};
use crate::Effect;

pub struct Connect<El, S> {
    render: fn(&S) -> El,
    state_type: PhantomData<S>,
}

impl<El, S> Connect<El, S> {
    pub const fn new(render: fn(&S) -> El) -> ComponentElement<Self> {
        let connect = Self {
            render,
            state_type: PhantomData,
        };
        ComponentElement::new(connect)
    }
}

impl<El, S> Clone for Connect<El, S> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            state_type: PhantomData,
        }
    }
}

impl<El, S, E> Component<S, E> for Connect<El, S>
where
    El: Element<S, E>,
    S: State,
{
    type Element = AsElement<Self>;

    fn lifecycle(&self, lifecycle: Lifecycle<&Self>, _state: &S, _env: &E) -> EventResult<S> {
        match lifecycle {
            Lifecycle::Mounted => EventResult::from(Effect::SubscribeState),
            Lifecycle::Unmounted => EventResult::from(Effect::UnsubscribeState),
            _ => EventResult::nop(),
        }
    }

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

impl<El, S, E> Element<S, E> for AsElement<Connect<El, S>>
where
    El: Element<S, E>,
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
        let element = (self.inner.render)(state);
        Element::render(element, state, env, context)
    }

    fn update(
        self,
        scope: ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let element = (self.inner.render)(state);
        Element::update(element, scope, state, env, context)
    }
}

impl<El, S, E> ElementSeq<S, E> for AsElement<Connect<El, S>>
where
    El: Element<S, E>,
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
        self.update(storage.scope(), state, env, context)
    }
}
