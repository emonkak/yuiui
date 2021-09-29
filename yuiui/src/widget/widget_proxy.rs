use std::any::Any;
use std::marker::PhantomData;
use yuiui_support::slot_tree::NodeId;

use super::{DrawContext, Effect, LayoutContext, Lifecycle, Widget};
use crate::event::WindowEvent;
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub struct WidgetProxy<W, M, LS> {
    widget: W,
    message_type: PhantomData<M>,
    local_state_type: PhantomData<LS>,
}

impl<W, M, LS> WidgetProxy<W, M, LS> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<W, M> Widget<M, dyn Any> for WidgetProxy<W, M, W::LocalState>
where
    W: 'static + Widget<M>,
    M: 'static,
    W::LocalState: 'static,
{
    type LocalState = Box<dyn Any>;

    fn initial_state(&self) -> Self::LocalState {
        Box::new(self.widget.initial_state())
    }

    fn should_update(&self, new_widget: &dyn Any, state: &Self::LocalState) -> bool {
        self.widget.should_update(
            new_widget.downcast_ref().unwrap(),
            state.downcast_ref().unwrap(),
        )
    }

    fn on_event(&self, event: &WindowEvent, state: &mut Self::LocalState) -> Effect<M> {
        self.widget.on_event(event, state.downcast_mut().unwrap())
    }

    fn on_lifecycle(&self, lifecycle: Lifecycle<&dyn Any>, state: &mut Self::LocalState) -> Effect<M> {
        self.widget.on_lifecycle(
            lifecycle.map(|widget| widget.downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
        )
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<M>,
        state: &mut Self::LocalState,
    ) -> Size {
        self.widget.layout(
            box_constraints,
            children,
            context,
            state.downcast_mut().unwrap(),
        )
    }

    fn draw(
        &self,
        bounds: Rectangle,
        children: &[NodeId],
        context: &mut DrawContext<M>,
        state: &mut Self::LocalState,
    ) -> Primitive {
        self.widget
            .draw(bounds, children, context, state.downcast_mut().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }
}
