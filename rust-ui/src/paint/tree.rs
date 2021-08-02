use std::fmt;
use std::mem;
use std::sync::mpsc::Sender;

use crate::bit_flags::BitFlags;
use crate::event::{EventManager, GenericEvent};
use crate::generator::GeneratorState;
use crate::geometrics::{Point, Rectangle, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::slot_vec::SlotVec;
use crate::tree::walk::WalkDirection;
use crate::tree::{NodeId, Tree};
use crate::widget::null::Null;
use crate::widget::tree::{Patch, WidgetPod, WidgetTree};

use super::{PaintContext, PaintCycle, PaintHint};

#[derive(Debug)]
pub struct PaintTree<Painter> {
    tree: WidgetTree<Painter>,
    root_id: NodeId,
    paint_states: SlotVec<PaintState<Painter>>,
    event_manager: EventManager<Painter>,
    update_notifier: Sender<NodeId>,
}

#[derive(Debug)]
pub struct PaintState<Painter> {
    rectangle: Rectangle,
    box_constraints: BoxConstraints,
    mounted_pod: Option<WidgetPod<Painter>>,
    deleted_children: Vec<WidgetPod<Painter>>,
    hint: PaintHint,
    flags: BitFlags<PaintFlag>,
}

#[derive(Debug)]
pub enum PaintFlag {
    None = 0b000,
    Dirty = 0b001,
    NeedsLayout = 0b010,
    NeedsPaint = 0b100,
}

impl<Painter> PaintTree<Painter> {
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

    pub fn apply_patch(&mut self, patch: Patch<Painter>) {
        match patch {
            Patch::Append(parent_id, widget_pod) => {
                let child_id = self.tree.append_child(parent_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.mark_parents_as_dirty(child_id);
            }
            Patch::Insert(ref_id, widget_pod) => {
                let child_id = self.tree.insert_before(ref_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.mark_parents_as_dirty(child_id);
            }
            Patch::Update(target_id, new_element) => {
                self.tree[target_id].update(new_element);
                let paint_state = &mut self.paint_states[target_id];
                paint_state.flags |= [PaintFlag::Dirty, PaintFlag::NeedsLayout];
                if matches!(paint_state.hint, PaintHint::Always) {
                    paint_state.flags |= PaintFlag::NeedsPaint;
                }
                self.mark_parents_as_dirty(target_id);
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

                    let parent_paint_state = &mut self.paint_states[parent_id];
                    parent_paint_state.deleted_children = deleted_children;
                } else {
                    unreachable!("Root cannot be removed");
                }
            }
        }
    }

    pub fn layout(&mut self, viewport: Size, painter: &mut Painter) {
        self.do_layout(self.root_id, BoxConstraints::tight(viewport), painter);
    }

    pub fn layout_subtree(&mut self, target_id: NodeId, viewport: Size, painter: &mut Painter) {
        let mut current_id = target_id;
        let mut box_constraints = if target_id == self.root_id {
            BoxConstraints::tight(viewport)
        } else {
            self.paint_states[target_id].box_constraints
        };
        while let Some(parent_id) = self.do_layout(current_id, box_constraints, painter) {
            current_id = parent_id;
            box_constraints = if parent_id == self.root_id {
                BoxConstraints::tight(viewport)
            } else {
                self.paint_states[parent_id].box_constraints
            };
        }
    }

    fn do_layout(&mut self, target_id: NodeId, box_constraints: BoxConstraints, painter: &mut Painter) -> Option<NodeId> {
        let target_node = &self.tree[target_id];

        let mut current_id = target_id;
        let mut current_box_constraints = box_constraints;
        let mut current_layout = {
            let WidgetPod { widget, state, .. } = &**target_node;
            widget.layout(
                target_id,
                box_constraints,
                &self.tree,
                &mut **state.lock().unwrap(),
                painter,
            )
        };

        let mut layout_stack = Vec::new();
        let mut calculated_size = Size::ZERO;

        loop {
            match current_layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let paint_state = &self.paint_states[child_id];
                    if paint_state.flags.contains(PaintFlag::NeedsLayout)
                        || paint_state.box_constraints != child_box_constraints
                    {
                        let WidgetPod { widget, state, .. } = &*self.tree[child_id];
                        layout_stack.push((current_id, current_box_constraints, current_layout));
                        current_id = child_id;
                        current_box_constraints = child_box_constraints;
                        current_layout = widget.layout(
                            child_id,
                            child_box_constraints,
                            &self.tree,
                            &mut **state.lock().unwrap(),
                            painter,
                        );
                    } else {
                        calculated_size = paint_state.rectangle.size;
                    }
                }
                GeneratorState::Yielded(LayoutRequest::ArrangeChild(child_id, point)) => {
                    let mut paint_state = &mut self.paint_states[child_id];
                    paint_state.rectangle.point = point;
                    calculated_size = paint_state.rectangle.size;
                }
                GeneratorState::Complete(size) => {
                    let mut paint_state = &mut self.paint_states[current_id];
                    paint_state.box_constraints = current_box_constraints;
                    paint_state.flags -= PaintFlag::NeedsLayout;

                    let size_changed = paint_state.rectangle.size != size;
                    if size_changed {
                        paint_state.rectangle.size = size;
                        paint_state.flags |= PaintFlag::Dirty;
                        if matches!(paint_state.hint, PaintHint::Always) {
                            paint_state.flags |= PaintFlag::NeedsPaint;
                        }
                    }

                    if let Some((next_id, next_box_constraints, next_layout)) = layout_stack.pop() {
                        current_id = next_id;
                        current_box_constraints = next_box_constraints;
                        current_layout = next_layout;
                        calculated_size = size;
                    } else {
                        return if size_changed {
                            target_node.parent()
                        } else {
                            None
                        };
                    }
                }
            }
        }
    }

    pub fn paint(&mut self, painter: &mut Painter) {
        let mut walker = self.tree.walk(self.root_id);
        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        while let Some((node_id, node, direction)) =
            walker.next_if(|node_id, _| self.paint_states[node_id].flags.contains(PaintFlag::Dirty))
        {
            let rectangle = self.paint_states[node_id].rectangle;

            match direction {
                WalkDirection::Downward => absolute_point += latest_point,
                WalkDirection::Upward => absolute_point -= rectangle.point,
                _ => {}
            }

            latest_point = rectangle.point;

            if matches!(direction, WalkDirection::Upward) {
                continue;
            }

            let paint_state = &mut self.paint_states[node_id];

            if paint_state.flags.contains(PaintFlag::NeedsPaint) {
                let mut context = PaintContext {
                    event_manager: &mut self.event_manager,
                };

                for WidgetPod {
                    widget,
                    state,
                    children,
                    ..
                } in mem::take(&mut paint_state.deleted_children)
                {
                    widget.on_paint_cycle(
                        PaintCycle::DidUnmount(&children),
                        &mut **state.lock().unwrap(),
                        painter,
                        &mut context,
                    );
                }

                let widget_pod = &**node;
                let WidgetPod {
                    widget,
                    state,
                    children,
                    ..
                } = widget_pod;

                if let Some(old_widget_pod) = paint_state.mounted_pod.replace(widget_pod.clone()) {
                    widget.on_paint_cycle(
                        PaintCycle::DidUpdate(
                            &children,
                            &*old_widget_pod.widget,
                            &old_widget_pod.children,
                        ),
                        &mut **state.lock().unwrap(),
                        painter,
                        &mut context,
                    );
                } else {
                    widget.on_paint_cycle(
                        PaintCycle::DidMount(children),
                        &mut **state.lock().unwrap(),
                        painter,
                        &mut context,
                    );
                }

                let absolute_rectangle = Rectangle {
                    point: absolute_point + rectangle.point,
                    size: rectangle.size,
                };

                paint_state.hint = widget.paint(
                    &absolute_rectangle,
                    &mut **state.lock().unwrap(),
                    painter,
                    &mut context,
                );

                paint_state.flags -= PaintFlag::NeedsPaint;
            }
        }
    }

    pub fn dispatch(&self, event: &GenericEvent) {
        for handler in self.event_manager.get(&event.type_id) {
            handler.dispatch(&self.tree, &event.payload, &self.update_notifier)
        }
    }

    fn mark_parents_as_dirty(&mut self, target_id: NodeId) {
        for (parent_id, _) in self.tree.ancestors(target_id) {
            let paint_state = &mut self.paint_states[parent_id];
            if paint_state.flags.contains(PaintFlag::Dirty) {
                break;
            }
            paint_state.flags |= PaintFlag::Dirty;
        }
    }
}

impl<Painter> fmt::Display for PaintTree<Painter> {
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
                write!(f, " x=\"{}\"", paint_state.rectangle.point.x.round())?;
                write!(f, " y=\"{}\"", paint_state.rectangle.point.y.round())?;
                write!(f, " width=\"{}\"", paint_state.rectangle.size.width.round())?;
                write!(
                    f,
                    " height=\"{}\"",
                    paint_state.rectangle.size.height.round()
                )?;
                if paint_state.flags.contains(PaintFlag::Dirty) {
                    write!(f, " dirty")?;
                }
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

impl<Painter> Default for PaintState<Painter> {
    fn default() -> Self {
        Self {
            rectangle: Rectangle::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            mounted_pod: None,
            deleted_children: Vec::new(),
            hint: PaintHint::Always,
            flags: [
                PaintFlag::Dirty,
                PaintFlag::NeedsLayout,
                PaintFlag::NeedsPaint,
            ]
            .into(),
        }
    }
}

impl Into<usize> for PaintFlag {
    fn into(self) -> usize {
        self as _
    }
}
