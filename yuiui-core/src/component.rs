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

pub struct FunctionComponent<RenderFn, Props, Element, S, M, E>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
    Element: self::Element<S, M, E>,
{
    render_fn: RenderFn,
    props: Props,
    _phantom: PhantomData<(S, M, E)>,
}

impl<RenderFn, Props, Element, S, M, E> FunctionComponent<RenderFn, Props, Element, S, M, E>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
    Element: self::Element<S, M, E>,
{
    pub const fn new(render_fn: RenderFn, props: Props) -> Self {
        Self {
            render_fn,
            props,
            _phantom: PhantomData,
        }
    }
}

impl<RenderFn, Props, Element, S, M, E> Component<S, M, E>
    for FunctionComponent<RenderFn, Props, Element, S, M, E>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
    Element: self::Element<S, M, E>,
{
    type Element = Element;

    #[inline]
    fn render(&self, context: &mut RenderContext<S>) -> Self::Element {
        (self.render_fn)(&self.props, context)
    }
}

impl<RenderFn, Props, Element, S, M, E> AsRef<Props>
    for FunctionComponent<RenderFn, Props, Element, S, M, E>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
    Element: self::Element<S, M, E>,
{
    #[inline]
    fn as_ref(&self) -> &Props {
        &self.props
    }
}

impl<RenderFn, Props, Element, S, M, E> fmt::Debug
    for FunctionComponent<RenderFn, Props, Element, S, M, E>
where
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
    Props: fmt::Debug,
    Element: self::Element<S, M, E>,
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

impl<RenderFn, Props, Element, S, M, E> HigherOrderComponent<Props, S, M, E> for RenderFn
where
    Element: self::Element<S, M, E>,
    RenderFn: Fn(&Props, &mut RenderContext<S>) -> Element,
{
    type Component = FunctionComponent<RenderFn, Props, Element, S, M, E>;

    #[inline]
    fn build(self, props: Props) -> Self::Component {
        FunctionComponent::new(self, props)
    }
}
