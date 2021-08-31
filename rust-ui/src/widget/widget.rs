use std::any::{self, Any, TypeId};
use std::fmt;
use std::sync::Arc;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle};
use crate::support::generator::Generator;

use super::element::{Children, Element, ElementId, ElementTree, BuildElement, Key};
use super::message::{AnyMessage, MessageEmitter, MessageQueue, MessageSender};

pub trait Widget<Renderer>: Send + Sync + WidgetMeta {
    type State: Default;

    type Message: Send + Sync;

    #[inline]
    fn update(
        &self,
        _children: &Children<Renderer>,
        _state: &mut Self::State,
        _event: &Self::Message,
        _messages: &mut MessageQueue,
    ) -> bool {
        true
    }

    #[inline]
    fn should_render(
        &self,
        _children: &Children<Renderer>,
        _state: &Self::State,
        _new_widget: &Self,
        _new_children: &Children<Renderer>,
    ) -> bool {
        true
    }

    #[inline]
    fn render(
        &self,
        children: &Children<Renderer>,
        _state: &Self::State,
        _element_id: ElementId,
    ) -> Children<Renderer> {
        children.clone()
    }

    #[inline]
    fn lifecycle(
        &self,
        _children: &Children<Renderer>,
        _state: &mut Self::State,
        _lifecycle: Lifecycle<&Self, &Children<Renderer>>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
    ) {
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        _children: &Children<Renderer>,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        element_id: ElementId,
        element_tree: &'a ElementTree<Renderer>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = element_tree[element_id].first_child() {
                co.suspend(LayoutRequest::LayoutChild(child_id, box_constraints))
                    .await
            } else {
                box_constraints.max
            }
        })
    }

    #[inline]
    fn draw(
        &self,
        _children: &Children<Renderer>,
        _state: &mut Self::State,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter<Self::Message>,
    ) -> Option<Primitive> {
        None
    }
}

pub trait PolymophicWidget<Renderer>: Send + Sync + WidgetMeta {
    fn initial_state(&self) -> AnyState;

    fn inbound_type(&self) -> TypeId;

    fn update(
        &self,
        children: &Children<Renderer>,
        _state: &mut AnyState,
        _event: &AnyMessage,
        _messages: &mut MessageQueue,
    ) -> bool;

    fn should_render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        new_widget: &dyn PolymophicWidget<Renderer>,
        new_children: &Children<Renderer>,
    ) -> bool;

    fn render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        element_id: ElementId,
    ) -> Children<Renderer>;

    fn lifecycle(
        &self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>, &Children<Renderer>>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    );

    fn layout<'a>(
        &'a self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        box_constraints: BoxConstraints,
        element_id: ElementId,
        element_tree: &'a ElementTree<Renderer>,
        renderer: &mut Renderer,
        message_sender: &MessageSender,
    ) -> Generator<'a, LayoutRequest, Size, Size>;

    fn draw(
        &self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        bounds: Rectangle,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Option<Primitive>;
}

pub trait WidgetMeta {
    #[inline]
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    #[inline]
    fn short_type_name(&self) -> &'static str {
        get_short_type_name(any::type_name::<Self>())
    }

    #[inline]
    fn with_key(self, key: Key) -> WithKey<Self>
    where
        Self: Sized,
    {
        WithKey { inner: self, key }
    }

    fn as_any(&self) -> &dyn Any;
}

pub struct WithKey<Inner> {
    inner: Inner,
    key: Key,
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
        children: &Children<Renderer>,
        state: &mut AnyState,
        message: &AnyMessage,
        messages: &mut MessageQueue,
    ) -> bool {
        self.update(
            children,
            state.downcast_mut::<Widget::State>().unwrap(),
            message.downcast_ref::<Widget::Message>().unwrap(),
            messages,
        )
    }

    #[inline]
    fn should_render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        new_widget: &dyn PolymophicWidget<Renderer>,
        new_children: &Children<Renderer>,
    ) -> bool {
        self.should_render(
            children,
            state.downcast_ref().unwrap(),
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
            new_children,
        )
    }

    #[inline]
    fn render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        element_id: ElementId,
    ) -> Children<Renderer> {
        self.render(
            children,
            state.downcast_ref().unwrap(),
            element_id
        )
    }

    #[inline]
    fn lifecycle(
        &self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>, &Children<Renderer>>,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) {
        self.lifecycle(
            children,
            state.downcast_mut().unwrap(),
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        );
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        box_constraints: BoxConstraints,
        element_id: ElementId,
        element_tree: &'a ElementTree<Renderer>,
        renderer: &mut Renderer,
        message_sender: &MessageSender,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.layout(
            children,
            state.downcast_mut().unwrap(),
            box_constraints,
            element_id,
            element_tree,
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        )
    }

    #[inline]
    fn draw(
        &self,
        children: &Children<Renderer>,
        state: &mut AnyState,
        bounds: Rectangle,
        renderer: &mut Renderer,
        element_id: ElementId,
        message_sender: &MessageSender,
    ) -> Option<Primitive> {
        self.draw(
            children,
            state.downcast_mut().unwrap(),
            bounds,
            renderer,
            &mut MessageEmitter::new(element_id, message_sender),
        )
    }
}

impl<Widget, Renderer> BuildElement<Renderer> for Widget
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
    Widget::Message: 'static,
{
    #[inline]
    fn build_element(self, children: Children<Renderer>) -> Element<Renderer>
    where
        Self: Sized,
    {
        Element {
            widget: Arc::new(self),
            children,
            key: None,
        }
    }
}

impl<Widget, Renderer> BuildElement<Renderer> for WithKey<Widget>
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
    Widget::Message: 'static,
{
    #[inline]
    fn build_element(self, children: Children<Renderer>) -> Element<Renderer> {
        Element {
            widget: Arc::new(self.inner),
            children,
            key: Some(self.key),
        }
    }
}

fn get_short_type_name(name: &str) -> &str {
    name.split('<')
        .next()
        .unwrap_or(name)
        .split("::")
        .last()
        .unwrap_or(name)
}

#[cfg(test)]
#[test]
fn test_get_short_type_name() {
    assert_eq!(get_short_type_name("Foo"), "Foo");
    assert_eq!(get_short_type_name("Foo<Bar>"), "Foo");
    assert_eq!(get_short_type_name("Foo<Bar::Baz>"), "Foo");
    assert_eq!(get_short_type_name("Foo::Bar"), "Bar");
    assert_eq!(get_short_type_name("Foo::Bar<Baz>"), "Bar");
    assert_eq!(get_short_type_name("Foo::Bar<Baz::Qux>"), "Bar");
}
