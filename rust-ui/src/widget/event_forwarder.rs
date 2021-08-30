use rust_ui_derive::WidgetMeta;

use crate::event::{InboundEmitter, OutboundEmitter};
use crate::paint::{BoxConstraints, LayoutRequest, Lifecycle};
use crate::geometrics::{Rectangle, Size};
use crate::graphics::Primitive;
use crate::support::generator::Generator;

use super::element::{ElementId, ElementTree, Children};
use super::{Widget, WidgetMeta};

#[derive(Debug, WidgetMeta)]
pub struct EventFowarder<Child: 'static, ListenerFn: 'static> {
    child: Child,
    listener_id: ElementId,
    listener_fn: ListenerFn,
}

impl<Child, ListenerFn> EventFowarder<Child, ListenerFn> {
    pub fn new(child: Child, listener_id: ElementId, listener_fn: ListenerFn) -> Self {
        Self {
            child,
            listener_id,
            listener_fn,
        }
    }
}

impl<Child, ListenerFn, Renderer> Widget<Renderer> for EventFowarder<Child, ListenerFn>
where
    Child: Widget<Renderer>,
    ListenerFn: Fn(&Child::Outbound, &mut InboundEmitter<Child::Outbound>) + Send + Sync + 'static,
    Renderer: 'static,
{
    type State = Child::State;
    type Inbound = Child::Inbound;
    type Outbound = Child::Outbound;
    type PaintObject = Child::PaintObject;

    fn update(
        &self,
        children: &Children<Renderer>,
        state: &mut Self::State,
        event: &Self::Inbound,
        context: &mut OutboundEmitter<Self::Outbound>,
    ) -> bool {
        let result = self.child.update(children, state, event, context);
        let outbound_events = context.outbound_events();

        if !outbound_events.is_empty() {
            let mut inbound_emitter = context.create_inbound_emitter(self.listener_id);
            for outbound_event in outbound_events {
                (self.listener_fn)(outbound_event, &mut inbound_emitter);
            }
        }

        result
    }

    #[inline]
    fn should_render(
        &self,
        children: &Children<Renderer>,
        state: &Self::State,
        new_widget: &Self,
        new_children: &Children<Renderer>,
    ) -> bool {
        self.child.should_render(children, state, &new_widget.child, new_children)
    }

    #[inline]
    fn render(
        &self,
        children: &Children<Renderer>,
        state: &Self::State,
        element_id: ElementId,
    ) -> Children<Renderer> {
        self.child.render(children, state, element_id)
    }

    #[inline]
    fn lifecycle(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut Self::PaintObject,
        lifecycle: Lifecycle<&Self, &Children<Renderer>>,
        renderer: &mut Renderer,
        context: &mut InboundEmitter<Self::Inbound>,
    ) {
        self.child.lifecycle(children, paint_object, lifecycle.map(|widget| &widget.child), renderer, context)
    }

    #[inline]
    fn layout<'a>(
        &'a self,
        children: &Children<Renderer>,
        paint_object: &mut Self::PaintObject,
        box_constraints: BoxConstraints,
        element_id: ElementId,
        element_tree: &'a ElementTree<Renderer>,
        renderer: &mut Renderer,
        context: &mut InboundEmitter<Self::Inbound>,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        self.child.layout(children, paint_object, box_constraints, element_id, element_tree, renderer, context)
    }

    #[inline]
    fn draw(
        &self,
        children: &Children<Renderer>,
        paint_object: &mut Self::PaintObject,
        bounds: Rectangle,
        renderer: &mut Renderer,
        context: &mut InboundEmitter<Self::Inbound>,
    ) -> Option<Primitive> {
        self.child.draw(children, paint_object, bounds, renderer, context)
    }
}
