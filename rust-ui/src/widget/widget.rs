use std::any::{self, Any};
use std::fmt;
use std::sync::Arc;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle, PaintContext};
use crate::render::RenderContext;
use crate::support::generator::Generator;

use super::element::{Children, Element, IntoElement, Key};
use super::state::{State, StateCell};
use super::widget_tree::{WidgetId, WidgetTree};

pub trait Widget<Renderer>: Send + Sync + WidgetMeta {
    type State: Default + Send + Sync;

    #[inline]
    fn should_update(
        &self,
        _children: &Children<Renderer>,
        _state: StateCell<Self::State>,
        _new_widget: &Self,
        _new_children: &Children<Renderer>,
    ) -> bool {
        true
    }

    #[inline]
    fn render(
        self: Arc<Self>,
        children: Children<Renderer>,
        _state: StateCell<Self::State>,
        _context: RenderContext,
    ) -> Children<Renderer> {
        children
    }

    #[inline]
    fn lifecycle(
        self: Arc<Self>,
        _children: Children<Renderer>,
        _state: StateCell<Self::State>,
        _lifecycle: Lifecycle<Arc<Self>, Children<Renderer>>,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) {
    }

    #[inline]
    fn layout<'a>(
        self: Arc<Self>,
        _children: Children<Renderer>,
        _state: StateCell<Self::State>,
        box_constraints: BoxConstraints,
        widget_id: WidgetId,
        widget_tree: &'a WidgetTree<Renderer>,
        _renderer: &mut Renderer,
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
        self: Arc<Self>,
        _children: Children<Renderer>,
        _state: StateCell<Self::State>,
        _bounds: Rectangle,
        _renderer: &mut Renderer,
        _context: &mut PaintContext,
    ) -> Option<Primitive> {
        None
    }
}

pub trait PolymophicWidget<Renderer>: Send + Sync + WidgetMeta {
    fn initial_state(&self) -> Box<dyn Any + Send + Sync>;

    fn should_update(
        &self,
        children: &Children<Renderer>,
        state: Arc<State>,
        new_widget: &dyn PolymophicWidget<Renderer>,
        new_children: &Children<Renderer>,
    ) -> bool;

    fn render(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        context: RenderContext
    ) -> Children<Renderer>;

    fn lifecycle(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        lifecycle: Lifecycle<Arc<dyn PolymophicWidget<Renderer>>, Children<Renderer>>,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    );

    fn layout<'a>(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        box_constraints: BoxConstraints,
        widget_id: WidgetId,
        widget_tree: &'a WidgetTree<Renderer>,
        renderer: &mut Renderer,
    ) -> Generator<'a, LayoutRequest, Size, Size>;

    fn draw(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        bounds: Rectangle,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) -> Option<Primitive>;
}

pub trait WidgetMeta {
    #[inline]
    fn name(&self) -> &'static str {
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

impl<Renderer> fmt::Debug for dyn PolymophicWidget<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {{ .. }}", self.name())
    }
}

impl<Widget, Renderer> PolymophicWidget<Renderer> for Widget
where
    Widget: self::Widget<Renderer> + 'static,
    Widget::State: 'static,
{
    #[inline]
    fn initial_state(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(Widget::State::default())
    }

    #[inline]
    fn should_update(
        &self,
        children: &Children<Renderer>,
        state: Arc<State>,
        new_widget: &dyn PolymophicWidget<Renderer>,
        new_children: &Children<Renderer>,
    ) -> bool {
        self.should_update(
            children,
            StateCell::new(state),
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
            new_children,
        )
    }

    #[inline]
    fn render(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        context: RenderContext,
    ) -> Children<Renderer> {
        self.render(
            children,
            StateCell::new(state),
            context,
        )
    }

    #[inline]
    fn lifecycle(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        lifecycle: Lifecycle<Arc<dyn PolymophicWidget<Renderer>>, Children<Renderer>>,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) {
        self.lifecycle(
            children,
            StateCell::new(state),
            lifecycle.map(downcast_widget),
            renderer,
            context,
        );
    }

    #[inline]
    fn layout<'a>(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        box_constraints: BoxConstraints,
        widget_id: WidgetId,
        widget_tree: &'a WidgetTree<Renderer>,
        renderer: &mut Renderer,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.layout(
            children,
            StateCell::new(state),
            box_constraints,
            widget_id,
            widget_tree,
            renderer,
        )
    }

    #[inline]
    fn draw(
        self: Arc<Self>,
        children: Children<Renderer>,
        state: Arc<State>,
        bounds: Rectangle,
        renderer: &mut Renderer,
        context: &mut PaintContext,
    ) -> Option<Primitive> {
        self.draw(children, StateCell::new(state), bounds, renderer, context)
    }
}

impl<Widget, Renderer> IntoElement<Renderer> for Widget
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Renderer>) -> Element<Renderer>
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

impl<Widget, Renderer> IntoElement<Renderer> for WithKey<Widget>
where
    Widget: self::Widget<Renderer> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Renderer>) -> Element<Renderer> {
        Element {
            widget: Arc::new(self.inner),
            children,
            key: Some(self.key),
        }
    }
}

pub fn downcast_widget<Widget, Renderer>(widget: Arc<dyn PolymophicWidget<Renderer>>) -> Arc<Widget>
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
    let mut cursor = 0;

    while let Some(separator_offset) = name[cursor..].find("::") {
        let slice_name = &name[cursor..cursor + separator_offset];
        if let Some(generics_offset) = slice_name.find("<") {
            return &name[cursor..cursor + generics_offset];
        }
        cursor += separator_offset + 2;
    }

    if let Some(generics_offset) = name[cursor..].find("<") {
        &name[cursor..cursor + generics_offset]
    } else {
        &name[cursor..]
    }
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
