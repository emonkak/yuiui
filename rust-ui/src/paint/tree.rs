use std::fmt;
use std::mem;
use std::sync::mpsc::Sender;

use crate::event::{EventManager, GenericEvent};
use crate::geometrics::{Point, Rectangle, Size, Vector};
use crate::graphics::{Primitive, Renderer};
use crate::support::bit_flags::BitFlags;
use crate::support::generator::GeneratorState;
use crate::support::slot_vec::SlotVec;
use crate::support::tree::walk::WalkDirection;
use crate::widget::{create_widget_tree, WidgetId, WidgetPod, WidgetTree, WidgetTreePatch};

use super::context::PaintContext;
use super::layout::{BoxConstraints, LayoutRequest};
use super::lifecycle::Lifecycle;

pub struct PaintTree<Renderer> {
    tree: WidgetTree<Renderer>,
    root_id: WidgetId,
    paint_states: SlotVec<PaintState<Renderer>>,
    event_manager: EventManager,
    update_sender: Sender<WidgetId>,
}

#[derive(Debug)]
struct PaintState<Renderer> {
    bounds: Rectangle,
    absolute_translation: Vector,
    box_constraints: BoxConstraints,
    mounted_pod: Option<WidgetPod<Renderer>>,
    deleted_children: Vec<WidgetPod<Renderer>>,
    flags: BitFlags<PaintFlag>,
    draw_cache: Option<Primitive>,
}

#[derive(Debug)]
enum PaintFlag {
    Dirty = 0b001,
    NeedsLayout = 0b010,
    NeedsPaint = 0b100,
}

impl<Renderer> PaintTree<Renderer> {
    pub fn new(viewport_size: Size, update_sender: Sender<WidgetId>) -> Self {
        let (tree, root_id) = create_widget_tree();
        let mut paint_states = SlotVec::new();

        paint_states.insert_at(
            root_id,
            PaintState {
                box_constraints: BoxConstraints::tight(viewport_size),
                ..PaintState::default()
            },
        );

        Self {
            tree,
            root_id,
            paint_states,
            event_manager: EventManager::new(),
            update_sender,
        }
    }

    pub fn mark_update_root(&mut self, widget_id: WidgetId) {
        self.paint_states[widget_id].flags |= [
            PaintFlag::Dirty,
            PaintFlag::NeedsLayout,
            PaintFlag::NeedsPaint,
        ];
        self.mark_parents_as_dirty(widget_id);
    }

