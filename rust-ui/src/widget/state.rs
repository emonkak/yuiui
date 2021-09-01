use std::any::Any;
use std::marker::PhantomData;

use super::paint_object::{PaintObject, PaintObjectProxy, PolyPaintObject};

#[derive(Debug)]
pub enum State<Renderer> {
    PureState(Box<dyn Any>),
    PaintObject(Box<PolyPaintObject<Renderer>>),
}

impl<Renderer> State<Renderer>
where
    Renderer: 'static,
{
    #[inline]
    pub fn as_any(&self) -> &dyn Any {
        match self {
            Self::PureState(state) => &**state,
            Self::PaintObject(paint_object) => paint_object.as_any(),
        }
    }

    #[inline]
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        match self {
            Self::PureState(state) => &mut **state,
            Self::PaintObject(paint_object) => paint_object.as_any_mut(),
        }
    }
}

#[derive(Debug)]
pub struct StateContainer<R, W: ?Sized, S: ?Sized, M: ?Sized> {
    state: State<R>,
    widget_type: PhantomData<W>,
    state_type: PhantomData<S>,
    message_type: PhantomData<M>,
}

impl<R, W, S, M> StateContainer<R, W, S, M>
where
    R: 'static,
    W: ?Sized,
    S: ?Sized,
    M: ?Sized,
{
    #[inline]
    pub fn from_pure_state(state: S) -> Self
    where
        S: 'static + Sized,
    {
        Self {
            state: State::PureState(Box::new(state)),
            widget_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        }
    }

    #[inline]
    pub fn from_paint_object(paint_object: S) -> Self
    where
        R: 'static,
        W: 'static + Sized,
        S: 'static + PaintObject<R, Widget = W, Message = M> + Sized,
        M: 'static + Sized,
    {
        Self {
            state: State::PaintObject(Box::new(PaintObjectProxy::new(paint_object))),
            widget_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn polymorphize(self) -> StateContainer<R, dyn Any, dyn Any, dyn Any + Send> {
        StateContainer {
            state: self.state,
            widget_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        }
    }
}

impl<R, W, S, M> From<StateContainer<R, W, S, M>> for State<R>
where
    W: ?Sized,
    S: ?Sized,
    M: ?Sized,
{
    fn from(container: StateContainer<R, W, S, M>) -> Self {
        container.state
    }
}
