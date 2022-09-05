use std::fmt;
use std::marker::PhantomData;

use crate::element::{ComponentElement, Element};
use crate::event::{EventResult, Lifecycle};
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type LocalState;

    type Element: Element<S, E>;

    fn initial_state(&self, state: &S, env: &E) -> Self::LocalState;

    fn should_update(
        &self,
        _other: &Self,
        _local_state: &Self::LocalState,
        _state: &S,
        _env: &E,
    ) -> bool {
        true
    }

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

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, LocalState, El, S: State, E> {
    pub props: Props,
    pub initial_state: fn(&Props, &S, &E) -> LocalState,
    pub should_update: Option<fn(&Props, &Props, &LocalState, &S, &E) -> bool>,
    pub lifecycle: Option<fn(&Props, Lifecycle<&Props>, &mut LocalState, &S, &E) -> EventResult<S>>,
    pub render: fn(&Props, &LocalState, &S, &E) -> El,
}

impl<Props, LocalState, El, S, E> Component<S, E> for FunctionComponent<Props, LocalState, El, S, E>
where
    LocalState: Default,
    El: Element<S, E>,
    S: State,
{
    type Element = El;

    type LocalState = LocalState;

    fn initial_state(&self, state: &S, env: &E) -> Self::LocalState {
        (self.initial_state)(&self.props, state, env)
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

    fn render(&self, local_state: &Self::LocalState, state: &S, env: &E) -> Self::Element {
        (self.render)(&self.props, local_state, state, env)
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

pub struct Memoize<Render, Dependence, El, S, E> {
    render: Render,
    dependence: Dependence,
    _phantom: PhantomData<(El, S, E)>,
}

impl<Render, Dependence, El, S, E> Memoize<Render, Dependence, El, S, E>
where
    Render: Fn(&S, &E) -> El,
    Dependence: PartialEq,
    El: Element<S, E>,
    S: State,
{
    pub fn new(render: Render, dependence: Dependence) -> Self {
        Self {
            render,
            dependence,
            _phantom: PhantomData,
        }
    }
}

impl<Render, Dependence, El, S, E> Component<S, E> for Memoize<Render, Dependence, El, S, E>
where
    Render: Fn(&S, &E) -> El,
    Dependence: PartialEq,
    El: Element<S, E>,
    S: State,
{
    type Element = El;

    type LocalState = ();

    fn initial_state(&self, _state: &S, _env: &E) -> Self::LocalState {
        ()
    }

    fn should_update(
        &self,
        other: &Self,
        _local_state: &Self::LocalState,
        _state: &S,
        _env: &E,
    ) -> bool {
        self.dependence != other.dependence
    }

    fn render(&self, _local_state: &Self::LocalState, state: &S, env: &E) -> Self::Element {
        (self.render)(state, env)
    }
}

impl<Render, Dependence, El, S, E> fmt::Debug for Memoize<Render, Dependence, El, S, E>
where
    Dependence: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Memoize")
            .field("dependence", &self.dependence)
            .finish_non_exhaustive()
    }
}
