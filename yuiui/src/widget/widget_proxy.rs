use std::any::Any;
use std::marker::PhantomData;
use yuiui_support::slot_tree::NodeId;

use super::{Command, DrawContext, LayoutContext, Lifecycle, Widget};
use crate::event::WindowEvent;
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub struct WidgetProxy<W, M, S> {
    widget: W,
    message_type: PhantomData<M>,
    state_type: PhantomData<S>,
}

impl<W, M, S> WidgetProxy<W, M, S> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            message_type: PhantomData,
            state_type: PhantomData,
        }
    }
}

impl<W, M> Widget<M, dyn Any> for WidgetProxy<W, M, W::State>
where
    W: 'static + Widget<M>,
    M: 'static,
    W::State: 'static,
{
    type State = Box<dyn Any>;

    fn initial_state(&self) -> Self::State {
        Box::new(self.widget.initial_state())
    }

    fn should_update(&self, new_widget: &dyn Any, state: &Self::State) -> bool {
        self.widget.should_update(
            new_widget.downcast_ref().unwrap(),
            state.downcast_ref().unwrap(),
        )
    }

    fn on_event(&self, event: WindowEvent, state: &mut Self::State) -> Option<Command<M>> {
        self.widget.on_event(event, state.downcast_mut().unwrap())
    }

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn Any>,
        state: &mut Self::State,
    ) -> Option<Command<M>> {
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
        state: &mut Self::State,
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
        state: &mut Self::State,
    ) -> Primitive {
        self.widget
            .draw(bounds, children, context, state.downcast_mut().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }
}
