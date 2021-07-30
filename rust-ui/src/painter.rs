use std::any::Any;
use std::fmt;
use std::ptr;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::event::{EventManager, EventType};
use crate::generator::GeneratorState;
use crate::geometrics::{Point, Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::lifecycle::{Lifecycle, LifecycleContext};
use crate::slot_vec::SlotVec;
use crate::tree::NodeId;
use crate::tree::walk::WalkDirection;
use crate::widget::tree::{WidgetPod, WidgetFlag, WidgetTree};
use crate::widget::{PolymophicWidget};

#[derive(Debug)]
pub struct Painter<Handle> {
    paint_states: SlotVec<PaintState<Handle>>,
    event_manager: EventManager<Handle>,
    update_notifier: Sender<NodeId>,
}

#[derive(Debug)]
pub struct PaintState<Handle> {
    pub rectangle: Rectangle,
    pub mounted_widget: Option<Arc<dyn PolymophicWidget<Handle> + Send + Sync>>,
    pub needs_paint: bool,
}

pub trait PaintContext<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

impl<Handle> Painter<Handle> {
    pub fn new(update_notifier: Sender<NodeId>) -> Self {
        Self {
            paint_states: SlotVec::new(),
            event_manager: EventManager::new(),
            update_notifier,
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
        let mut calculated_size = Size::ZERO;

        let mut current_id = target_id;
        let mut current_layout = {
            let WidgetPod { widget, state, .. } = &*tree[target_id];
            widget.layout(
                current_id,
                BoxConstraints::tight(&viewport_size),
                tree,
                &mut **state.lock().unwrap(),
            )
        };

        loop {
            match current_layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let WidgetPod {
                        widget,
                        state,
                        ..
                    } = &*tree[child_id];
                    if force_layout || tree[current_id].flags.contains([WidgetFlag::Fresh, WidgetFlag::Dirty]) {
                        layout_stack.push((current_id, current_layout));
                        current_id = child_id;
                        current_layout = widget.layout(
                            child_id,
                            child_box_constraints,
                            &tree,
                            &mut **state.lock().unwrap(),
                        );
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

                    if paint_state.rectangle.size != size {
                        paint_state.rectangle.size = size;
                        paint_state.needs_paint = true;
                    } else {
                        let WidgetPod { flags, .. } = &*tree[current_id];
                        paint_state.needs_paint = flags.contains([WidgetFlag::Fresh, WidgetFlag::Dirty]);
                    }

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
        old_tree: &WidgetTree<Handle>,
        new_tree: &WidgetTree<Handle>,
        paint_context: &mut dyn PaintContext<Handle>,
    ) {
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        let mut walker = new_tree.walk(target_id);

        while let Some((node_id, node, direction)) = walker.next_if(|node_id, _| {
            self.paint_states[node_id].needs_paint
        }) {
            let rectangle = self.paint_states[node_id].rectangle;

            if direction == WalkDirection::Downward {
                absolute_point += latest_point;
            } else if direction == WalkDirection::Upward {
                absolute_point -= rectangle.point;
            }

            latest_point = rectangle.point;

            if direction == WalkDirection::Downward || direction == WalkDirection::Sideward {
                if !ptr::eq(new_tree, old_tree) {
                    for &child_id in node.deleted_children.iter() {
                        self.dispose(child_id, &*old_tree[child_id]);

                        for (child_id, child) in old_tree.pre_ordered_descendants(child_id) {
                            self.dispose(child_id, child);
                        }
                    }
                }

                let WidgetPod { widget, state, .. } = &*new_tree[node_id];
                let paint_state = &mut self.paint_states[node_id];

                let mut context = LifecycleContext {
                    event_manager: &mut self.event_manager,
                };

                if let Some(old_widget) = paint_state.mounted_widget.replace(widget.clone()) {
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

                let absolute_rectangle = Rectangle {
                    point: absolute_point + rectangle.point,
                    size: rectangle.size,
                };

                widget.paint(
                    &absolute_rectangle,
                    &mut **state.lock().unwrap(),
                    paint_context,
                );

                paint_state.needs_paint = false;
            }
        }
    }

    pub fn dispatch<EventType>(&mut self, event: EventType::Event, tree: &WidgetTree<Handle>)
    where
        Handle: fmt::Debug,
        EventType: self::EventType + 'static,
    {
        let boxed_event: Box<dyn Any> = Box::new(event);
        for handler in self.event_manager.get::<EventType>() {
            handler.dispatch(tree, &boxed_event, &self.update_notifier)
        }
    }

    pub fn format_tree<'a>(
        &'a self,
        target_id: NodeId,
        tree: &'a WidgetTree<Handle>,
    ) -> impl fmt::Display + 'a {
        tree.to_formatter(
            target_id,
            move |f, node_id, node| {
                let paint_state = &self.paint_states[node_id];
                write!(f, "<{}", node.widget.name())?;
                write!(f, " id=\"{}\"", node_id)?;
                if let Some(key) = node.key {
                    write!(f, " key=\"{}\"", key)?;
                }
                write!(f, " x=\"{}\"", paint_state.rectangle.point.x)?;
                write!(f, " y=\"{}\"", paint_state.rectangle.point.y)?;
                write!(f, " width=\"{}\"", paint_state.rectangle.size.width)?;
                write!(f, " height=\"{}\"", paint_state.rectangle.size.height)?;
                if node.flags.contains(WidgetFlag::Fresh) {
                    write!(f, " fresh")?;
                }
                if node.flags.contains(WidgetFlag::Dirty) {
                    write!(f, " dirty")?;
                }
                write!(f, ">")?;
                Ok(())
            },
            |f, _, node| write!(f, "</{}>", node.widget.name()),
        )
    }

    fn dispose(&mut self, target_id: NodeId, widget_pod: &WidgetPod<Handle>) {
        let WidgetPod { widget, state, ..  } = widget_pod;
        let mut context = LifecycleContext {
            event_manager: &mut self.event_manager,
        };
        widget.lifecycle(
            Lifecycle::OnUnmount,
            &mut **state.lock().unwrap(),
            &mut context,
        );
        self.paint_states.remove(target_id);
    }
}

impl<Handle> Default for PaintState<Handle> {
    fn default() -> Self {
        Self {
            rectangle: Rectangle::ZERO,
            mounted_widget: None,
            needs_paint: true,
        }
    }
}
