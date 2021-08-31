use std::any::{self, Any, TypeId};
use std::fmt;
use std::marker::PhantomData;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle};
use crate::support::generator::Generator;

use super::element::{Children, ElementId, Key, WithKey};
use super::message::{AnyMessage, MessageEmitter, MessageQueue, MessageSender};

pub trait Widget<Renderer>: ShouldRender + AsAny + Send + Sync {
    type State;

    type Message: Send;

    fn initial_state(&self) -> Self::State;

    #[inline]
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Self::Message,
        _messages: &mut MessageQueue,
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
        _lifecycle: Lifecycle<&Self>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) {
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = child_ids.first() {
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
        _context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        None
    }

    #[inline]
    fn with_key(self, key: Key) -> WithKey<Self>
    where
        Self: Sized,
    {
        WithKey { inner: self, key }
    }
}

pub trait PolymophicWidget<Renderer>: ShouldRender + AsAny + Send + Sync {
    fn inbound_type(&self) -> TypeId;

    fn initial_state(&self) -> Box<dyn Any>;

    fn update(
        &self,
        _state: &mut AnyState,
        _event: &AnyMessage,
        _messages: &mut MessageQueue,
    ) -> bool;

    fn render(&self, state: &AnyState, element_id: ElementId) -> Children<Renderer>;

    fn lifecycle(
        &self,
        state: &mut AnyState,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    );

    fn layout<'a>(
        &'a self,
        state: &mut AnyState,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Generator<'a, LayoutRequest, Size, Size>;

    fn draw(
        &self,
        state: &mut AnyState,
        bounds: Rectangle,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Option<Primitive>;

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
}

pub trait ShouldRender<T: ?Sized = dyn Any> {
    fn should_render(&self, _other: &T) -> bool {
        true
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub struct Proxy<W, S, M, R> {
    widget: W,
    renderer_type: PhantomData<R>,
    state_type: PhantomData<S>,
    message_type: PhantomData<M>,
}

unsafe impl<W: Send, R, S, M> Send for Proxy<W, R, S, M> {}
unsafe impl<W: Sync, R, S, M> Sync for Proxy<W, R, S, M> {}

impl<W, R> From<W> for Proxy<W, R, W::State, W::Message> where W: Widget<R> {
    fn from(widget: W) -> Self {
        Proxy {
            widget,
            renderer_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        }
    }
}

impl<W, R, S, M> Widget<R> for Proxy<W, R, S, M>
where
    W: Widget<R, State = S, Message = M> + 'static,
    R: 'static,
    S: 'static,
    M: 'static,
{
    type State = Box<dyn Any>;
    type Message = Box<dyn Any + Send>;

    #[inline]
    fn initial_state(&self) -> Self::State {
        Box::new(self.widget.initial_state())
    }

    #[inline]
    fn update(
        &self,
        state: &mut Self::State,
        event: &Self::Message,
        messages: &mut MessageQueue,
    ) -> bool {
        self.widget.update(
            state.downcast_mut().unwrap(),
            event.downcast_ref().unwrap(),
            messages
        )
    }

    #[inline]
    fn render(&self, state: &Self::State, element_id: ElementId) -> Children<R> {
        self.widget.render(
            state.downcast_ref().unwrap(),
            element_id
        )
    }

    #[inline]
    fn lifecycle(
        &self,
        state: &mut Self::State,
        lifecycle: Lifecycle<&Self>,
        renderer: &mut R,
        context: &mut MessageEmitter,
    ) {
        self.widget.lifecycle(
            state.downcast_mut().unwrap(),
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            renderer,
            context,
        )
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        state: &mut Self::State,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        renderer: &mut R,
        context: &mut MessageEmitter,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.widget.layout(
            state.downcast_mut().unwrap(),
            box_constraints,
            child_ids,
            renderer,
            context,
        )
    }

    #[inline]
    fn draw(
        &self,
        state: &mut Self::State,
        bounds: Rectangle,
        renderer: &mut R,
        context: &mut MessageEmitter,
    ) -> Option<Primitive> {
        self.widget.draw(
            state.downcast_mut().unwrap(),
            bounds,
            renderer,
            context,
        )
    }
}

impl<W, R, S, M> ShouldRender for Proxy<W, R, S, M>
where
    W: ShouldRender + 'static,
    R: 'static,
    S: 'static,
    M: 'static,
{
    fn should_render(&self, other: &dyn Any) -> bool {
        self.widget.should_render(other)
    }
}

impl<W, R, S, M> AsAny for Proxy<W, R, S, M>
where
    W: AsAny + 'static,
    R: 'static,
    S: 'static,
    M: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self.widget.as_any()
    }
}

pub type AnyState = Box<dyn Any>;

impl<Renderer> fmt::Debug for dyn PolymophicWidget<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {{ .. }}", self.short_type_name())
    }
}

impl<Widget, Renderer> PolymophicWidget<Renderer> for Widget
where
    Widget: self::Widget<Renderer> + 'static,
    Widget::State: 'static,
    Widget::Message: 'static,
{
    #[inline]
    fn inbound_type(&self) -> TypeId {
        TypeId::of::<Widget::Message>()
    }

    fn initial_state(&self) -> Box<dyn Any> {
        Box::new(self.initial_state())
    }

    #[inline]
    fn update(
        &self,
        state: &mut AnyState,
        message: &AnyMessage,
        messages: &mut MessageQueue,
    ) -> bool {
        self.update(
            state.downcast_mut::<Widget::State>().unwrap(),
            message.downcast_ref::<Widget::Message>().unwrap(),
            messages,
        )
    }

    #[inline]
    fn render(&self, state: &AnyState, element_id: ElementId) -> Children<Renderer> {
        self.render(state.downcast_ref().unwrap(), element_id)
    }

    #[inline]
    fn lifecycle(
        &self,
        state: &mut AnyState,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) {
        self.lifecycle(
            state.downcast_mut().unwrap(),
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        );
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        state: &mut AnyState,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.layout(
            state.downcast_mut().unwrap(),
            box_constraints,
            child_ids,
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        )
    }

    #[inline]
    fn draw(
        &self,
        state: &mut AnyState,
        bounds: Rectangle,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Option<Primitive> {
        self.draw(
            state.downcast_mut().unwrap(),
            bounds,
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        )
    }
}

impl<T: ShouldRender<T> + 'static> ShouldRender for T {
    fn should_render(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<T>()
            .map(|inner| self.should_render(inner))
            .unwrap_or(false)
    }
}
