use std::fmt;
use std::mem;
use std::sync::mpsc::Sender;

use crate::base::{PhysicalSize, Point, Rectangle, Size};
use crate::bit_flags::BitFlags;
use crate::event::{EventManager, GenericEvent};
use crate::generator::GeneratorState;
use crate::graphics::renderer::Renderer;
use crate::slot_vec::SlotVec;
use crate::tree::walk::WalkDirection;
use crate::tree::{NodeId, Tree};
use crate::widget::null::Null;
use crate::widget::tree::{Patch, WidgetPod, WidgetTree};

use super::layout::{BoxConstraints, LayoutRequest};
use super::{PaintContext, PaintCycle};

#[derive(Debug)]
pub struct PaintTree<Renderer> {
    tree: WidgetTree<Renderer>,
    root_id: NodeId,
    paint_states: SlotVec<PaintState<Renderer>>,
    event_manager: EventManager<Renderer>,
}

#[derive(Debug)]
pub struct PaintState<Renderer> {
    bounds: Rectangle,
    absolute_point: Point,
    box_constraints: BoxConstraints,
    mounted_pod: Option<WidgetPod<Renderer>>,
    deleted_children: Vec<WidgetPod<Renderer>>,
    flags: BitFlags<PaintFlag>,
}

#[derive(Debug)]
pub enum PaintFlag {
    None = 0b000,
    Dirty = 0b001,
    NeedsLayout = 0b010,
    NeedsPaint = 0b100,
}

impl<Renderer: self::Renderer> PaintTree<Renderer> {
    pub fn new(window_size: PhysicalSize) -> Self {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::new(Null, Vec::new()));

        let mut paint_state = PaintState::default();
        paint_state.box_constraints = BoxConstraints::tight(window_size.into());

        let mut paint_states = SlotVec::new();
        paint_states.insert_at(root_id, paint_state);

