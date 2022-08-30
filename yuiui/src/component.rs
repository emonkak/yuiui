use std::fmt;

use crate::element::{ComponentElement, Element};
use crate::event::{EventResult, Lifecycle};
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type Element: Element<S, E>;

    type LocalState: Default;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _local_state: &mut Self::LocalState,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn render(&self, local_state: &Self::LocalState, state: &S, env: &E) -> Self::Element;

    fn should_update(
        &self,
        _other: &Self,
        _local_state: &Self::LocalState,
        _state: &S,
        _env: &E,
    ) -> bool {
        true
    }

    fn el(self) -> ComponentElement<Self, S, E>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, LocalState, El, S: State, E> {
    pub props: Props,
    pub render: fn(&Props, &LocalState, &S, &E) -> El,
    pub should_update: Option<fn(&Props, &Props, &LocalState, &S, &E) -> bool>,
    pub lifecycle: Option<fn(&Props, Lifecycle<&Props>, &mut LocalState, &S, &E) -> EventResult<S>>,
}

impl<Props, LocalState, El, S, E> Component<S, E> for FunctionComponent<Props, LocalState, El, S, E>
where
    LocalState: Default,
    El: Element<S, E>,
    S: State,
{
    type Element = El;

    type LocalState = LocalState;

    fn render(&self, local_state: &Self::LocalState, state: &S, env: &E) -> Self::Element {
        (self.render)(&self.props, local_state, state, env)
    }

    fn should_update(
        &self,
        other: &Self,
        local_state: &Self::LocalState,
        state: &S,
        env: &E,
    ) -> bool {
        if let Some(should_update_fn) = &self.should_update {
            should_update_fn(&self.props, &other.props, local_state, state, env)
        } else {
            true
        }
    }

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        local_state: &mut Self::LocalState,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let sub_lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, sub_lifecycle, local_state, state, env)
        } else {
            EventResult::nop()
        }
    }
}

impl<Props, LocalState, El, S, E> fmt::Debug for FunctionComponent<Props, LocalState, El, S, E>
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
