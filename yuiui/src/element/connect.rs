use std::marker::PhantomData;

use crate::component::Component;
use crate::context::RenderContext;
use crate::event::{EventResult, Lifecycle};
use crate::state::State;
use crate::view_node::{ViewNode, ViewNodeScope};
use crate::Effect;

use super::{ComponentElement, Element, ElementSeq};

pub struct Connect<El, S> {
    render: fn(&S) -> El,
    _phantom: PhantomData<S>,
}

impl<El, S> Connect<El, S> {
    pub const fn new(render: fn(&S) -> El) -> ComponentElement<Self> {
        let connect = Self {
            render,
            _phantom: PhantomData,
        };
        ComponentElement::new(connect)
    }
}

impl<El, S> Clone for Connect<El, S> {
    fn clone(&self) -> Self {
        Self {
            render: self.render.clone(),
            _phantom: PhantomData,
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
        element.render(state, env, context)
    }

    fn update(
        self,
        scope: &mut ViewNodeScope<Self::View, Self::Components, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool {
        let element = (self.inner.render)(state);
        element.update(scope, state, env, context)
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
        self.update(&mut storage.scope(), state, env, context)
    }
}
