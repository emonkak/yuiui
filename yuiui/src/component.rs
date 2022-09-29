use std::fmt;
use std::marker::PhantomData;

use crate::context::MessageContext;
use crate::element::{ComponentEl, Element};
use crate::event::Lifecycle;
use crate::view_node::ViewRef;

pub trait Component<S, M, B>: Sized {
    type Element: Element<S, M, B>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _view_ref: ViewRef<'_, <Self::Element as Element<S, M, B>>::View, S, M, B>,
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

pub trait ComponentProps<S, M, B>: Sized {
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<Self>,
        _context: &mut MessageContext<M>,
        _state: &S,
        _backend: &mut B,
    ) {
    }
}

impl<S, M, B> ComponentProps<S, M, B> for () {}

pub trait HigherOrderComponent<Props, S, M, B> {
    type Component: Component<S, M, B>;

    fn build_component(self, props: Props) -> Self::Component;

    fn el(self) -> ComponentEl<Self::Component>
    where
        Self: Sized,
        Props: Default,
    {
        self.build_component(Props::default()).el()
    }

    fn el_with(self, props: Props) -> ComponentEl<Self::Component>
    where
        Self: Sized,
    {
        self.build_component(props).el()
    }
}

impl<Props, E, S, M, B, RenderFn> HigherOrderComponent<Props, S, M, B> for RenderFn
where
    E: Element<S, M, B>,
    RenderFn: Fn(&Props, &S) -> E,
    Props: ComponentProps<S, M, B>,
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

    fn build_component(self, props: Props) -> Self::Component {
        FunctionComponent::new(props, self, Props::lifecycle)
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
        _view_ref: ViewRef<'_, <Self::Element as Element<S, M, B>>::View, S, M, B>,
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

impl<Props, E, S, M, B, RenderFn, LifeCycleFn> Clone
    for FunctionComponent<Props, E, S, M, B, RenderFn, LifeCycleFn>
where
    RenderFn: Clone + Fn(&Props, &S) -> E,
    LifeCycleFn: Clone + Fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
    Props: Clone,
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
