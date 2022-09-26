use std::fmt;
use std::marker::PhantomData;

use crate::context::MessageContext;
use crate::element::{ComponentEl, Element};
use crate::event::Lifecycle;

pub trait Component<S, M, B>: Sized {
    type Element: Element<S, M, B>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _context: &mut MessageContext<M>,
        _state: &S,
        _backend: &mut B,
    ) {
    }

    fn render(&self, state: &S) -> Self::Element;

    fn el(self) -> ComponentEl<Self> {
        ComponentEl::new(self)
    }
}

pub trait HigherOrderComponent<Props, S, M, B> {
    type Component: Component<S, M, B>;

    fn into_component(self, props: Props) -> Self::Component;

    fn el(self) -> ComponentEl<Self::Component>
    where
        Self: Sized,
        Props: Default,
    {
        self.into_component(Props::default()).el()
    }

    fn el_with(self, props: Props) -> ComponentEl<Self::Component>
    where
        Self: Sized,
    {
        self.into_component(props).el()
    }
}

impl<Props, E, S, M, B, RenderFn> HigherOrderComponent<Props, S, M, B> for RenderFn
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
{
    type Component = FunctionComponent<
        Props,
        E,
        S,
        M,
        B,
        RenderFn,
        fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
    >;

    fn into_component(self, props: Props) -> Self::Component {
        FunctionComponent::new(
            props,
            self,
            |_props, _lifecycle, _context, _state, _backend| {},
        )
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> HigherOrderComponent<Props, S, M, B>
    for (RenderFn, LifeCycleFn)
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
{
    type Component = FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>;

    fn into_component(self, props: Props) -> Self::Component {
        FunctionComponent::new(props, self.0, self.1)
    }
}

pub struct FunctionComponent<
    Props,
    E,
    S,
    M,
    B,
    RenderFn = fn(&Props, &S) -> E,
    LifeCycleFn = fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
> where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
{
    props: Props,
    render_fn: RenderFn,
    lifecycle_fn: LifeCycleFn,
    _phantom: PhantomData<(E, S, M, B)>,
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn>
    FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
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

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> Component<S, M, B>
    for FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
    E: Element<S, M, B>,
{
    type Element = E;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<Self>,
        context: &mut MessageContext<M>,
        state: &S,
        backend: &mut B,
    ) {
        let lifecycle = lifecycle.map(|component| component.props);
        (self.lifecycle_fn)(&self.props, lifecycle, context, state, backend)
    }

    fn render(&self, state: &S) -> Self::Element {
        (self.render_fn)(&self.props, state)
    }
}

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> fmt::Debug
    for FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Fn(&Props, &S) -> E,
    LifeCycleFn: Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
    Props: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FunctionComponent")
            .field(&self.props)
            .finish()
    }
}
