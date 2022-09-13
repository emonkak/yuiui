use std::fmt;

use crate::context::MessageContext;
use crate::element::{ComponentElement, Element};
use crate::event::Lifecycle;

pub trait Component<S, M, B> {
    type Element: Element<S, M, B>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _context: &mut MessageContext<M>,
        _state: &S,
        _backend: &mut B,
    ) {
    }

    fn render(&self, state: &S) -> Self::Element;

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, E, S, M, B> {
    props: Props,
    render: fn(&Props, &S) -> E,
    lifecycle: Option<fn(&Props, Lifecycle<&Props>, &mut MessageContext<M>, &S, &mut B)>,
}

impl<Props, E, S, M, B> FunctionComponent<Props, E, S, M, B> {
    pub fn new(props: Props, render: fn(&Props, &S) -> E) -> Self {
        Self {
            props,
            render,
            lifecycle: None,
        }
    }

    pub fn lifecycle(
        mut self,
        lifecycle: impl Into<Option<fn(&Props, Lifecycle<&Props>, &mut MessageContext<M>, &S, &mut B)>>,
    ) -> Self {
        self.lifecycle = lifecycle.into();
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
        lifecycle: Lifecycle<&Self>,
        context: &mut MessageContext<M>,
        state: &S,
        backend: &mut B,
    ) {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, lifecycle, context, state, backend)
        }
    }

    fn render(&self, state: &S) -> Self::Element {
        (self.render)(&self.props, state)
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
