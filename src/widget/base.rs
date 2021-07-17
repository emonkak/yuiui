use std::any::{self, Any};

use crate::generator::Generator;
use crate::geometrics::{Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::paint::PaintContext;
use crate::tree::NodeId;

use super::element::Element;
use super::{Children, Key, Widget, WidgetMeta, WidgetTree};

pub struct WithKey<Inner> {
    pub(super) inner: Inner,
    pub(super) key: Key,
}

impl<Handle, Inner: Widget<Handle> + 'static> Widget<Handle> for WithKey<Inner> {
    type State = Inner::State;

    #[inline]
    fn initial_state(&self) -> Self::State {
        self.inner.initial_state()
    }

    #[inline]
    fn should_update(&self, new_widget: &Self, state: &Self::State) -> bool {
        println!("WithKey::should_update(); {}", any::type_name::<Self>());
        self.inner.should_update(&new_widget.inner, state)
    }

    #[inline]
    fn render(&self, children: Children<Handle>, state: &mut Self::State) -> Children<Handle> {
        self.inner.render(children, state)
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        box_constraints: BoxConstraints,
        node_id: NodeId,
        tree: &'a WidgetTree<Handle>,
        state: &Self::State,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.inner.layout(box_constraints, node_id, tree, state)
    }

    #[inline(always)]
    fn paint(
        &self,
        handle: &Handle,
        rectangle: &Rectangle,
        state: &mut Self::State,
        paint_context: &mut dyn PaintContext<Handle>,
    ) {
        self.inner.paint(handle, rectangle, state, paint_context)
    }

    fn into_element(self, children: Children<Handle>) -> Element<Handle>
    where
        Self: Sized + 'static,
        Self::State: 'static,
    {
        Element {
            widget: Box::new(self.inner),
            children,
            key: Some(self.key),
        }
    }
}

impl<Inner: WidgetMeta + 'static> WidgetMeta for WithKey<Inner> {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
}
