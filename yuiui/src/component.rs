use std::fmt;

use crate::context::MessageContext;
use crate::element::{ComponentElement, Element};
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

    fn el(self) -> ComponentElement<Self> {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, E, S, M, B> {
    props: Props,
    render_fn: fn(&Props, &S) -> E,
    lifecycle_fn: Option<fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B)>,
}

impl<Props, E, S, M, B> FunctionComponent<Props, E, S, M, B> {
    pub fn new(props: Props, render_fn: fn(&Props, &S) -> E) -> Self {
        Self {
            props,
            render_fn,
            lifecycle_fn: None,
        }
    }

    pub fn lifecycle(
        mut self,
        lifecycle_fn: fn(&Props, Lifecycle<Props>, &mut MessageContext<M>, &S, &mut B),
    ) -> Self {
        self.lifecycle_fn = Some(lifecycle_fn);
        self
    }
}

impl<Props, E, S, M, B> Component<S, M, B> for FunctionComponent<Props, E, S, M, B>
where
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
        if let Some(lifecycle_fn) = self.lifecycle_fn {
            let lifecycle = lifecycle.map(|component| component.props);
            lifecycle_fn(&self.props, lifecycle, context, state, backend)
        }
    }

    fn render(&self, state: &S) -> Self::Element {
        (self.render_fn)(&self.props, state)
    }
}

impl<Props, E, S, M, B> Clone for FunctionComponent<Props, E, S, M, B>
where
    Props: Clone,
{
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            render_fn: self.render_fn.clone(),
            lifecycle_fn: self.lifecycle_fn.clone(),
        }
    }
}

impl<Props, E, S, M, B> fmt::Debug for FunctionComponent<Props, E, S, M, B>
where
    Props: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FunctionComponent")
            .field(&self.props)
            .finish()
    }
}
