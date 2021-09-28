use std::any::{self, Any};
use std::fmt;
use std::rc::Rc;
use yuiui_support::slot_tree::NodeId;

use super::{
    short_type_name_of, AsAny, DrawContext, Effect, LayoutContext, Lifecycle, WidgetProxy,
};
use crate::event::WindowEvent;
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub type RcWidget<Message> = Rc<dyn Widget<Message, dyn Any, State = Box<dyn Any>>>;

pub trait Widget<Message, Own: ?Sized = Self>: AsAny {
    type State;

    fn initial_state(&self) -> Self::State;

    fn should_update(&self, _new_widget: &Own, _state: &Self::State) -> bool {
        true
    }

    fn on_event(&self, _event: &WindowEvent, _state: &mut Self::State) -> Effect<Message> {
        Effect::None
    }

    fn on_lifecycle(
        &self,
        _lifecycle: Lifecycle<&Own>,
        _state: &mut Self::State,
    ) -> Effect<Message> {
        Effect::None
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<Message>,
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
        context: &mut DrawContext<Message>,
        _state: &mut Self::State,
    ) -> Primitive {
        children.iter().fold(Primitive::None, |primitive, child| {
            primitive + context.draw_child(*child)
        })
    }

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn into_rc(self) -> RcWidget<Message>
    where
        Self: 'static + Sized + Widget<Message>,
        <Self as Widget<Message>>::State: 'static,
        Message: 'static,
    {
        Rc::new(WidgetProxy::new(self))
    }
}

impl<M, O: ?Sized, S> fmt::Debug for dyn Widget<M, O, State = S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
