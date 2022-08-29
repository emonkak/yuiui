use std::fmt;

use crate::element::{ComponentElement, Element};
use crate::event::EventResult;
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type Element: Element<S, E>;

    type LocalState: Default;

    fn lifecycle(
        &self,
        _lifecycle: ComponentLifecycle<Self>,
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
    pub lifecycle:
        Option<fn(&Props, ComponentLifecycle<Props>, &mut LocalState, &S, &E) -> EventResult<S>>,
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

    fn should_update(&self, other: &Self, local_state: &Self::LocalState, state: &S, env: &E) -> bool {
        if let Some(should_update_fn) = &self.should_update {
            should_update_fn(&self.props, &other.props, local_state, state, env)
        } else {
            true
        }
    }

    fn lifecycle(
        &self,
        lifecycle: ComponentLifecycle<Self>,
        local_state: &mut Self::LocalState,
        state: &S,
        env: &E,
    ) -> EventResult<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map_component(|component| component.props);
            lifecycle_fn(&self.props, lifecycle, local_state, state, env)
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

#[derive(Debug)]
pub enum ComponentLifecycle<C> {
    Mounted,
    Updated(C),
    Unmounted,
}

impl<C> ComponentLifecycle<C> {
    pub fn map_component<F, D>(self, f: F) -> ComponentLifecycle<D>
    where
        F: FnOnce(C) -> D,
    {
        match self {
            Self::Mounted => ComponentLifecycle::Mounted,
            Self::Updated(component) => ComponentLifecycle::Updated(f(component)),
            Self::Unmounted => ComponentLifecycle::Unmounted,
        }
    }
}
