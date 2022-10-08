use slot_vec::graph::NodeId;
use std::any::Any;
use std::marker::PhantomData;

use super::{DrawContext, Effect, Event, LayoutContext, Lifecycle, Widget};
use crate::geometrics::{BoxConstraints, Rect, Size};
use crate::graphics::Primitive;
use crate::style::LayoutStyle;

pub struct WidgetProxy<Inner, State, Message, LocalState> {
    inner: Inner,
    state_type: PhantomData<State>,
    message_type: PhantomData<Message>,
    local_state_type: PhantomData<LocalState>,
}

impl<Inner, State, Message, LocalState> WidgetProxy<Inner, Message, State, LocalState> {
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            state_type: PhantomData,
            message_type: PhantomData,
            local_state_type: PhantomData,
        }
    }
}

impl<Inner, State, Message> Widget<State, Message, dyn Any>
    for WidgetProxy<Inner, State, Message, Inner::LocalState>
where
    Inner: 'static + Widget<State, Message>,
    State: 'static,
    Message: 'static,
    Inner::LocalState: 'static,
{
    type LocalState = Box<dyn Any>;

    fn initial_state(&self) -> Self::LocalState {
        Box::new(self.inner.initial_state())
    }

    fn should_update(&self, new_widget: &dyn Any) -> bool {
        self.inner
            .should_update(&new_widget.downcast_ref::<Self>().unwrap().inner)
    }

    fn on_lifecycle(
        &self,
        lifecycle: Lifecycle<&dyn Any>,
        state: &mut Self::LocalState,
    ) -> Effect<Message> {
        self.inner.on_lifecycle(
            lifecycle.map(|widget| &widget.downcast_ref::<Self>().unwrap().inner),
            state.downcast_mut().unwrap(),
        )
    }

    fn on_event(
        &self,
        event: Event<State>,
        bounds: Rect,
        state: &mut Self::LocalState,
    ) -> Effect<Message> {
        self.inner
            .on_event(event, bounds, state.downcast_mut().unwrap())
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<State, Message>,
        state: &mut Self::LocalState,
    ) -> Size {
        self.inner.layout(
            box_constraints,
            children,
            context,
            state.downcast_mut().unwrap(),
        )
    }

    fn layout_style(&self) -> LayoutStyle {
        self.inner.layout_style()
    }

    fn draw(
        &self,
        bounds: Rect,
        children: &[NodeId],
        context: &mut DrawContext<State, Message>,
        state: &mut Self::LocalState,
    ) -> Primitive {
        self.inner
            .draw(bounds, children, context, state.downcast_mut().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }
}
