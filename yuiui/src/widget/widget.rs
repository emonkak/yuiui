use std::any::{self, Any};
use std::fmt;
use std::rc::Rc;
use yuiui_support::slot_tree::NodeId;

use super::{short_type_name_of, AsAny, Attributes, DrawContext, LayoutContext, WidgetProxy};
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub type BoxedWidget = Rc<dyn Widget<dyn Any, State = Box<dyn Any>>>;

pub trait Widget<Own: ?Sized = Self>: AsAny {
    type State;

    fn initial_state(&self) -> Self::State;

    fn should_update(
        &self,
        _new_widget: &Own,
        _old_attributes: &Attributes,
        _new_attributes: &Attributes,
        _state: &Self::State,
    ) -> bool {
        true
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext,
        _state: &mut Self::State,
    ) -> Size {
        if let Some(child) = children.first() {
            context.layout_child(*child, box_constraints)
        } else {
            box_constraints.max
        }
    }

    fn draw(
        &self,
        _bounds: Rectangle,
        children: &[NodeId],
        context: &mut DrawContext,
        _state: &mut Self::State,
    ) -> Primitive {
        children.iter().fold(Primitive::None, |primitive, child| {
            primitive + context.draw_child(*child)
        })
    }

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn into_boxed(self) -> BoxedWidget
    where
        Self: 'static + Sized + Widget<Self>,
    {
        Rc::new(WidgetProxy::new(self))
    }
}

impl<O: ?Sized, S> fmt::Debug for dyn Widget<O, State = S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
