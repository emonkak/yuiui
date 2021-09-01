use std::fmt;
use std::mem;

use crate::geometrics::{Point, Rectangle, Size, Vector};
use crate::graphics::{Primitive, Renderer};
use crate::support::bit_flags::BitFlags;
use crate::support::generator::GeneratorState;
use crate::support::slot_vec::SlotVec;
use crate::support::tree::WalkDirection;
use crate::widget::element::{create_element_tree, Element, ElementId, ElementTree, Patch};
use crate::widget::{Message, MessageEmitter, MessageSender, State};

use super::layout::{BoxConstraints, LayoutRequest};
use super::lifecycle::Lifecycle;

pub struct PaintTree<Renderer> {
    tree: ElementTree<Renderer>,
    root_id: ElementId,
    paint_states: SlotVec<PaintState<Renderer>>,
    message_sender: MessageSender,
}

struct PaintState<Renderer> {
    bounds: Rectangle,
    absolute_translation: Vector,
    box_constraints: BoxConstraints,
    mounted_element: Option<Element<Renderer>>,
    deleted_nodes: Vec<(ElementId, Element<Renderer>, PaintState<Renderer>)>,
    flags: BitFlags<PaintFlag>,
    draw_cache: Option<Primitive>,
}

#[derive(Debug)]
enum PaintFlag {
    Dirty = 0b1,
    NeedsLayout = 0b10,
    NeedsPaint = 0b100,
    NeedsLifecycle = 0b1000,
}