        Self {
            tree,
            root_id,
            paint_states,
            event_manager: EventManager::new(),
        }
    }

    pub fn apply_patch(&mut self, patch: Patch<Renderer>) {
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
                paint_state.flags |= [
                    PaintFlag::Dirty,
                    PaintFlag::NeedsLayout,
                    PaintFlag::NeedsPaint,
                ];
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

    pub fn layout_root(&mut self, window_size: PhysicalSize, renderer: &mut Renderer) {
        self.do_layout(
            self.root_id,
            BoxConstraints::tight(window_size.into()),
            renderer,
        );
    }

    pub fn layout_subtree(&mut self, target_id: NodeId, renderer: &mut Renderer) {
        let mut current_id = target_id;
        let mut box_constraints = self.paint_states[target_id].box_constraints;

        while let Some(parent_id) = self.do_layout(current_id, box_constraints, renderer) {
            current_id = parent_id;
            box_constraints = self.paint_states[parent_id].box_constraints;
        }
    }

    fn do_layout(
        &mut self,
        initial_id: NodeId,
        initial_box_constraints: BoxConstraints,
        renderer: &mut Renderer,
    ) -> Option<NodeId> {
        let initial_node = &self.tree[initial_id];

        let mut context = (initial_id, initial_box_constraints, {
            let WidgetPod { widget, state, .. } = &**initial_node;
            widget.layout(
                initial_id,
                initial_box_constraints,
                &self.tree,
                &mut **state.lock().unwrap(),
                renderer,
            )
        });
        let mut context_stack = Vec::new();
        let mut calculated_size = Size::ZERO;

        loop {
            let (_, _, ref mut layout) = context;

            match layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let paint_state = &self.paint_states[child_id];
                    if paint_state.flags.contains(PaintFlag::NeedsLayout)
                        || paint_state.box_constraints != child_box_constraints
                    {
                        let WidgetPod { widget, state, .. } = &*self.tree[child_id];
                        context_stack.push(context);
                        context = (
                            child_id,
                            child_box_constraints,
                            widget.layout(
                                child_id,
                                child_box_constraints,
                                &self.tree,
                                &mut **state.lock().unwrap(),
                                renderer,
                            ),
                        );
                    } else {
                        calculated_size = paint_state.bounds.size();
                    }
                }
                GeneratorState::Yielded(LayoutRequest::ArrangeChild(child_id, point)) => {
                    let mut paint_state = &mut self.paint_states[child_id];
                    paint_state.bounds.x = point.x;
                    paint_state.bounds.y = point.y;
                    calculated_size = paint_state.bounds.size();
                }
                GeneratorState::Complete(size) => {
                    let (node_id, box_constraints, _) = context;

                    let mut paint_state = &mut self.paint_states[node_id];
                    paint_state.box_constraints = box_constraints;
                    paint_state.flags -= PaintFlag::NeedsLayout;

                    let size_changed = size != paint_state.bounds.size();
                    if size_changed {
                        paint_state.bounds.width = size.width;
                        paint_state.bounds.height = size.height;
                        paint_state.flags |= [PaintFlag::Dirty, PaintFlag::NeedsPaint];
                    }

                    if let Some(next_context) = context_stack.pop() {
                        context = next_context;
                        calculated_size = size;
                    } else {
                        return if size_changed {
                            initial_node.parent()
                        } else {
                            None
                        };
                    }
                }
            }
        }
    }

    pub fn paint(&mut self, renderer: &mut Renderer) -> Renderer::DrawOp {
        let mut tree_walker = self.tree.walk(self.root_id);

        let mut absolute_point = Point { x: 0.0, y: 0.0 };
        let mut latest_point = Point { x: 0.0, y: 0.0 };

        let mut child_draw_op = Renderer::DrawOp::default();
        let mut current_draw_op = Renderer::DrawOp::default();

        while let Some((node_id, node, direction)) = tree_walker
            .next_match(|node_id, _| self.paint_states[node_id].flags.contains(PaintFlag::Dirty))
        {
            let paint_state = &mut self.paint_states[node_id];
            let bounds = paint_state.bounds;
            let needs_repaint = paint_state.flags.contains(PaintFlag::NeedsPaint);

            match direction {
                WalkDirection::Downward => {
                    absolute_point += latest_point;
                    latest_point = bounds.point();
                    if !needs_repaint || node.has_child() {
                        continue;
                    }
                }
                WalkDirection::Upward => {
                    absolute_point -= bounds.point();
                    latest_point = bounds.point();
                    if !needs_repaint {
                        continue;
                    }
                    child_draw_op = mem::take(&mut current_draw_op);
                }
                WalkDirection::Sideward => {
                    latest_point = bounds.point();
                    if !needs_repaint || node.has_child() {
                        continue;
                    }
                }
            }

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
                    renderer,
                    &mut context,
                );
            }

            let widget_pod = &**node;
            let WidgetPod { widget, state, children, ..  } = widget_pod;
            let absolute_bounds = bounds.offset(absolute_point.x, absolute_point.y);

            let draw_op = widget.draw(
                mem::take(&mut child_draw_op),
                absolute_bounds,
                &mut **state.lock().unwrap(),
                renderer,
                &mut context,
            );

            if let Some(old_widget_pod) = paint_state.mounted_pod.replace(widget_pod.clone()) {
                widget.on_paint_cycle(
                    PaintCycle::DidUpdate(
                        &children,
                        &*old_widget_pod.widget,
                        &old_widget_pod.children,
                    ),
                    &mut **state.lock().unwrap(),
                    renderer,
                    &mut context,
                );
            } else {
                widget.on_paint_cycle(
                    PaintCycle::DidMount(children),
                    &mut **state.lock().unwrap(),
                    renderer,
                    &mut context,
                );
            }

            current_draw_op = current_draw_op + draw_op;

            paint_state.absolute_point = absolute_point;
            paint_state.flags -= PaintFlag::NeedsPaint;
        }

        current_draw_op
    }

    pub fn dispatch(&self, event: &GenericEvent, update_notifier: &Sender<NodeId>) {
        for handler in self.event_manager.get(&event.type_id) {
            handler.dispatch(&self.tree, &event.payload, update_notifier)
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

impl<Renderer> fmt::Display for PaintTree<Renderer> {
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
                write!(f, " x=\"{}\"", paint_state.bounds.x.round())?;
                write!(f, " y=\"{}\"", paint_state.bounds.y.round())?;
                write!(f, " width=\"{}\"", paint_state.bounds.width.round())?;
                write!(f, " height=\"{}\"", paint_state.bounds.height.round())?;
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

impl<Renderer> Default for PaintState<Renderer> {
    fn default() -> Self {
        Self {
            bounds: Rectangle::ZERO,
            absolute_point: Point::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            mounted_pod: None,
            deleted_children: Vec::new(),
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
