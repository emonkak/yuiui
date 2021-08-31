use std::any::{self, Any, TypeId};
use std::fmt;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle};
use crate::support::generator::Generator;

use super::element::{Children, ElementId, Key, WithKey};
use super::message::{AnyMessage, MessageEmitter, MessageQueue, MessageSender};

pub trait Widget<Renderer>: AsAny + Send + Sync  {
    type State: Default;

    type Message: Send;

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
    fn should_render(
        &self,
        _state: &Self::State,
        _new_widget: &Self,
    ) -> bool {
        true
    }

    #[inline]
    fn render(
        &self,
        _state: &Self::State,
        _element_id: ElementId,
    ) -> Children<Renderer> {
        Vec::new()
    }

    #[inline]
    fn lifecycle(
        &self,
        _state: &mut Self::State,
        _lifecycle: Lifecycle<&Self>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
    ) {
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
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
        _context: &mut MessageEmitter<Self::Message>,
    ) -> Option<Primitive> {
        None
    }

    #[inline]
    fn with_key(self, key: Key) -> WithKey<Self> where Self: Sized {
        WithKey { inner: self, key }
    }
}

pub trait PolymophicWidget<Renderer>: AsAny + Send + Sync {
    fn initial_state(&self) -> AnyState;

    fn inbound_type(&self) -> TypeId;

    fn update(
        &self,
        _state: &mut AnyState,
        _event: &AnyMessage,
        _messages: &mut MessageQueue,
    ) -> bool;

    fn should_render(
        &self,
        state: &AnyState,
        new_widget: &dyn PolymophicWidget<Renderer>,
    ) -> bool;

    fn render(
        &self,
        state: &AnyState,
        element_id: ElementId,
    ) -> Children<Renderer>;

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
        name
            .split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
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
    fn initial_state(&self) -> AnyState {
        Box::new(Widget::State::default())
    }

    #[inline]
    fn inbound_type(&self) -> TypeId {
        TypeId::of::<Widget::Message>()
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
    fn should_render(
        &self,
        state: &AnyState,
        new_widget: &dyn PolymophicWidget<Renderer>,
    ) -> bool {
        self.should_render(
            state.downcast_ref().unwrap(),
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
        )
    }

    #[inline]
    fn render(
        &self,
        state: &AnyState,
        element_id: ElementId,
    ) -> Children<Renderer> {
        self.render(
            state.downcast_ref().unwrap(),
            element_id
        )
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
