use std::any::Any;
use std::sync::Arc;
use std::fmt;
use std::mem;
use std::sync::mpsc::Sender;

use crate::bit_flags::BitFlags;
use crate::event::{EventHandler, EventManager, EventType, HandlerId};
use crate::generator::GeneratorState;
use crate::geometrics::{Point, Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::lifecycle::Lifecycle;
use crate::slot_vec::SlotVec;
use crate::tree::walk::WalkDirection;
use crate::tree::{NodeId, Tree};
use crate::widget::element::{BoxedWidget, Children};
use crate::widget::null::Null;
use crate::widget::tree::{Patch, WidgetPod, WidgetTree};

#[derive(Debug)]
pub struct PaintTree<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    paint_states: SlotVec<PaintState<Handle>>,
    event_manager: EventManager<Handle>,
    update_notifier: Sender<NodeId>,
}

#[derive(Debug)]
pub struct PaintState<Handle> {
    pub rectangle: Rectangle,
    pub mounted_widget: Option<(BoxedWidget<Handle>, Children<Handle>)>,
    pub deleted_children: Vec<WidgetPod<Handle>>,
    pub hint: PaintHint,
    pub flags: BitFlags<PaintFlag>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PaintHint {
    Always,
    Once,
}

#[derive(Debug)]
pub enum PaintFlag {
    None = 0b00,
    NeedsLayout = 0b01,
    NeedsPaint = 0b10,
}

pub trait Painter<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

pub struct PaintContext<'a, Handle> {
    event_manager: &'a mut EventManager<Handle>,
    painter: &'a mut dyn Painter<Handle>,
}

impl<Handle> PaintTree<Handle> {
    pub fn new(update_notifier: Sender<NodeId>) -> Self {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::new(Null, Vec::new()));

        let mut paint_states = SlotVec::new();
        paint_states.insert_at(root_id, PaintState::default());

