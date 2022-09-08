use std::fmt;

use crate::context::EffectContext;
use crate::element::{ComponentElement, Element};
use crate::event::{EventResult, Lifecycle};
use crate::state::State;

pub trait Component<S: State, B>: Sized {
    type Element: Element<S, B>;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _context: &EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn render(&self, state: &S, backend: &B) -> Self::Element;

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, E, S: State, B> {
    props: Props,
    render: fn(&Props, &S, &B) -> E,
    lifecycle: Option<fn(&Props, Lifecycle<&Props>, &EffectContext, &S, &B) -> EventResult<S>>,
}

impl<Props, E, S, B> FunctionComponent<Props, E, S, B>
where
    S: State,
{
    pub fn new(props: Props, render: fn(&Props, &S, &B) -> E) -> Self {
        Self {
            props,
            render,
            lifecycle: None,
        }
    }

    pub fn lifecycle(
        mut self,
        lifecycle: impl Into<
            Option<fn(&Props, Lifecycle<&Props>, &EffectContext, &S, &B) -> EventResult<S>>,
        >,
    ) -> Self {
        self.lifecycle = lifecycle.into();
        self
    }
}

impl<Props, E, S, B> Component<S, B> for FunctionComponent<Props, E, S, B>
where
    E: Element<S, B>,
    S: State,
{
    type Element = E;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        context: &EffectContext,
        state: &S,
        backend: &B,
    ) -> EventResult<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, lifecycle, context, state, backend)
        } else {
            EventResult::nop()
        }
    }

    fn render(&self, state: &S, backend: &B) -> Self::Element {
        (self.render)(&self.props, state, backend)
    }
}

impl<Props, E, S, B> fmt::Debug for FunctionComponent<Props, E, S, B>
where
    Props: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FunctionComponent")
            .field(&self.props)
            .finish()
    }
}
