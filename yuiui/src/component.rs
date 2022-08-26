use std::fmt;

use crate::element::{ComponentElement, Element};
use crate::event::EventResult;
use crate::state::State;

pub trait Component<S: State, E>: Sized {
    type Element: Element<S, E>;

    fn render(&self, state: &S, env: &E) -> Self::Element;

    fn should_update(&self, _other: &Self, _state: &S, _env: &E) -> bool {
        true
    }

    fn lifecycle(
        &self,
        _lifecycle: ComponentLifecycle<Self>,
        _state: &S,
        _env: &E,
    ) -> EventResult<S> {
        EventResult::nop()
    }

    fn el(self) -> ComponentElement<Self, S, E>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, El, S: State, E> {
    pub props: Props,
    pub render: fn(&Props, &S, &E) -> El,
    pub should_update: Option<fn(&Props, &Props, &S, &E) -> bool>,
    pub lifecycle: Option<fn(&Props, ComponentLifecycle<Props>, &S, &E) -> EventResult<S>>,
}

impl<Props, El, S, E> Component<S, E> for FunctionComponent<Props, El, S, E>
where
    El: Element<S, E>,
    S: State,
{
    type Element = El;

    fn render(&self, state: &S, env: &E) -> Self::Element {
        (self.render)(&self.props, state, env)
    }

    fn should_update(&self, other: &Self, state: &S, env: &E) -> bool {
        if let Some(should_update_fn) = &self.should_update {
            should_update_fn(&self.props, &other.props, state, env)
        } else {
            true
        }
    }

    fn lifecycle(&self, lifecycle: ComponentLifecycle<Self>, state: &S, env: &E) -> EventResult<S> {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map_component(|component| component.props);
            lifecycle_fn(&self.props, lifecycle, state, env)
        } else {
            EventResult::nop()
        }
    }
}

impl<Props, El, S, E> fmt::Debug for FunctionComponent<Props, El, S, E>
where
    Props: fmt::Debug,
    S: State,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionComponent")
            .field("props", &self.props)
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
