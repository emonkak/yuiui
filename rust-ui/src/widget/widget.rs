use std::any::{self, Any};
use std::fmt;
use std::sync::Arc;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle, PaintContext};
use crate::support::generator::Generator;

use super::element::{Children, Element, ElementId, ElementTree, BuildElement, Key};
use super::message::{AnyMessage, MessageContext};

pub trait Widget<Renderer>: Send + Sync + WidgetMeta {
    type State: Default;

    type Message: Send + Sync;

    type PaintObject: Default;

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
    fn update(&self, _state: &mut Self::State, _message: Self::Message) -> bool {
        true
    }

    #[inline]
    fn render(
        &self,
        children: &Children<Renderer>,
        _state: &Self::State,
        _message_context: MessageContext<Self::Message>,
    ) -> Children<Renderer> {
        children.clone()
    }

    #[inline]
    fn lifecycle(
        &self,
        _children: &Children<Renderer>,
        _paint_object: &mut Self::PaintObject,
        _lifecycle: Lifecycle<Arc<Self>, Children<Renderer>>,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) {
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        _children: &Children<Renderer>,
        _paint_object: &mut Self::PaintObject,
        box_constraints: BoxConstraints,
        widget_id: ElementId,
        widget_tree: &'a ElementTree<Renderer>,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = widget_tree[widget_id].first_child() {
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
        _paint_object: &mut Self::PaintObject,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) -> Option<Primitive> {
        None
    }
}

pub trait PolymophicWidget<Renderer>: Send + Sync + WidgetMeta {
    fn initial_state(&self) -> AnyState;

    fn initial_paint_object(&self) -> AnyPaintObject;

    fn should_render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        new_widget: &Arc<dyn PolymophicWidget<Renderer>>,
        new_children: &Children<Renderer>,
    ) -> bool;

    fn update(&self, _state: &mut AnyState, _message: AnyMessage) -> bool;

    fn render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        element_id: ElementId,
        version: usize,
    ) -> Children<Renderer>;

    fn lifecycle(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        lifecycle: Lifecycle<Arc<dyn PolymophicWidget<Renderer>>, Children<Renderer>>,
        renderer: &mut Renderer,
        _context: &mut PaintContext,
    );

    fn layout<'a>(
        &'a self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        box_constraints: BoxConstraints,
        widget_id: ElementId,
        widget_tree: &'a ElementTree<Renderer>,
        renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) -> Generator<'a, LayoutRequest, Size, Size>;

    fn draw(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        bounds: Rectangle,
        renderer: &mut Renderer,
        _context: &mut PaintContext,
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

pub type AnyPaintObject = Box<dyn Any>;

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
    Widget::PaintObject: 'static,
{
    #[inline]
    fn initial_state(&self) -> AnyState {
        Box::new(Widget::State::default())
    }

    #[inline]
    fn initial_paint_object(&self) -> AnyPaintObject {
        Box::new(Widget::PaintObject::default())
    }

    #[inline]
    fn should_render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        new_widget: &Arc<dyn PolymophicWidget<Renderer>>,
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
    fn update(&self, state: &mut AnyState, message: AnyMessage) -> bool {
        self.update(
            state.downcast_mut::<Widget::State>().unwrap(),
            *message.downcast::<Widget::Message>().unwrap(),
        )
    }

    #[inline]
    fn render(
        &self,
        children: &Children<Renderer>,
        state: &AnyState,
        element_id: ElementId,
        version: usize,
    ) -> Children<Renderer> {
        self.render(
            children,
            state.downcast_ref().unwrap(),
            MessageContext::new(element_id, version),
        )
    }

    #[inline]
    fn lifecycle(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        lifecycle: Lifecycle<Arc<dyn PolymophicWidget<Renderer>>, Children<Renderer>>,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) {
        self.lifecycle(
            children,
            paint_object.downcast_mut().unwrap(),
            lifecycle.map(coerce_widget),
            renderer,
            context,
        );
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        box_constraints: BoxConstraints,
        element_id: ElementId,
        element_tree: &'a ElementTree<Renderer>,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.layout(
            children,
            paint_object.downcast_mut().unwrap(),
            box_constraints,
            element_id,
            element_tree,
            renderer,
            context,
        )
    }

    #[inline]
    fn draw(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut AnyPaintObject,
        bounds: Rectangle,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) -> Option<Primitive> {
        self.draw(
            children,
            paint_object.downcast_mut().unwrap(),
            bounds,
            renderer,
            context,
        )
    }
}

impl<Widget, Renderer> BuildElement<Renderer> for Widget
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
    Widget::Message: 'static,
    Widget::PaintObject: 'static,
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
    Widget::PaintObject: 'static,
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

pub fn coerce_widget<Widget, Renderer>(widget: Arc<dyn PolymophicWidget<Renderer>>) -> Arc<Widget>
where
    Widget: 'static,
{
    assert!(widget.as_any().is::<Widget>());
    unsafe {
        let ptr = Arc::into_raw(widget).cast::<Widget>();
        Arc::from_raw(ptr)
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
