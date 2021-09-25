use std::any::Any;
use std::marker::PhantomData;
use yuiui_support::slot_tree::NodeId;

use super::{Attributes, DrawContext, LayoutContext, Widget};
use crate::geometrics::{BoxConstraints, Rectangle, Size};
use crate::graphics::Primitive;

pub struct WidgetProxy<W, S> {
    widget: W,
    state_type: PhantomData<S>,
}

impl<W, S> WidgetProxy<W, S> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            state_type: PhantomData,
        }
    }
}

impl<W> Widget<dyn Any> for WidgetProxy<W, W::State>
where
    W: 'static + Widget,
    W::State: 'static,
{
    type State = Box<dyn Any>;

    fn initial_state(&self) -> Self::State {
        Box::new(self.widget.initial_state())
    }

    fn should_update(
        &self,
        new_widget: &dyn Any,
        old_attributes: &Attributes,
        new_attributes: &Attributes,
        state: &Self::State,
    ) -> bool {
        self.widget.should_update(
            new_widget.downcast_ref().unwrap(),
            old_attributes,
            new_attributes,
            state.downcast_ref().unwrap(),
        )
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext,
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
        context: &mut DrawContext,
        state: &mut Self::State,
    ) -> Primitive {
        self.widget
            .draw(bounds, children, context, state.downcast_mut().unwrap())
    }

    fn type_name(&self) -> &'static str {
        self.widget.type_name()
    }
}