    pub fn apply_patch(&mut self, patch: WidgetTreePatch<Renderer>) {
        match patch {
            WidgetTreePatch::Append(parent_id, widget_pod) => {
                let child_id = self.tree.append_child(parent_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.mark_parents_as_dirty(child_id);
            }
            WidgetTreePatch::Insert(ref_id, widget_pod) => {
                let child_id = self.tree.insert_before(ref_id, widget_pod);
                self.paint_states.insert_at(child_id, PaintState::default());
                self.mark_parents_as_dirty(child_id);
            }
            WidgetTreePatch::Update(target_id, new_element) => {
                self.tree[target_id].update(new_element);
                let paint_state = &mut self.paint_states[target_id];
                paint_state.flags |= [
                    PaintFlag::Dirty,
                    PaintFlag::NeedsLayout,
                    PaintFlag::NeedsPaint,
                ];
                self.mark_parents_as_dirty(target_id);
            }
            WidgetTreePatch::Placement(target_id, ref_id) => {
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            WidgetTreePatch::Remove(target_id) => {
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

    pub fn layout_root(&mut self, viewport_size: Size, renderer: &mut Renderer) {
        self.layout(self.root_id, BoxConstraints::tight(viewport_size), renderer);
    }

    pub fn layout_subtree(&mut self, target_id: WidgetId, renderer: &mut Renderer) {
        let mut current_id = target_id;
        let mut box_constraints = self.paint_states[target_id].box_constraints;

        while let Some(parent_id) = self.layout(current_id, box_constraints, renderer) {
            current_id = parent_id;
            box_constraints = self.paint_states[parent_id].box_constraints;
        }
    }

    fn layout(
        &mut self,
        initial_id: WidgetId,
        initial_box_constraints: BoxConstraints,
        renderer: &mut Renderer,
    ) -> Option<WidgetId> {
        let initial_node = &self.tree[initial_id];

        let mut context = (initial_id, initial_box_constraints, {
            let WidgetPod {
                widget,
                children,
                state,
                ..
            } = (**initial_node).clone();
            widget.layout(
                children,
                state,
                initial_box_constraints,
                initial_id,
                &self.tree,
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
                        let WidgetPod {
                            widget,
                            children,
                            state,
                            ..
                        } = (*self.tree[child_id]).clone();
                        context_stack.push(context);
                        context = (
                            child_id,
                            child_box_constraints,
                            widget.layout(
                                children,
                                state,
                                child_box_constraints,
                                child_id,
                                &self.tree,
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
                    let (widget_id, box_constraints, _) = context;

                    let mut paint_state = &mut self.paint_states[widget_id];
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

    pub fn paint(&mut self, pipeline: &mut Renderer::Pipeline, renderer: &mut Renderer)
    where
        Renderer: self::Renderer,
    {
        let mut tree_walker = self.tree.walk(self.root_id);

        let mut absolute_translation = Vector::ZERO;
        let mut latest_point = Point::ZERO;
        let mut depth = 0;

        while let Some((widget_id, node, direction)) = tree_walker
            .next_match(|widget_id, _| self.paint_states[widget_id].flags.contains(PaintFlag::Dirty))
        {
            let paint_state = &mut self.paint_states[widget_id];
            let bounds = paint_state.bounds;

            let draw_phase;
            let lifecycle_phase;

            match direction {
                WalkDirection::Downward => {
                    absolute_translation = absolute_translation + latest_point.into();
                    depth += 1;
                    draw_phase = true;
                    lifecycle_phase = !node.has_child();
                }
                WalkDirection::Sideward => {
                    draw_phase = false;
                    lifecycle_phase = !node.has_child();
                }
                WalkDirection::Upward => {
                    absolute_translation = absolute_translation - bounds.point().into();
                    depth -= 1;
                    draw_phase = false;
                    lifecycle_phase = true;
                }
            }

            latest_point = bounds.point();

            if !draw_phase && !lifecycle_phase {
                continue;
            }

            if draw_phase && !paint_state.flags.contains(PaintFlag::NeedsPaint) {
                if let Some(primitive) = &paint_state.draw_cache {
                    renderer.update_pipeline(pipeline, primitive, depth);
                }
                continue;
            }

            let mut context = PaintContext::new(
                widget_id,
                &mut self.event_manager,
                self.update_sender.clone()
            );

            if draw_phase {
                let WidgetPod {
                    widget,
                    children,
                    state,
                    ..
                } = (**node).clone();
                let absolute_bounds = bounds.translate(absolute_translation);

                let draw_result =
                    widget.draw(children, state, absolute_bounds, renderer, &mut context);

                if let Some(primitive) = &draw_result {
                    renderer.update_pipeline(pipeline, primitive, depth);
                }

                paint_state.absolute_translation = absolute_translation;
                paint_state.draw_cache = draw_result;
            }

            if lifecycle_phase {
                for WidgetPod {
                    widget,
                    state,
                    children,
                    ..
                } in mem::take(&mut paint_state.deleted_children)
                {
                    widget.lifecycle(
                        children,
                        state,
                        Lifecycle::DidUnmount(),
                        renderer,
                        &mut context,
                    );
                }

                let widget_pod = &**node;
                let old_widget_pod = paint_state.mounted_pod.replace(widget_pod.clone());
                let WidgetPod {
                    widget,
                    state,
                    children,
                    ..
                } = widget_pod.clone();

                if let Some(old_widget_pod) = old_widget_pod {
                    widget.lifecycle(
                        children,
                        state,
                        Lifecycle::DidUpdate(old_widget_pod.widget, old_widget_pod.children),
                        renderer,
                        &mut context,
                    );
                } else {
                    widget.clone().lifecycle(
                        children,
                        state,
                        Lifecycle::DidMount(),
                        renderer,
                        &mut context,
                    );
                }

                paint_state.flags -= PaintFlag::NeedsPaint;
            }
        }

        renderer.finish_pipeline(pipeline);
    }

    pub fn dispatch(&self, event: &GenericEvent) {
        for handler in self.event_manager.get(&event.type_id) {
            handler.dispatch(&event.payload)
        }
    }

    fn mark_parents_as_dirty(&mut self, target_id: WidgetId) {
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
            |f, widget_id, node| {
                let paint_state = &self.paint_states[widget_id];
                write!(f, "<{}", node.widget.name())?;
                write!(f, " id=\"{}\"", widget_id)?;
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
            absolute_translation: Vector::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            mounted_pod: None,
            deleted_children: Vec::new(),
            flags: [
                PaintFlag::Dirty,
                PaintFlag::NeedsLayout,
                PaintFlag::NeedsPaint,
            ]
            .into(),
            draw_cache: None,
        }
    }
}

impl Into<usize> for PaintFlag {
    fn into(self) -> usize {
        self as _
    }
}
