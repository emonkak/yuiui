pub mod element;
pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;
pub mod subscriber;
pub mod text;
pub mod tree;

use std::any::{self, Any};
use std::fmt;
use std::sync::Arc;

use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle, PaintContext};
use crate::render::RenderContext;
use crate::support::generator::Generator;
use crate::support::tree::NodeId;

use self::element::{Children, Element, IntoElement, Key};
use self::tree::WidgetTree;

pub trait Widget<Renderer>: Send + Sync + WidgetMeta {
    type State: Default + Send + Sync;

    #[inline]
    fn should_update(
        &self,
        _new_widget: &Self,
        _old_children: &Children<Renderer>,
        _new_children: &Children<Renderer>,
        _state: &Self::State,
    ) -> bool {
        true
    }

    #[inline]
    fn on_lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self, &Children<Renderer>>,
        _state: &mut Self::State,
        _renderer: &mut Renderer,
        _context: &mut PaintContext<Renderer>,
    ) {
    }

    #[inline]
    fn render(
        &self,
        children: Children<Renderer>,
        _state: &Self::State,
        _context: &mut RenderContext<Self, Renderer, Self::State>,
    ) -> Children<Renderer> {
        children
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Renderer>,
        _state: &mut Self::State,
        _renderer: &mut Renderer,
    ) -> Generator<LayoutRequest, Size, Size> {
        Generator::new(move |co| async move {
            if let Some(child_id) = tree[node_id].first_child() {
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
        _bounds: Rectangle,
        _state: &mut Self::State,
        _renderer: &mut Renderer,
        _context: &mut PaintContext<Renderer>,
    ) -> Option<Primitive> {
        None
    }
}

pub trait PolymophicWidget<Renderer>: Send + Sync + WidgetMeta {
    fn initial_state(&self) -> Box<dyn Any + Send + Sync>;

    fn should_update(
        &self,
        new_widget: &dyn PolymophicWidget<Renderer>,
        old_children: &Children<Renderer>,
        new_children: &Children<Renderer>,
        state: &dyn Any,
    ) -> bool;

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>, &Children<Renderer>>,
        state: &mut dyn Any,
        renderer: &mut Renderer,
        context: &mut PaintContext<Renderer>,
    );

    fn render(
        &self,
        children: Children<Renderer>,
        state: &dyn Any,
        node_id: NodeId,
    ) -> Children<Renderer>;

    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Renderer>,
        state: &mut dyn Any,
        renderer: &mut Renderer,
    ) -> Generator<LayoutRequest, Size, Size>;

    fn draw(
        &self,
        bounds: Rectangle,
        state: &mut dyn Any,
        renderer: &mut Renderer,
        context: &mut PaintContext<Renderer>,
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
        let initial_state: Widget::State = Default::default();
        Box::new(initial_state)
    }

    #[inline]
    fn should_update(
        &self,
        new_widget: &dyn PolymophicWidget<Renderer>,
        old_children: &Children<Renderer>,
        new_children: &Children<Renderer>,
        state: &dyn Any,
    ) -> bool {
        self.should_update(
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
            old_children,
            new_children,
            state.downcast_ref().unwrap(),
        )
    }

    #[inline]
    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn PolymophicWidget<Renderer>, &Children<Renderer>>,
        state: &mut dyn Any,
        renderer: &mut Renderer,
        context: &mut PaintContext<Renderer>,
    ) {
        self.on_lifecycle(
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
            renderer,
            context,
        );
    }

    #[inline]
    fn render(
        &self,
        children: Children<Renderer>,
        state: &dyn Any,
        node_id: NodeId,
    ) -> Children<Renderer> {
        self.render(
            children,
            state.downcast_ref().unwrap(),
            &mut RenderContext::new(node_id),
        )
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Renderer>,
        state: &mut dyn Any,
        renderer: &mut Renderer,
    ) -> Generator<LayoutRequest, Size, Size> {
        self.layout(
            node_id,
            box_constraints,
            tree,
            state.downcast_mut().unwrap(),
            renderer,
        )
    }

    #[inline]
    fn draw(
        &self,
        bounds: Rectangle,
        state: &mut dyn Any,
        renderer: &mut Renderer,
        context: &mut PaintContext<Renderer>,
    ) -> Option<Primitive> {
        self.draw(bounds, state.downcast_mut().unwrap(), renderer, context)
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
