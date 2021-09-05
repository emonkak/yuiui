use std::any::{self, Any, TypeId};
use std::fmt;
use std::marker::PhantomData;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle};
use crate::support::generator::Generator;

use super::element::{Children, ElementId};
use super::message::MessageSink;
use super::state::StateContainer;

pub type PolyWidget<Renderer> =
    dyn Widget<Renderer, dyn Any, State = dyn Any, Message = dyn Any + Send>;

pub trait Widget<Renderer, Own: ?Sized = Self>: WidgetSeal + Send + Sync {
    type State: ?Sized;

    type Message: ?Sized + Send;

    fn initial_state(&self) -> StateContainer<Renderer, Own, Self::State, Self::Message>;

    #[inline]
    fn should_render(&self, _other: &Own) -> bool {
        true
    }

    #[inline]
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Self::Message,
        _messages: &mut MessageSink,
    ) -> bool {
        true
    }

    #[inline]
    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        Vec::new()
    }

    #[inline]
    fn lifecycle(
        &self,
        _state: &mut Self::State,
        _lifecycle: Lifecycle<&Own>,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) {
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        children: Vec<ElementId>,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = children.first() {
                co.suspend(LayoutRequest::LayoutChild(*child_id, box_constraints))
                    .await
            } else {
                box_constraints.max
            }
        })
    }

    #[inline]
    fn draw(
        &self,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) -> Option<Primitive> {
        None
    }

    #[inline]
    fn message_type_id(&self) -> TypeId
    where
        Self::Message: 'static,
    {
        TypeId::of::<Self::Message>()
    }

    #[inline]
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    #[inline]
    fn short_type_name(&self) -> &'static str {
        let name = self.type_name();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }

    fn as_any(&self) -> &dyn Any;
}

impl<R> fmt::Debug for PolyWidget<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {{ .. }}", self.short_type_name())
    }
}

pub trait WidgetSeal {}

pub struct WidgetProxy<W, R, S, M> {
    pub widget: W,
    pub state_type: PhantomData<S>,
    pub message_type: PhantomData<M>,
    pub renderer_type: PhantomData<R>,
}

impl<W, R, S, M> WidgetProxy<W, R, S, M> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            renderer_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        }
    }
}

impl<W, R, S, M> Widget<R, dyn Any> for WidgetProxy<W, R, S, M>
where
    W: 'static + Widget<R, State = S, Message = M>,
    R: 'static,
    S: 'static,
    M: 'static,
{
    type State = dyn Any;
    type Message = dyn Any + Send;

    #[inline]
    fn should_render(&self, other: &dyn Any) -> bool {
        self.widget.should_render(other.downcast_ref().unwrap())
    }

    #[inline]
    fn initial_state(&self) -> StateContainer<R, dyn Any, Self::State, Self::Message> {
        self.widget.initial_state().polymorphize()
    }

    #[inline]
    fn update(
        &self,
        state: &mut Self::State,
        message: &Self::Message,
        messages: &mut MessageSink,
    ) -> bool {
        self.widget.update(
            state.downcast_mut().unwrap(),
            message.downcast_ref().unwrap(),
            messages,
        )
    }

    #[inline]
    fn render(&self, state: &Self::State, element_id: ElementId) -> Children<R> {
        self.widget
            .render(state.downcast_ref().unwrap(), element_id)
    }

    #[inline]
    fn lifecycle(
        &self,
        state: &mut Self::State,
        lifecycle: Lifecycle<&dyn Any>,
        renderer: &mut R,
        messages: &mut MessageSink,
    ) {
        self.widget.lifecycle(
            state.downcast_mut().unwrap(),
            lifecycle.map(|widget| widget.downcast_ref().unwrap()),
            renderer,
            messages,
        )
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        state: &mut Self::State,
        box_constraints: BoxConstraints,
        children: Vec<ElementId>,
        renderer: &mut R,
        messages: &mut MessageSink,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.widget.layout(
            state.downcast_mut().unwrap(),
            box_constraints,
            children,
            renderer,
            messages,
        )
    }

    #[inline]
    fn draw(
        &self,
        state: &mut Self::State,
        bounds: Rectangle,
        renderer: &mut R,
        messages: &mut MessageSink,
    ) -> Option<Primitive> {
        self.widget
            .draw(state.downcast_mut().unwrap(), bounds, renderer, messages)
    }

    #[inline]
    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }

    #[inline]
    fn short_type_name(&self) -> &'static str {
        self.widget.short_type_name()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self.widget.as_any()
    }
}

impl<W, R, S, M> WidgetSeal for WidgetProxy<W, R, S, M> {}

unsafe impl<W: Send, R, S, M> Send for WidgetProxy<W, R, S, M> {}

unsafe impl<W: Sync, R, S, M> Sync for WidgetProxy<W, R, S, M> {}
