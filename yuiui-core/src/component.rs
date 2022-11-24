use std::fmt;
use std::marker::PhantomData;

use crate::context::RenderContext;
use crate::element::{ComponentElement, Element, MemoizeElement};

pub trait Component<S, M, E>: Sized {
    type Element: Element<S, M, E>;

    fn render(&self, context: &mut RenderContext<S>) -> Self::Element;

    #[inline]
    fn el(self) -> ComponentElement<Self> {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, Element, S, M, E, RenderFn>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    props: Props,
    render_fn: RenderFn,
    _phantom: PhantomData<(Element, S, M, E)>,
}

impl<Props, Element, S, M, E, RenderFn> FunctionComponent<Props, Element, S, M, E, RenderFn>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    #[inline]
    pub const fn new(props: Props, render_fn: RenderFn) -> Self {
        Self {
            props,
            render_fn,
            _phantom: PhantomData,
        }
    }
}

impl<Props, Element, S, M, E, RenderFn> Component<S, M, E>
    for FunctionComponent<Props, Element, S, M, E, RenderFn>
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    type Element = Element;

    #[inline]
    fn render(&self, context: &mut RenderContext<S>) -> Self::Element {
        (self.render_fn)(&self.props, context)
    }
}

impl<Props, Element, S, M, E, RenderFn> AsRef<Props>
    for FunctionComponent<Props, Element, S, M, E, RenderFn>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    #[inline]
    fn as_ref(&self) -> &Props {
        &self.props
    }
}

impl<Props, Element, S, M, E, RenderFn> fmt::Debug
    for FunctionComponent<Props, Element, S, M, E, RenderFn>
where
    Props: fmt::Debug,
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponent")
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}

pub trait HigherOrderComponent<Props, S, M, E> {
    type Component: Component<S, M, E>;

    fn build(self, props: Props) -> Self::Component;

    #[inline]
    fn el(self, props: Props) -> ComponentElement<Self::Component>
    where
        Self: Sized,
    {
        self.build(props).el()
    }

    #[inline]
    fn memoize(self, props: Props) -> MemoizeElement<Self, Props>
    where
        Self: Sized,
    {
        MemoizeElement::new(self, props)
    }
}

impl<Props, Element, S, M, E, RenderFn> HigherOrderComponent<Props, S, M, E> for RenderFn
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    type Component = FunctionComponent<Props, Element, S, M, E, RenderFn>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponent::new(props, self)
    }
}
