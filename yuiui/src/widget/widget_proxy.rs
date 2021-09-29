use std::any::Any;
use std::marker::PhantomData;
use yuiui_support::slot_tree::NodeId;

use super::{DrawContext, Effect, Event, LayoutContext, Lifecycle, Widget};
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub struct WidgetProxy<W, S, M, LS> {
    widget: W,
    state_type: PhantomData<S>,
    message_type: PhantomData<M>,
    local_state_type: PhantomData<LS>,
}

impl<W, S, M, LS> WidgetProxy<W, M, S, LS> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            state_type: PhantomData,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<W, S, M> Widget<S, M, dyn Any> for WidgetProxy<W, S, M, W::LocalState>
where
    W: 'static + Widget<S, M>,
    S: 'static,
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

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn Any>,
        state: &mut Self::LocalState,
    ) -> Effect<M> {
        self.widget.on_lifecycle(
            lifecycle.map(|widget| widget.downcast_ref().unwrap()),
            state.downcast_mut().unwrap(),
        )
    }

    fn on_event(&self, event: &Event<S>, state: &mut Self::LocalState) -> Effect<M> {
        self.widget.on_event(event, state.downcast_mut().unwrap())
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<S, M>,
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
        context: &mut DrawContext<S, M>,
        state: &mut Self::LocalState,
    ) -> Primitive {
        self.widget
            .draw(bounds, children, context, state.downcast_mut().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }
}