        Self {
            tree,
            root_id,
            paint_states,
            event_manager: EventManager::new(),
            update_notifier,
        }
    }

    pub fn apply_patch(&mut self, patch: Patch<Handle>) {
        match patch {
            Patch::Append(parent_id, widget_pod) => {
                let child_id = self.tree.append_child(parent_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.emit_changes(child_id);
            }
            Patch::Insert(ref_id, widget_pod) => {
                let child_id = self.tree.insert_before(ref_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.emit_changes(child_id);
            }
            Patch::Update(target_id, new_element) => {
                self.tree[target_id].update(new_element);
                if self.paint_states[target_id].mark_as_dirty() {
                    self.emit_changes(target_id);
                }
            }
            Patch::Placement(target_id, ref_id) => {
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            Patch::Remove(target_id) => {
                let (node, subtree) = self.tree.detach(target_id);
                let mut deleted_children = Vec::new();

                for (child_id, child) in subtree {
                    self.paint_states.remove(child_id);
                    deleted_children.push(child.into_inner());
                }

                if let Some(parent_id) = node.parent() {
                    self.paint_states.remove(target_id);
                    deleted_children.push(node.into_inner());

                    let paint_state = &mut self.paint_states[parent_id];
                    paint_state.deleted_children = deleted_children;
                    if paint_state.mark_as_dirty() {
                        self.emit_changes(parent_id);
                    }
                } else {
                    unreachable!("Root removed");
                }
            }
        }
    }

    pub fn layout(&mut self, viewport_size: Size, force_layout: bool) -> Size {
        let mut layout_stack = Vec::new();
        let mut calculated_size = Size::ZERO;

        let mut current_id = self.root_id;
        let mut current_layout = {
            let WidgetPod { widget, state, .. } = &*self.tree[current_id];
            widget.layout(
                current_id,
                BoxConstraints::tight(&viewport_size),
                &self.tree,
                &mut **state.lock().unwrap(),
            )
        };

        loop {
            match current_layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let WidgetPod { widget, state, .. } = &*self.tree[child_id];
                    if force_layout || self.paint_states[current_id].flags.contains(PaintFlag::NeedsLayout) {
                        layout_stack.push((current_id, current_layout));
                        current_id = child_id;
                        current_layout = widget.layout(
                            child_id,
                            child_box_constraints,
                            &self.tree,
                            &mut **state.lock().unwrap(),
                        );
                    } else {
                        calculated_size = self.paint_states[child_id].rectangle.size;
                    }
                }
                GeneratorState::Yielded(LayoutRequest::ArrangeChild(child_id, point)) => {
                    let mut paint_state = &mut self.paint_states[child_id];
                    paint_state.rectangle.point = point;
                    calculated_size = paint_state.rectangle.size;
                }
                GeneratorState::Complete(size) => {
                    let mut paint_state = &mut self.paint_states[current_id];

                    paint_state.flags ^= PaintFlag::NeedsLayout;

                    if paint_state.rectangle.size != size {
                        paint_state.rectangle.size = size;
                        paint_state.flags |= PaintFlag::NeedsPaint;
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

    pub fn paint(&mut self, painter: &mut dyn Painter<Handle>) {
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        let mut walker = self.tree.walk(self.root_id);

        while let Some((node_id, node, direction)) =
            walker.next_if(|node_id, _| self.paint_states[node_id].flags.contains(PaintFlag::NeedsPaint))
        {
            let rectangle = self.paint_states[node_id].rectangle;

            if direction == WalkDirection::Downward {
                absolute_point += latest_point;
            } else if direction == WalkDirection::Upward {
                absolute_point -= rectangle.point;
            }

            latest_point = rectangle.point;

            if direction == WalkDirection::Downward || direction == WalkDirection::Sideward {
                let mut context = PaintContext {
                    event_manager: &mut self.event_manager,
                    painter,
                };

                for widget_pod in mem::take(&mut self.paint_states[node_id].deleted_children) {
                    let WidgetPod {
                        widget,
                        state,
                        children,
                        ..
                    } = widget_pod;
                    widget.lifecycle(
                        Lifecycle::OnUnmount(&children),
                        &mut **state.lock().unwrap(),
                        &mut context,
                    );
                }

                let WidgetPod {
                    widget,
                    state,
                    children,
                    ..
                } = &**node;
                let paint_state = &mut self.paint_states[node_id];

                if let Some((old_widget, old_children)) = paint_state
                    .mounted_widget
                    .replace((widget.clone(), children.clone()))
                {
                    widget.lifecycle(
                        Lifecycle::OnUpdate(&*old_widget, &children, &old_children),
                        &mut **state.lock().unwrap(),
                        &mut context,
                    );
                } else {
                    widget.lifecycle(
                        Lifecycle::OnMount(children),
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
                    &mut context,
                );

                paint_state.flags ^= PaintFlag::NeedsPaint;
            }
        }
    }

    pub fn dispatch<EventType>(&self, event: EventType::Event)
    where
        Handle: fmt::Debug,
        EventType: self::EventType + 'static,
    {
        let boxed_event: Box<dyn Any> = Box::new(event);
        for handler in self.event_manager.get::<EventType>() {
            handler.dispatch(&self.tree, &boxed_event, &self.update_notifier)
        }
    }

    fn emit_changes(&mut self, target_id: NodeId) {
        for (parent_id, _) in self.tree.ancestors(target_id) {
            let paint_state = &mut self.paint_states[parent_id];
            if paint_state.flags.intersects([PaintFlag::NeedsLayout, PaintFlag::NeedsPaint]) {
                break;
            }
            paint_state.flags |= PaintFlag::NeedsLayout;
            paint_state.flags |= PaintFlag::NeedsPaint;
        }
    }
}

impl<Handle> fmt::Display for PaintTree<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.format(
            f,
            self.root_id,
            |f, node_id, node| {
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
                if paint_state.flags.contains(PaintFlag::NeedsLayout) {
                    write!(f, " needs_layout")?;
                }
                if paint_state.flags.contains(PaintFlag::NeedsPaint) {
                    write!(f, " needs_paint")?;
                }
                write!(f, ">")?;
                Ok(())
            },
            |f, _, node| write!(f, "</{}>", node.widget.name()),
        )
    }
}

impl<Handle> PaintState<Handle> {
    fn mark_as_dirty(&mut self) -> bool {
        if self.hint == PaintHint::Always {
            self.flags |= PaintFlag::NeedsLayout;
            self.flags |= PaintFlag::NeedsPaint;
            true
        } else {
            false
        }
    }
}

impl<Handle> Default for PaintState<Handle> {
    fn default() -> Self {
        Self {
            rectangle: Rectangle::ZERO,
            mounted_widget: None,
            deleted_children: Vec::new(),
            hint: PaintHint::Always,
            flags: [PaintFlag::NeedsLayout, PaintFlag::NeedsPaint].into(),
        }
    }
}

impl Into<usize> for PaintFlag {
    fn into(self) -> usize {
        self as _
    }
}

impl<'a, Handle> PaintContext<'a, Handle> {
    pub fn add_handler(
        &mut self,
        handler: Arc<dyn EventHandler<Handle> + Send + Sync>,
    ) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(
        &mut self,
        handler_id: HandlerId,
    ) -> Arc<dyn EventHandler<Handle> + Send + Sync> {
        self.event_manager.remove(handler_id)
    }
}

impl<'a, Handle> Painter<Handle> for PaintContext<'a, Handle> {
    #[inline]
    fn handle(&self) -> &Handle {
        self.painter.handle()
    }

    #[inline]
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        self.painter.fill_rectangle(color, rectangle)
    }

    #[inline]
    fn commit(&mut self, rectangle: &Rectangle) {
        self.painter.commit(rectangle)
    }
}
