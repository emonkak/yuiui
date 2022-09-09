use std::fmt;

use crate::context::MessageContext;
use crate::element::{ComponentElement, Element};
use crate::event::Lifecycle;
use crate::state::Store;

pub trait Component<S, M, B>: Sized {
    type Element: Element<S, M, B>;

    type State: Default;

    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _local_state: &mut Self::State,
        _context: &mut MessageContext<M>,
        _store: &Store<S>,
        _backend: &B,
    ) {
    }

    fn render(&self, local_state: &Self::State, store: &Store<S>, backend: &B) -> Self::Element;

    fn el(self) -> ComponentElement<Self>
    where
        Self: Sized,
    {
        ComponentElement::new(self)
    }
}

pub struct FunctionComponent<Props, LocalState, E, S, M, B> {
    props: Props,
    render: fn(&Props, &LocalState, &S, &B) -> E,
    lifecycle:
        Option<fn(&Props, Lifecycle<&Props>, &mut LocalState, &mut MessageContext<M>, &S, &B)>,
}

impl<Props, LocalState, E, S, M, B> FunctionComponent<Props, LocalState, E, S, M, B> {
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
            Option<fn(&Props, Lifecycle<&Props>, &mut LocalState, &mut MessageContext<M>, &S, &B)>,
        >,
    ) -> Self {
        self.lifecycle = lifecycle.into();
        self
    }
}

impl<Props, LocalState, E, S, M, B> Component<S, M, B>
    for FunctionComponent<Props, LocalState, E, S, M, B>
where
    LocalState: Default,
    E: Element<S, M, B>,
{
    type Element = E;

    type State = LocalState;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&Self>,
        local_state: &mut Self::State,
        context: &mut MessageContext<M>,
        store: &Store<S>,
        backend: &B,
    ) {
        if let Some(lifecycle_fn) = &self.lifecycle {
            let lifecycle = lifecycle.map(|component| &component.props);
            lifecycle_fn(&self.props, lifecycle, local_state, context, store, backend)
        }
    }

    fn render(&self, local_state: &Self::State, store: &Store<S>, backend: &B) -> Self::Element {
        (self.render)(&self.props, local_state, store, backend)
    }
}

impl<Props, LocalState, E, S, M, B> fmt::Debug for FunctionComponent<Props, LocalState, E, S, M, B>
where
    Props: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FunctionComponent")
            .field(&self.props)
            .finish()
    }
}
