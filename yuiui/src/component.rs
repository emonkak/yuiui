use std::fmt;
use std::marker::PhantomData;

use crate::context::MessageContext;
use crate::element::{ComponentEl, Element};
use crate::event::Lifecycle;
use crate::store::Store;
use crate::view_node::ViewNodeMut;

pub trait Component<S, M, R>: Sized {
    type Element: Element<S, M, R>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<
            '_,
            <Self::Element as Element<S, M, R>>::View,
            <Self::Element as Element<S, M, R>>::Components,
            S,
            M,
            R,
        >,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) {
    }

    fn render(&self, state: &S) -> Self::Element;

    fn el(self) -> ComponentEl<Self> {
        ComponentEl::new(self)
    }
}

pub trait ComponentProps<S, M, R>: Sized {
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _renderer: &mut R,
    ) {
    }
}

impl<S, M, R> ComponentProps<S, M, R> for () {}

pub trait HigherOrderComponent<Props, S, M, R> {
    type Component: Component<S, M, R>;

    fn build(self, props: Props) -> Self::Component;

    fn el(self) -> ComponentEl<Self::Component>
    where
        Self: Sized,
        Props: Default,
    {
        self.build(Props::default()).el()
    }

    fn el_with(self, props: Props) -> ComponentEl<Self::Component>
    where
        Self: Sized,
    {
        self.build(props).el()
    }
}

impl<Props, E, S, M, R, RenderFn> HigherOrderComponent<Props, S, M, R> for RenderFn
where
    E: Element<S, M, R>,
    RenderFn: Fn(&Props, &S) -> E,
    Props: ComponentProps<S, M, R>,
{
    type Component = FunctionComponent<Props, E, S, M, R, RenderFn>;

    fn build(self, props: Props) -> Self::Component {
        FunctionComponent::new(props, self, Props::lifecycle)
    }
}

pub struct FunctionComponent<
    Props,
    E,
    S,
    M,
    R,
    RenderFn = fn(&Props, &S) -> E,
    LifeCycleFn = fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
> where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
{
    props: Props,
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(E, S, M, R)>,
}

impl<Props, E, S, M, R, RenderFn, LifeCycleFn>
    FunctionComponent<Props, E, S, M, R, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
{
    pub fn new(props: Props, render_fn: RenderFn, lifecycle_fn: LifeCycleFn) -> Self {
        Self {
            props,
            render_fn,
            lifecycle_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, E, S, M, R, RenderFn, LifeCycleFn> Component<S, M, R>
    for FunctionComponent<Props, E, S, M, R, RenderFn, LifeCycleFn>
where
    E: Element<S, M, R>,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
{
    type Element = E;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        _view_node: ViewNodeMut<
            '_,
            <Self::Element as Element<S, M, R>>::View,
            <Self::Element as Element<S, M, R>>::Components,
            S,
            M,
            R,
        >,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        renderer: &mut R,
    ) {
        let lifecycle = lifecycle.map(|component| component.props);
        (self.lifecycle_fn)(&self.props, lifecycle, context, store, renderer)
    }

    fn render(&self, state: &S) -> Self::Element {
        (self.render_fn)(&self.props, state)
    }
}

impl<Props, E, S, M, R, RenderFn, LifeCycleFn> Clone
    for FunctionComponent<Props, E, S, M, R, RenderFn, LifeCycleFn>
where
    Props: Clone,
    RenderFn: Clone + Fn(&Props, &S) -> E,
    LifeCycleFn: Clone + Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
{
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            render_fn: self.render_fn.clone(),
            lifecycle_fn: self.lifecycle_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Props, E, S, M, R, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponent<Props, E, S, M, R, RenderFn, LifeCycleFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &Store<S>, &mut R),
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FunctionComponent")
            .field(&self.props)
            .finish()
    }
}
