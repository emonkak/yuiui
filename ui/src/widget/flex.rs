use std::any::Any;

use geometrics::{Point, Size};
use layout::{BoxConstraints, LayoutResult};
use tree::NodeId;
use widget::widget::{Element, Layout, LayoutContext, Widget, WidgetMeta, WidgetNode, WidgetTree};

#[derive(PartialEq)]
pub struct Flex {
    direction: Axis,
}

#[derive(PartialEq)]
pub struct FlexItem {
    params: Params,
}

pub struct FlexLayout {
    direction: Axis,

    // layout continuation state
    phase: Phase,
    minor: f32,

    // the total measure of non-flex children
    total_non_flex: f32,

    // the sum of flex parameters of all children
    flex_sum: f32,
}

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, Default, PartialEq)]
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
    fn major(&self, coords: &Size) -> f32 {
        match self {
            Axis::Horizontal => coords.width,
            Axis::Vertical => coords.height,
        }
    }

    fn minor(&self, coords: &Size) -> f32 {
        match self {
            Axis::Horizontal => coords.height,
            Axis::Vertical => coords.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32) -> Point {
        match self {
            Axis::Horizontal => Point { x: major, y: minor },
            Axis::Vertical => Point { x: minor, y: major },
        }
    }

    fn pack_size(&self, major: f32, minor: f32) -> Size {
        match self {
            Axis::Horizontal => Size { width: major, height: minor },
            Axis::Vertical => Size { width: minor, height: major },
        }
    }
}

impl Flex {
    pub fn row() -> Self {
        Self {
            direction: Axis::Horizontal,
        }
    }

    pub fn column() -> Self {
        Self {
            direction: Axis::Vertical,
        }
    }
}

impl FlexLayout {
    fn new(direction: Axis) -> Self {
        Self {
            direction,
            phase: Phase::NonFlex,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }

    fn get_params<Handle>(&self, widget: &WidgetNode<Handle>) -> Params {
        widget.as_any()
            .downcast_ref::<FlexItem>()
            .map(|flex_item| flex_item.params)
            .unwrap_or_default()
    }

    fn get_next_child<'a, Handle: 'a>(
        &self,
        children: impl Iterator<Item = (NodeId, &'a WidgetNode<Handle>)>,
        phase: Phase,
    ) -> Option<NodeId> {
        for (child_id, child) in children {
            if self.get_params(child).get_flex_phase() == phase {
                return Some(child_id);
            }
        }
        None
    }

    fn finish_layout<Handle>(
        &self,
        node_id: NodeId,
        box_constraints: &BoxConstraints,
        tree: &WidgetTree<Handle>,
        layout_context: &mut LayoutContext<'_, Handle>
    ) -> LayoutResult {
        let mut major = 0.0;
        for (child_id, _) in tree.children(node_id) {
            // top-align, could do center etc. based on child height
            layout_context.arrange(child_id, self.direction.pack_point(major, 0.0));
            major += self.direction.major(layout_context.get_size(child_id));
        }
        let total_major = self.direction.major(&box_constraints.max);
        let minor = self.minor;
        LayoutResult::Size(self.direction.pack_size(total_major, minor))
    }
}

impl<Handle> Widget<Handle> for Flex {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn should_update(&self, next_widget: &Self, _next_children: &[Element<Handle>]) -> bool {
        self == next_widget
    }

    fn layout(&self) -> Box<dyn Layout<Handle>> {
        Box::new(FlexLayout::new(self.direction))
    }
}

impl WidgetMeta for Flex {
    fn as_any(&self) -> &dyn Any {
        self
    }
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

impl<Handle> Widget<Handle> for FlexItem {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn should_update(&self, next_widget: &Self, _next_children: &[Element<Handle>]) -> bool {
        self == next_widget
    }
}

impl WidgetMeta for FlexItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Handle> Layout<Handle> for FlexLayout {
    fn measure(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        layout_context: &mut LayoutContext<'_, Handle>
    ) -> LayoutResult {
        let next_child_id = if let Some((child_id, size)) = response {
            self.minor = self.direction.minor(&size).max(self.minor);

            if self.phase == Phase::NonFlex {
                self.total_non_flex += self.direction.major(&size);

                // Advance to the next child; finish non-flex phase if at end.
                if let Some(child_id) = self.get_next_child(tree.next_siblings(child_id), Phase::NonFlex) {
                    child_id
                } else if let Some(child_id) = self.get_next_child(tree.next_siblings(child_id), Phase::Flex) {
                    self.phase = Phase::Flex;
                    child_id
                } else {
                    return self.finish_layout(node_id, &box_constraints, tree, layout_context);
                }
            } else {
                if let Some(child_id) = self.get_next_child(tree.next_siblings(child_id), Phase::Flex) {
                    child_id
                } else {
                    return self.finish_layout(node_id, &box_constraints, tree, layout_context);
                }
            }
        } else {
            // Start layout process, no children measured yet.
            if let Some(first_child_id) = tree[node_id].first_child() {
                self.total_non_flex = 0.0;
                self.flex_sum = tree
                    .children(node_id)
                    .map(|(_, node)| self.get_params(node).flex)
                    .sum();
                self.minor = self.direction.minor(&box_constraints.min);

                if let Some(child_id) = self.get_next_child(tree.children(node_id), Phase::NonFlex) {
                    self.phase = Phase::NonFlex;
                    child_id
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
            let total_major = self.direction.major(&box_constraints.max);
            // TODO: should probably max with 0.0 to avoid negative sizes
            let remaining = total_major - self.total_non_flex;
            let major = remaining * self.get_params(&tree[next_child_id]).flex / self.flex_sum;
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
}