impl<Renderer: 'static> PaintTree<Renderer> {
    pub fn new(viewport_size: Size, message_sender: MessageSender) -> Self {
        let (tree, root_id, _) = create_element_tree();
        let mut paint_states = SlotVec::new();

        paint_states.insert_at(
            root_id,
            PaintState {
                box_constraints: BoxConstraints::tight(viewport_size),
                ..PaintState::new()
            },
        );

        Self {
            tree,
            root_id,
            paint_states,
            message_sender,
        }
    }

    pub fn mark_update_root(&mut self, element_id: ElementId) {
        self.paint_states[element_id].flags |= [
            PaintFlag::Dirty,
            PaintFlag::NeedsLayout,
            PaintFlag::NeedsPaint,
            PaintFlag::NeedsLifecycle,
        ];
        self.mark_parents_as_dirty(element_id);
    }

    pub fn apply_patch(&mut self, patch: Patch<Renderer>) {
        match patch {
            Patch::Append(parent_id, element) => {
                let paint_state = PaintState::new();
                let child_id = self.tree.append_child(parent_id, element);
                self.paint_states.insert_at(child_id, paint_state);
                self.mark_parents_as_dirty(child_id);
            }
            Patch::Insert(ref_id, element) => {
                let paint_state = PaintState::new();
                let child_id = self.tree.insert_before(ref_id, element);
                self.paint_states.insert_at(child_id, paint_state);
                self.mark_parents_as_dirty(child_id);
            }
            Patch::Update(target_id, new_element) => {
                *self.tree[target_id] = new_element;
                let paint_state = &mut self.paint_states[target_id];
                paint_state.flags |= [
                    PaintFlag::Dirty,
                    PaintFlag::NeedsLayout,
                    PaintFlag::NeedsPaint,
                    PaintFlag::NeedsLifecycle,
                ];
                self.mark_parents_as_dirty(target_id);
            }
            Patch::Move(target_id, ref_id) => {
                self.tree.move_position(target_id).insert_before(ref_id);
            }
            Patch::Remove(target_id) => {
                let (node, subtree) = self.tree.detach(target_id);
                let mut deleted_nodes = Vec::new();

                for (child_id, child) in subtree {
                    let paint_state = self.paint_states.remove(child_id);
                    deleted_nodes.push((child_id, child.into_inner(), paint_state));
                }

                let parent_id = node.parent().expect("root removed");
                let paint_state = self.paint_states.remove(target_id);
                deleted_nodes.push((target_id, node.into_inner(), paint_state));

                let parent_paint_state = &mut self.paint_states[parent_id];
                parent_paint_state.deleted_nodes.extend(deleted_nodes);
            }
        }
    }

    pub fn layout_root(&mut self, viewport_size: Size, renderer: &mut Renderer) {
        self.layout(self.root_id, BoxConstraints::tight(viewport_size), renderer);
    }

    pub fn layout_subtree(&mut self, target_id: ElementId, renderer: &mut Renderer) {
        let mut current_id = target_id;
        let mut box_constraints = self.paint_states[target_id].box_constraints;

        while let Some(parent_id) = self.layout(current_id, box_constraints, renderer) {
            current_id = parent_id;
            box_constraints = self.paint_states[parent_id].box_constraints;
        }
    }

    fn layout(
        &mut self,
        root_id: ElementId,
        root_box_constraints: BoxConstraints,
        renderer: &mut Renderer,
    ) -> Option<ElementId> {
        let initial_node = &self.tree[root_id];

        let mut layout_context = (root_id, root_box_constraints, {
            let Element { widget, .. } = &**initial_node;

            let child_ids = self
                .tree
                .children(root_id)
                .map(|(child_id, _)| child_id)
                .collect();

            widget.layout(
                State::from(widget.initial_state()).as_any_mut(),
                root_box_constraints,
                child_ids,
                renderer,
                &mut MessageEmitter::new(root_id, &self.message_sender),
            )
        });
        let mut layout_stack = Vec::new();
        let mut calculated_size = Size::ZERO;

        loop {
            let (_, _, ref mut layout) = layout_context;

            match layout.resume(calculated_size) {
                GeneratorState::Yielded(LayoutRequest::LayoutChild(
                    child_id,
                    child_box_constraints,
                )) => {
                    let paint_state = &self.paint_states[child_id];
                    if paint_state.flags.contains(PaintFlag::NeedsLayout)
                        || paint_state.box_constraints != child_box_constraints
                    {
                        let Element { widget, .. } = &*self.tree[child_id];

                        let child_ids = self
                            .tree
                            .children(child_id)
                            .map(|(child_id, _)| child_id)
                            .collect();

                        layout_stack.push(layout_context);
                        layout_context = (
                            child_id,
                            child_box_constraints,
                            widget.layout(
                                State::from(widget.initial_state()).as_any_mut(),
                                child_box_constraints,
                                child_ids,
                                renderer,
                                &mut MessageEmitter::new(child_id, &&self.message_sender),
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
                    let (element_id, box_constraints, _) = layout_context;

                    let mut paint_state = &mut self.paint_states[element_id];
                    paint_state.box_constraints = box_constraints;
                    paint_state.flags -= PaintFlag::NeedsLayout;

                    let size_changed = size != paint_state.bounds.size();
                    if size_changed {
                        paint_state.bounds.width = size.width;
                        paint_state.bounds.height = size.height;
                        paint_state.flags |= [PaintFlag::Dirty, PaintFlag::NeedsPaint];
                    }

                    if let Some(next_layout_context) = layout_stack.pop() {
                        layout_context = next_layout_context;
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

        while let Some((element_id, node, direction)) = tree_walker.next_match(|element_id, _| {
            self.paint_states[element_id]
                .flags
                .contains(PaintFlag::Dirty)
        }) {
            let paint_state = &mut self.paint_states[element_id];
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
                    draw_phase = !node.has_child();
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

            if draw_phase {
                let Element { widget, .. } = &**node;
                let absolute_bounds = bounds.translate(absolute_translation);

                let draw_result = widget.draw(
                    State::from(widget.initial_state()).as_any_mut(),
                    absolute_bounds,
                    renderer,
                    &mut MessageEmitter::new(element_id, &self.message_sender),
                );

                if let Some(primitive) = &draw_result {
                    renderer.update_pipeline(pipeline, primitive, depth);
                }

                paint_state.absolute_translation = absolute_translation;
                paint_state.draw_cache = draw_result;
            }

            if lifecycle_phase {
                if paint_state.flags.contains(PaintFlag::NeedsLifecycle) {
                    let element = &**node;
                    let Element { widget, .. } = element;

                    if let Some(old_element) = paint_state.mounted_element.take() {
                        widget.lifecycle(
                            State::from(widget.initial_state()).as_any_mut(),
                            Lifecycle::DidUpdate(old_element.widget.as_any()),
                            renderer,
                            &mut MessageEmitter::new(element_id, &self.message_sender),
                        );
                    } else {
                        widget.lifecycle(
                            State::from(widget.initial_state()).as_any_mut(),
                            Lifecycle::DidMount(),
                            renderer,
                            &mut MessageEmitter::new(element_id, &&self.message_sender),
                        );
                    };

                    paint_state.mounted_element = Some(element.clone());
                }

                for (element_id, Element { widget, .. }, _paint_state) in
                    mem::take(&mut paint_state.deleted_nodes)
                {
                    widget.lifecycle(
                        State::from(widget.initial_state()).as_any_mut(),
                        Lifecycle::DidUnmount(),
                        renderer,
                        &mut MessageEmitter::new(element_id, &self.message_sender),
                    );
                }

                paint_state.flags -= [PaintFlag::NeedsPaint, PaintFlag::NeedsLifecycle];
            }
        }

        renderer.finish_pipeline(pipeline);
    }

    pub fn send_message(&self, message: Message) {
        self.message_sender.send(message).unwrap();
    }

    fn mark_parents_as_dirty(&mut self, target_id: ElementId) {
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
            |f, element_id, node| {
                let paint_state = &self.paint_states[element_id];
                write!(f, "<{}", node.widget.short_type_name())?;
                write!(f, " id=\"{}\"", element_id)?;
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
            |f, _, node| write!(f, "</{}>", node.widget.short_type_name()),
        )
    }
}

impl<Renderer> PaintState<Renderer> {
    fn new() -> Self {
        Self {
            bounds: Rectangle::ZERO,
            absolute_translation: Vector::ZERO,
            box_constraints: BoxConstraints::LOOSE,
            mounted_element: None,
            deleted_nodes: Vec::new(),
            flags: [
                PaintFlag::Dirty,
                PaintFlag::NeedsLayout,
                PaintFlag::NeedsPaint,
                PaintFlag::NeedsLifecycle,
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
