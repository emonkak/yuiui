use std::fmt;
use std::marker::PhantomData;
use std::mem;

use crate::element::{ComponentElement, Element};
use crate::event::{EventContext, EventResult};
use crate::sequence::CommitMode;
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
        EventResult::Nop
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
            EventResult::Nop
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
pub struct ComponentNode<C: Component<S, E>, S: State, E> {
    pub(crate) component: C,
    pub(crate) pending_component: Option<C>,
    pub(crate) state: PhantomData<S>,
    pub(crate) env: PhantomData<E>,
}

impl<C, S, E> ComponentNode<C, S, E>
where
    C: Component<S, E>,
    S: State,
{
    pub(crate) fn new(component: C) -> Self {
        Self {
            component,
            pending_component: None,
            state: PhantomData,
            env: PhantomData,
        }
    }

    pub(crate) fn commit(
        &mut self,
        mode: CommitMode,
        state: &S,
        env: &E,
        context: &mut EventContext<S>,
    ) {
        let lifecycle = match mode {
            CommitMode::Mount => ComponentLifecycle::Mounted,
            CommitMode::Update => {
                let old_component = mem::replace(
                    &mut self.component,
                    self.pending_component
                        .take()
                        .expect("get pending component"),
                );
                ComponentLifecycle::Updated(old_component)
            }
            CommitMode::Unmount => {
                context.dispose_node();
                ComponentLifecycle::Unmounted
            }
        };
        context.process_result(self.component.lifecycle(lifecycle, state, env));
        context.next_component();
    }
}

pub trait ComponentStack<S: State, E> {
    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EventContext<S>);
}

impl<C, CS, S, E> ComponentStack<S, E> for (ComponentNode<C, S, E>, CS)
where
    C: Component<S, E>,
    CS: ComponentStack<S, E>,
    S: State,
{
    fn commit(&mut self, mode: CommitMode, state: &S, env: &E, context: &mut EventContext<S>) {
        self.0.commit(mode, state, env, context);
        self.1.commit(mode, state, env, context);
    }
}

impl<S: State, E> ComponentStack<S, E> for () {
    fn commit(&mut self, _mode: CommitMode, _state: &S, _env: &E, _context: &mut EventContext<S>) {}
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
