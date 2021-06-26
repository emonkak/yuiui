use std::any::Any;

use geometrics::{Point, Size};
use layout::{BoxConstraints, LayoutContext, LayoutNode, LayoutResult};
use tree::NodeId;
use widget::widget::{Widget, WidgetMaker};

pub struct Row;

pub struct Column;

pub struct Flex {
    direction: Axis,

    // layout continuation state
    phase: Phase,
    minor: f32,

    // the total measure of non-flex children
    total_non_flex: f32,

    // the sum of flex parameters of all children
    flex_sum: f32,
}

pub struct FlexItem {
    params: Params,
}

pub enum Axis {
    Horizontal,
    Vertical,
}

// Layout happens in two phases. First, the non-flex children
// are laid out. Then, the remaining space is divided across
// the flex children.
#[derive(Clone, Copy, PartialEq)]
enum Phase {
    NonFlex,
    Flex,
}

#[derive(Copy, Clone, Default)]
struct Params {
    flex: f32,
}

impl Params {
    // Determine the phase in which this child should be measured.
    fn get_flex_phase(&self) -> Phase {
        if self.flex == 0.0 {
            Phase::NonFlex
        } else {
            Phase::Flex
        }
    }
}

impl Axis {
    fn major(&self, coords: Size) -> f32 {
        match *self {
            Axis::Horizontal => coords.width,
            Axis::Vertical => coords.height,
        }
    }

    fn minor(&self, coords: Size) -> f32 {
        match *self {
            Axis::Horizontal => coords.height,
            Axis::Vertical => coords.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32) -> Point {
        match *self {
            Axis::Horizontal => Point { x: major, y: minor },
            Axis::Vertical => Point { x: minor, y: major },
        }
    }

    fn pack_size(&self, major: f32, minor: f32) -> Size {
        match *self {
            Axis::Horizontal => Size { width: major, height: minor },
            Axis::Vertical => Size { width: minor, height: major },
        }
    }
}

impl Row {
    pub fn new() -> Flex {
        Flex {
            direction: Axis::Horizontal,
            phase: Phase::NonFlex,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }
}

impl Column {
    pub fn new() -> Flex {
        Flex {
            direction: Axis::Vertical,
            phase: Phase::NonFlex,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }
}

impl Flex {
    fn get_params<Window>(&self, node: &LayoutNode<Window>) -> Params {
        node.as_widget::<FlexItem>()
            .map(|flex_item| flex_item.params)
            .unwrap_or_default()
    }

    /// Return the index (within `children`) of the next child that belongs in
    /// the specified phase.
    fn get_next_child<'a, Window: 'a>(
        &self,
        children: impl Iterator<Item = (NodeId, &'a LayoutNode<Window>)>,
        phase: Phase,
    ) -> Option<NodeId> {
        for (child_id, child) in children {
            if self.get_params(&*child).get_flex_phase() == phase {
                return Some(child_id);
            }
        }
        None
    }

    fn finish_layout<Window>(
        &self,
        node_id: NodeId,
        box_constraints: &BoxConstraints,
        rendering_tree: LayoutContext<'_, Window>
    ) -> LayoutResult {
        let mut major = 0.0;
        for (_, child) in rendering_tree.children_mut(node_id) {
            // top-align, could do center etc. based on child height
            child.arrange(self.direction.pack_point(major, 0.0));
            major += self.direction.major(child.rectangle.size);
        }
        let total_major = self.direction.major(box_constraints.max);
        LayoutResult::Size(self.direction.pack_size(total_major, self.minor))
    }
}

impl<Window> Widget<Window> for Flex {
    fn layout(
        &mut self,
        node_id: NodeId,
        response: Option<(NodeId, Size)>,
        box_constraints: &BoxConstraints,
        layout_context: LayoutContext<'_, Window>,
    ) -> LayoutResult {
        let next_child_id = if let Some((child_id, size)) = response {
            let minor = self.direction.minor(size);
            self.minor = self.minor.max(minor);

            if self.phase == Phase::NonFlex {
                self.total_non_flex += self.direction.major(size);
            }

            // Advance to the next child; finish non-flex phase if at end.
            if let Some(id) = self.get_next_child(layout_context.next_siblings(child_id), self.phase) {
                id
            } else if self.phase == Phase::NonFlex {
                if let Some(id) = self.get_next_child(layout_context.next_siblings(child_id), Phase::Flex) {
                    self.phase = Phase::Flex;
                    id
                } else {
                    return self.finish_layout(node_id, box_constraints, layout_context);
                }
            } else {
                return self.finish_layout(node_id, box_constraints, layout_context)
            }
        } else {
            // Start layout process, no children measured yet.
            if let Some(first_child_id) = layout_context[node_id].first_child() {
                self.total_non_flex = 0.0;
                self.flex_sum = layout_context
                    .children(node_id)
                    .map(|(_, node)| self.get_params(&*node).flex)
                    .sum();
                self.minor = self.direction.minor(box_constraints.min);

                if let Some(id) = self.get_next_child(layout_context.children(node_id), Phase::NonFlex) {
                    self.phase = Phase::NonFlex;
                    id
                } else {
                    // All children are flex, skip non-flex pass.
                    self.phase = Phase::Flex;
                    first_child_id
                }
            } else {
                return LayoutResult::Size(box_constraints.min);
            }
        };

        let (min_major, max_major) = if self.phase == Phase::NonFlex {
            (0.0, ::std::f32::INFINITY)
        } else {
            let total_major = self.direction.major(box_constraints.max);
            // TODO: should probably max with 0.0 to avoid negative sizes
            let remaining = total_major - self.total_non_flex;
            let major = remaining * self.get_params(&layout_context[next_child_id]).flex / self.flex_sum;
            (major, major)
        };

        let child_box_constraints = match self.direction {
            Axis::Horizontal => BoxConstraints {
                min: Size {
                    width: min_major,
                    height: box_constraints.min.height,
                },
                max: Size {
                    width: max_major,
                    height: box_constraints.max.height,
                }
            },
            Axis::Vertical => BoxConstraints {
                min: Size {
                    width: box_constraints.min.width,
                    height: min_major,
                },
                max: Size {
                    width: box_constraints.max.width,
                    height: max_major,
                }
            },
        };

        LayoutResult::RequestChild(next_child_id, child_box_constraints)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetMaker for Flex {
}

impl FlexItem {
    pub fn new(flex: f32) -> FlexItem {
        FlexItem {
            params: Params {
                flex
            }
        }
    }
}

impl<Window> Widget<Window> for FlexItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl WidgetMaker for FlexItem {
}
