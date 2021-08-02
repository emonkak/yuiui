pub mod element;
pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;
pub mod subscriber;
pub mod tree;

use std::any::{self, Any};
use std::fmt;
use std::sync::Arc;

use crate::generator::Generator;
use crate::geometrics::{Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::paint::{PaintContext, PaintCycle, PaintHint};
use crate::render::{RenderContext, RenderCycle};
use crate::tree::NodeId;

use self::element::{Children, Element, IntoElement, Key};
use self::tree::WidgetTree;

pub trait Widget<Handle>: Send + Sync + WidgetMeta {
    type State: Default + Send + Sync;

    #[inline]
    fn should_update(
        &self,
        _new_widget: &Self,
        _old_children: &Children<Handle>,
        _new_children: &Children<Handle>,
        _state: &Self::State,
    ) -> bool {
        true
    }

    #[inline]
    fn on_render_cycle(
        &self,
        _render_cycle: RenderCycle<&Self, &Children<Handle>>,
        _state: &mut Self::State,
        _context: &mut RenderContext<Self, Handle, Self::State>,
    ) {
    }

    #[inline]
    fn on_paint_cycle(
        &self,
        _paint_cycle: PaintCycle<&Self, &Children<Handle>>,
        _state: &mut Self::State,
        _context: &mut PaintContext<Handle>,
    ) {
    }

    #[inline]
    fn render(
        &self,
        children: Children<Handle>,
        _state: &Self::State,
        _context: &mut RenderContext<Self, Handle, Self::State>,
    ) -> Children<Handle> {
        children
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Handle>,
        _state: &mut Self::State,
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
    fn paint(
        &self,
        _rectangle: &Rectangle,
        _state: &mut Self::State,
        _context: &mut PaintContext<Handle>,
    ) -> PaintHint {
        PaintHint::Once
    }
}

pub trait PolymophicWidget<Handle>: Send + Sync + WidgetMeta {
    fn initial_state(&self) -> Box<dyn Any + Send + Sync>;

    fn should_update(
        &self,
        new_widget: &dyn PolymophicWidget<Handle>,
        old_children: &Children<Handle>,
        new_children: &Children<Handle>,
        state: &dyn Any,
    ) -> bool;

    fn on_render_cycle(
        &self,
        render_cycle: RenderCycle<&dyn PolymophicWidget<Handle>, &Children<Handle>>,
        state: &mut dyn Any,
        node_id: NodeId,
    );

    fn on_paint_cycle(
        &self,
        paint_cycle: PaintCycle<&dyn PolymophicWidget<Handle>, &Children<Handle>>,
        state: &mut dyn Any,
        context: &mut PaintContext<Handle>,
    );

    fn render(
        &self,
        children: Children<Handle>,
        state: &dyn Any,
        node_id: NodeId,
    ) -> Children<Handle>;

    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Handle>,
        state: &mut dyn Any,
    ) -> Generator<LayoutRequest, Size, Size>;

    fn paint(
        &self,
        rectangle: &Rectangle,
        state: &mut dyn Any,
        context: &mut PaintContext<Handle>,
    ) -> PaintHint;
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

impl<Handle> fmt::Debug for dyn PolymophicWidget<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {{ .. }}", self.name())
    }
}

impl<Widget, Handle> PolymophicWidget<Handle> for Widget
where
    Widget: self::Widget<Handle> + 'static,
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
        new_widget: &dyn PolymophicWidget<Handle>,
        old_children: &Children<Handle>,
        new_children: &Children<Handle>,
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
    fn on_render_cycle(
        &self,
        render_cycle: RenderCycle<&dyn PolymophicWidget<Handle>, &Children<Handle>>,
        state: &mut dyn Any,
        node_id: NodeId,
    ) {
        self.on_render_cycle(
            render_cycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
            &mut RenderContext::new(node_id),
        );
    }

    #[inline]
    fn on_paint_cycle(
        &self,
        paint_cycle: PaintCycle<&dyn PolymophicWidget<Handle>, &Children<Handle>>,
        state: &mut dyn Any,
        context: &mut PaintContext<Handle>,
    ) {
        self.on_paint_cycle(
            paint_cycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
            context,
        );
    }

    #[inline]
    fn render(
        &self,
        children: Children<Handle>,
        state: &dyn Any,
        node_id: NodeId,
    ) -> Children<Handle> {
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
        tree: &'a WidgetTree<Handle>,
        state: &mut dyn Any,
    ) -> Generator<LayoutRequest, Size, Size> {
        self.layout(
            node_id,
            box_constraints,
            tree,
            state.downcast_mut().unwrap(),
        )
    }

    #[inline]
    fn paint(
        &self,
        rectangle: &Rectangle,
        state: &mut dyn Any,
        context: &mut PaintContext<Handle>,
    ) -> PaintHint {
        self.paint(rectangle, state.downcast_mut().unwrap(), context)
    }
}

impl<Widget, Handle> IntoElement<Handle> for Widget
where
    Widget: self::Widget<Handle> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Handle>) -> Element<Handle>
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

impl<Widget, Handle> IntoElement<Handle> for WithKey<Widget>
where
    Widget: self::Widget<Handle> + WidgetMeta + 'static,
    Widget::State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Handle>) -> Element<Handle> {
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
