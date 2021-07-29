use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::event::{EventContext, EventManager, EventType};
use crate::generator::GeneratorState;
use crate::geometrics::{Point, Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::lifecycle::{Lifecycle, LifecycleContext};
use crate::slot_vec::SlotVec;
use crate::tree::walk::{walk_next_node, WalkDirection};
use crate::tree::NodeId;
use crate::widget::{PolymophicWidget, WidgetPod, WidgetTree};

#[derive(Debug)]
pub struct Painter<Handle> {
    paint_states: SlotVec<PaintState<Handle>>,
    event_manager: EventManager<Handle>,
}

#[derive(Debug)]
pub struct PaintState<Handle> {
    pub rectangle: Rectangle,
    pub mounted_widget: Option<Arc<dyn PolymophicWidget<Handle> + Send + Sync>>,
}

pub trait PaintContext<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

impl<Handle> Painter<Handle> {
    pub fn new() -> Self {
        Self {
            paint_states: SlotVec::new(),
            event_manager: EventManager::new(),
        }
    }

    pub fn layout(
        &mut self,
        target_id: NodeId,
        tree: &WidgetTree<Handle>,
        viewport_size: Size,
        force_layout: bool,
    ) -> Size {
        let mut layout_stack = Vec::new();
        let mut current_id = target_id;
        let mut current_layout = {
            let WidgetPod { widget, state, .. } = &*tree[current_id];
            widget.layout(
                current_id,
                BoxConstraints::tight(&viewport_size),
                tree,
                &mut **state.lock().unwrap(),
            )
        };
        let mut calculated_size = Size::ZERO;

        loop {
            match current_layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let WidgetPod {
                        widget,
                        state,
                        dirty,
                    } = &*tree[child_id];
                    if force_layout || *dirty {
                        let layout = widget.layout(
                            child_id,
                            child_box_constraints,
                            &tree,
                            &mut **state.lock().unwrap(),
                        );
                        layout_stack.push((child_id, layout));
                    } else {
                        calculated_size = self.paint_states[child_id].rectangle.size;
                    }
                }
                GeneratorState::Yielded(LayoutRequest::ArrangeChild(child_id, point)) => {
                    let mut paint_state = self.paint_states.get_or_insert_default(child_id);
                    paint_state.rectangle.point = point;
                    calculated_size = paint_state.rectangle.size;
                }
                GeneratorState::Complete(size) => {
                    let mut paint_state = self.paint_states.get_or_insert_default(current_id);
                    paint_state.rectangle.size = size;
                    calculated_size = size;

                    if let Some((next_id, next_layout)) = layout_stack.pop() {
                        current_id = next_id;
                        current_layout = next_layout;
                    } else {
                        break;
                    }
                }
            }
        }

        calculated_size
    }

    pub fn paint(
        &mut self,
        target_id: NodeId,
        tree: &mut WidgetTree<Handle>,
        paint_context: &mut dyn PaintContext<Handle>,
    ) {
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        let mut node_id = target_id;
        let mut direction = WalkDirection::Downward;

        loop {
            let mut node = &tree[node_id];

            loop {
                match direction {
                    WalkDirection::Downward | WalkDirection::Sideward => {
                        if node.dirty {
                            break;
                        }
                    }
                    WalkDirection::Upward => break,
                }

                if let Some((next_node_id, next_direction)) =
                    walk_next_node(node_id, target_id, node, &WalkDirection::Upward)
                {
                    node_id = next_node_id;
                    direction = next_direction;
                    node = &tree[node_id];
                } else {
                    break;
                }
            }

            let rectangle = self.paint_states[node_id].rectangle;

            if direction == WalkDirection::Downward {
                absolute_point += latest_point;
            } else if direction == WalkDirection::Upward {
                absolute_point -= rectangle.point;
            }

            latest_point = rectangle.point;

            if direction == WalkDirection::Downward || direction == WalkDirection::Sideward {
                let WidgetPod { widget, state, .. } = &**node;
                let absolute_rectangle = Rectangle {
                    point: absolute_point + rectangle.point,
                    size: rectangle.size,
                };

                let mut context = LifecycleContext {
                    event_manager: &mut self.event_manager,
                };

                let mounted_widget = &mut self.paint_states[node_id].mounted_widget;
                if let Some(old_widget) = mounted_widget.replace(widget.clone()) {
                    widget.lifecycle(
                        Lifecycle::OnUpdate(&*old_widget),
                        &mut **state.lock().unwrap(),
                        &mut context,
                    );
                } else {
                    widget.lifecycle(
                        Lifecycle::OnMount,
                        &mut **state.lock().unwrap(),
                        &mut context,
                    );
                }

                widget.paint(
                    &absolute_rectangle,
                    &mut **state.lock().unwrap(),
                    paint_context,
                );
            }

            if let Some((next_node_id, next_direction)) =
                walk_next_node(node_id, target_id, node, &direction)
            {
                node_id = next_node_id;
                direction = next_direction;
            } else {
                break;
            }
        }
    }

    pub fn dispose(&mut self, target_id: NodeId, tree: &mut WidgetTree<Handle>) {
        for (child_id, child) in tree.detach_subtree(target_id) {
            let WidgetPod {
                widget: child_widget,
                state: child_state,
                ..
            } = child.into_inner();
            let mut context = LifecycleContext {
                event_manager: &mut self.event_manager,
            };
            child_widget.lifecycle(
                Lifecycle::OnUnmount,
                &mut **child_state.lock().unwrap(),
                &mut context,
            );
            self.paint_states.remove(child_id);
        }
    }

    pub fn dispatch_events<EventType>(&mut self, event: EventType::Event, tree: &WidgetTree<Handle>)
    where
        Handle: fmt::Debug,
        EventType: self::EventType + 'static,
    {
        let boxed_event: Box<dyn Any> = Box::new(event);
        let mut context = EventContext {};
        for handler in self.event_manager.get::<EventType>() {
            handler.dispatch(tree, &boxed_event, &mut context)
        }
    }
}

impl<Handle> Default for PaintState<Handle> {
    fn default() -> Self {
        Self {
            rectangle: Rectangle::ZERO,
            mounted_widget: None,
        }
    }
}
