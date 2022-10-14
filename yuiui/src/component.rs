use std::fmt;
use std::marker::PhantomData;

use crate::element::{ComponentElement, Element};
use crate::event::Lifecycle;
use crate::id::IdContext;
use crate::store::Store;
use crate::view_node::ViewNodeMut;

pub trait Component<S, M, B>: Sized {
    type Element: Element<S, M, B>;

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<
            '_,
            <Self::Element as Element<S, M, B>>::View,
            <Self::Element as Element<S, M, B>>::Components,
            S,
            M,
            B,
        >,
        _id_context: &mut IdContext,
        _store: &Store<S>,
        _messages: &mut Vec<M>,
        _backend: &mut B,
    ) {
    }

    fn render(&self, state: &S) -> Self::Element;

    #[inline]
    fn el(self) -> ComponentElement<Self> {
        ComponentElement::new(self)
    }
}

pub trait HigherOrderComponent<Props, S, M, B> {
    type Component: Component<S, M, B>;

    fn build(self, props: Props) -> Self::Component;

    #[inline]
    fn el(self) -> ComponentElement<Self::Component>
    where
        Self: Sized,
        Props: Default,
    {
        self.build(Props::default()).el()
    }

    #[inline]
    fn el_with(self, props: Props) -> ComponentElement<Self::Component>
    where
        Self: Sized,
    {
        self.build(props).el()
    }
}

impl<Props, E, S, M, B, RenderFn> HigherOrderComponent<Props, S, M, B> for RenderFn
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
{
    type Component = FunctionComponentInstance<Props, E, S, M, B, RenderFn>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponentInstance::new(
            props,
            self,
            |_props, _lifecycle, _id_context, _store, _messages, _backend| {},
        )
    }
}

pub struct FunctionComponent<
    Props,
    E,
    S,
    M,
    B,
    RenderFn = fn(&Props, &S) -> E,
    LifeCycleFn = fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
> where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(Props, E, S, M, B)>,
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn>
    FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    pub const fn new(render_fn: RenderFn, lifecycle_fn: LifeCycleFn) -> Self {
        Self {
            render_fn,
            lifecycle_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> HigherOrderComponent<Props, S, M, B>
    for FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    type Component = FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponentInstance::new(props, self.render_fn, self.lifecycle_fn)
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponent").finish_non_exhaustive()
    }
}

pub struct FunctionComponentInstance<
    Props,
    E,
    S,
    M,
    B,
    RenderFn = fn(&Props, &S) -> E,
    LifeCycleFn = fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
> where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    props: Props,
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(E, S, M, B)>,
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn>
    FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    pub const fn new(props: Props, render_fn: RenderFn, lifecycle_fn: LifeCycleFn) -> Self {
        Self {
            props,
            render_fn,
            lifecycle_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> Component<S, M, B>
    for FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    type Element = E;

    #[inline]
    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<
            '_,
            <Self::Element as Element<S, M, B>>::View,
            <Self::Element as Element<S, M, B>>::Components,
            S,
            M,
            B,
        >,
        id_context: &mut IdContext,
        store: &Store<S>,
        messages: &mut Vec<M>,
        backend: &mut B,
    ) {
        let lifecycle = lifecycle.map(|component| component.props);
        (self.lifecycle_fn)(&self.props, lifecycle, id_context, store, messages, backend)
    }

    #[inline]
    fn render(&self, state: &S) -> Self::Element {
        (self.render_fn)(&self.props, state)
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> Clone
    for FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    Props: Clone,
    RenderFn: Clone + Fn(&Props, &S) -> E,
    LifeCycleFn:
        Clone + Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            render_fn: self.render_fn.clone(),
            lifecycle_fn: self.lifecycle_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> AsRef<Props>
    for FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    fn as_ref(&self) -> &Props {
        &self.props
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponentInstance<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut IdContext, &Store<S>, &mut Vec<M>, &mut B),
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponentInstance")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}
