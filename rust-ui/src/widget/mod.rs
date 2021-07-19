pub mod element;
pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;

use std::any::{self, Any};
use std::fmt;

use crate::generator::Generator;
use crate::geometrics::{Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::lifecycle::{Lifecycle, LifecycleContext};
use crate::paint::PaintContext;
use crate::tree::{Link, NodeId, Tree};

use self::element::{Child, Children, Element, IntoElement, Key};

pub type WidgetTree<Handle> = Tree<BoxedWidget<Handle>>;

pub type WidgetNode<Handle> = Link<BoxedWidget<Handle>>;

pub type BoxedWidget<Handle> = Box<dyn DynamicWidget<Handle>>;

pub trait Widget<Handle>: WidgetMeta {
    type State: Default;

    #[inline]
    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    #[inline]
    fn should_update(&self, _new_widget: &Self, _state: &Self::State) -> bool {
        true
    }

    #[inline]
    fn lifecycle(
        &self,
        _lifecycle: Lifecycle<&Self>,
        _state: &mut Self::State,
        _context: &mut LifecycleContext,
    ) {
    }

    #[inline]
    fn render(&self, children: Children<Handle>, _state: &Self::State) -> Child<Handle> {
        Child::Multiple(children)
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Handle>,
        _state: &'a Self::State,
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
        _handle: &Handle,
        _rectangle: &Rectangle,
        _state: &mut Self::State,
        _paint_context: &mut dyn PaintContext<Handle>,
    ) {
    }
}

pub trait DynamicWidget<Handle>: WidgetMeta {
    fn initial_state(&self) -> Box<dyn Any>;

    fn should_update(&self, new_widget: &dyn DynamicWidget<Handle>, state: &dyn Any) -> bool;

    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn DynamicWidget<Handle>>,
        state: &mut dyn Any,
        context: &mut LifecycleContext,
    );

    fn render(&self, children: Children<Handle>, state: &dyn Any) -> Child<Handle>;

    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Handle>,
        state: &'a dyn Any,
    ) -> Generator<LayoutRequest, Size, Size>;

    fn paint(
        &self,
        handle: &Handle,
        rectangle: &Rectangle,
        state: &mut dyn Any,
        paint_context: &mut dyn PaintContext<Handle>,
    );
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

impl<Handle> fmt::Debug for dyn DynamicWidget<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<Outer, Handle, State> DynamicWidget<Handle> for Outer
where
    Outer: Widget<Handle, State = State> + 'static,
    State: 'static,
{
    #[inline]
    fn initial_state(&self) -> Box<dyn Any> {
        Box::new(self.initial_state())
    }

    #[inline]
    fn should_update(&self, new_widget: &dyn DynamicWidget<Handle>, state: &dyn Any) -> bool {
        self.should_update(
            new_widget.as_any().downcast_ref::<Self>().unwrap(),
            state.downcast_ref().unwrap(),
        )
    }

    #[inline]
    fn lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn DynamicWidget<Handle>>,
        state: &mut dyn Any,
        context: &mut LifecycleContext,
    ) {
        self.lifecycle(
            lifecycle.map(|widget| widget.as_any().downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
            context,
        );
    }

    #[inline]
    fn render(&self, children: Children<Handle>, state: &dyn Any) -> Child<Handle> {
        self.render(children, state.downcast_ref().unwrap())
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Handle>,
        state: &'a dyn Any,
    ) -> Generator<LayoutRequest, Size, Size> {
        self.layout(
            node_id,
            box_constraints,
            tree,
            state.downcast_ref().unwrap(),
        )
    }

    #[inline]
    fn paint(
        &self,
        handle: &Handle,
        rectangle: &Rectangle,
        state: &mut dyn Any,
        paint_context: &mut dyn PaintContext<Handle>,
    ) {
        self.paint(
            handle,
            rectangle,
            state.downcast_mut().unwrap(),
            paint_context,
        )
    }
}

impl<Outer, Handle, State: 'static> IntoElement<Handle> for Outer
where
    Outer: Widget<Handle, State = State> + WidgetMeta + 'static,
    State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Handle>) -> Element<Handle>
    where
        Self: Sized + 'static,
    {
        Element {
            widget: Box::new(self),
            children,
            key: None,
        }
    }
}

impl<Inner, Handle, State> IntoElement<Handle> for WithKey<Inner>
where
    Inner: Widget<Handle, State = State> + 'static,
    State: 'static,
{
    #[inline]
    fn into_element(self, children: Children<Handle>) -> Element<Handle>
    where
        Inner: Sized + 'static,
    {
        Element {
            widget: Box::new(self.inner),
            children,
            key: Some(self.key),
        }
    }
}

fn get_short_type_name(full_name: &str) -> &str {
    let mut cursor = 0;

    while let Some(offset) = full_name[cursor..].find("::") {
        let slice_name = &full_name[cursor..cursor + offset];
        if slice_name.contains("<") {
            break;
        }
        cursor += offset + 2;
    }

    &full_name[cursor..]
}

#[cfg(test)]
#[test]
fn test_get_short_type_name() {
    assert_eq!(get_short_type_name("Foo"), "Foo");
    assert_eq!(get_short_type_name("Foo<Bar>"), "Foo<Bar>");
    assert_eq!(get_short_type_name("Foo<Bar::Baz>"), "Foo<Bar::Baz>");
    assert_eq!(get_short_type_name("Foo::Bar"), "Bar");
    assert_eq!(get_short_type_name("Foo::Bar<Baz>"), "Bar<Baz>");
    assert_eq!(get_short_type_name("Foo::Bar<Baz::Qux>"), "Bar<Baz::Qux>");
}
