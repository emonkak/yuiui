use std::fmt;

use crate::element::{ComponentElement, Element};
use crate::event::{EventResult, Lifecycle};
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type Element: Element<S, E>;

    fn lifecycle(&self, _lifecycle: Lifecycle<&Self>, _state: &S, _env: &E) -> EventResult<S> {
        EventResult::nop()
    }

    fn render(&self) -> Self::Element;

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, El, S: State, E> {
    props: Props,
    render: fn(&Props) -> El,
    lifecycle: Option<fn(&Props, Lifecycle<&Props>, &S, &E) -> EventResult<S>>,
}

impl<Props, El, S, E> FunctionComponent<Props, El, S, E>
where
    S: State,
{
    pub fn new(props: Props, render: fn(&Props) -> El) -> Self {
        Self {
            props,
            render,
            lifecycle: None,
        }
    }

    pub fn lifecycle(
        mut self,
        lifecycle: impl Into<Option<fn(&Props, Lifecycle<&Props>, &S, &E) -> EventResult<S>>>,
    ) -> Self {
        self.lifecycle = lifecycle.into();
        self
    }
}

impl<Props, El, S, E> Component<S, E> for FunctionComponent<Props, El, S, E>
where
    El: Element<S, E>,
    S: State,
{
    type Element = El;

    fn lifecycle(&self, lifecycle: Lifecycle<&Self>, state: &S, env: &E) -> EventResult<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let sub_lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, sub_lifecycle, state, env)
        } else {
            EventResult::nop()
        }
    }

    fn render(&self) -> Self::Element {
        (self.render)(&self.props)
    }
}

impl<Props, El, S, E> PartialEq for FunctionComponent<Props, El, S, E>
where
    Props: PartialEq,
    S: State,
{
    fn eq(&self, other: &Self) -> bool {
        self.props == other.props
            && &self.render as *const _ == &other.render as *const _
            && self.lifecycle.map(|x| &x as *const _) == other.lifecycle.map(|y| &y as *const _)
    }
}

impl<Props, El, S, E> fmt::Debug for FunctionComponent<Props, El, S, E>
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
