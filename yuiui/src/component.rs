use std::fmt;

use crate::context::EffectContext;
use crate::effect::EffectOps;
use crate::element::{ComponentElement, Element};
use crate::event::Lifecycle;
use crate::state::State;

pub trait Component<S: State, B>: Sized {
    type Element: Element<S, B>;

    type State: Default;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _local_state: &mut Self::State,
        _context: &EffectContext,
        _state: &S,
        _backend: &B,
    ) -> EffectOps<S> {
        EffectOps::nop()
    }

    fn render(&self, local_state: &Self::State, state: &S, backend: &B) -> Self::Element;

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, LocalState, E, S: State, B> {
    props: Props,
    render: fn(&Props, &LocalState, &S, &B) -> E,
    lifecycle: Option<
        fn(&Props, Lifecycle<&Props>, &mut LocalState, &EffectContext, &S, &B) -> EffectOps<S>,
    >,
}

impl<Props, LocalState, E, S, B> FunctionComponent<Props, LocalState, E, S, B>
where
    S: State,
{
    pub fn new(props: Props, render: fn(&Props, &LocalState, &S, &B) -> E) -> Self {
        Self {
            props,
            render,
            lifecycle: None,
        }
    }

    pub fn lifecycle(
        mut self,
        lifecycle: impl Into<
            Option<
                fn(
                    &Props,
                    Lifecycle<&Props>,
                    &mut LocalState,
                    &EffectContext,
                    &S,
                    &B,
                ) -> EffectOps<S>,
            >,
        >,
    ) -> Self {
        self.lifecycle = lifecycle.into();
        self
    }
}

impl<Props, LocalState, E, S, B> Component<S, B> for FunctionComponent<Props, LocalState, E, S, B>
where
    LocalState: Default,
    E: Element<S, B>,
    S: State,
{
    type Element = E;

    type State = LocalState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        local_state: &mut Self::State,
        context: &EffectContext,
        state: &S,
        backend: &B,
    ) -> EffectOps<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, lifecycle, local_state, context, state, backend)
        } else {
            EffectOps::nop()
        }
    }

    fn render(&self, local_state: &Self::State, state: &S, backend: &B) -> Self::Element {
        (self.render)(&self.props, local_state, state, backend)
    }
}

impl<Props, LocalState, E, S, B> fmt::Debug for FunctionComponent<Props, LocalState, E, S, B>
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
