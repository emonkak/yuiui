use std::any::Any;

use crate::geometrics::{Point, Size};
use crate::layout::{BoxConstraints, LayoutContext, LayoutResult};
use crate::tree::NodeId;

use super::{BoxedWidget, Widget, WidgetMeta, WidgetNode, WidgetTree};

pub struct Flex {
    direction: Axis,
}

pub struct FlexItem {
    params: Params,
}

pub struct FlexState {
    phase: Phase,
    minor: f32,
    total_non_flex: f32,
    flex_sum: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    NonFlex,
    Flex,
}

#[derive(Clone, Copy, Default)]
struct Params {
    flex: f32,
}

impl Params {
    fn flex_phase(&self) -> Phase {
        if self.flex == 0.0 {
            Phase::NonFlex
        } else {
            Phase::Flex
        }
    }
}

impl Axis {
    fn major(&self, size: &Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn minor(&self, size: &Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
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
            Axis::Horizontal => Size {
                width: major,
                height: minor,
            },
            Axis::Vertical => Size {
                width: minor,
                height: major,
            },
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

impl FlexState {
    fn get_params<Handle>(&self, widget: &BoxedWidget<Handle>) -> Params {
        widget
            .as_any()
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
            if self.get_params(child).flex_phase() == phase {
                return Some(child_id);
            }
        }
        None
    }

    fn finish_layout<Handle>(
        &mut self,
        direction: &Axis,
        node_id: NodeId,
        box_constraints: &BoxConstraints,
        tree: &WidgetTree<Handle>,
        context: &mut dyn LayoutContext,
    ) -> LayoutResult {
        let mut major = 0.0;
        for (child_id, _) in tree.children(node_id) {
            context.arrange(child_id, direction.pack_point(major, 0.0));
            major += direction.major(context.get_size(child_id));
        }
        let total_major = direction.major(&box_constraints.max);
        let minor = self.minor;
        let size = direction.pack_size(total_major, minor);
        *self = Default::default();
        LayoutResult::Size(size)
    }
}

impl Default for FlexState {
    fn default() -> Self {
        Self {
            phase: Phase::NonFlex,
            minor: 0.0,
            total_non_flex: 0.0,
            flex_sum: 0.0,
        }
    }
}

impl<Handle> Widget<Handle> for Flex {
    type State = FlexState;

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn layout(
        &self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        state: &mut Self::State,
        context: &mut dyn LayoutContext,
    ) -> LayoutResult {
        let next_child_id = if let Some((child_id, size)) = response {
            state.minor = self.direction.minor(&size).max(state.minor);

            if state.phase == Phase::NonFlex {
                state.total_non_flex += self.direction.major(&size);

                if let Some(child_id) =
                    state.get_next_child(tree.next_siblings(child_id), Phase::NonFlex)
                {
                    child_id
                } else if let Some(child_id) =
                    state.get_next_child(tree.next_siblings(child_id), Phase::Flex)
                {
                    state.phase = Phase::Flex;
                    child_id
                } else {
                    return state.finish_layout(
                        &self.direction,
                        node_id,
                        &box_constraints,
                        tree,
                        context,
                    );
                }
            } else {
                if let Some(child_id) =
                    state.get_next_child(tree.next_siblings(child_id), Phase::Flex)
                {
                    child_id
                } else {
                    return state.finish_layout(
                        &self.direction,
                        node_id,
                        &box_constraints,
                        tree,
                        context,
                    );
                }
            }
        } else {
            if let Some(first_child_id) = tree[node_id].first_child() {
                state.total_non_flex = 0.0;
                state.flex_sum = tree
                    .children(node_id)
                    .map(|(_, node)| state.get_params(node).flex)
                    .sum();
                state.minor = self.direction.minor(&box_constraints.min);

                if let Some(child_id) = state.get_next_child(tree.children(node_id), Phase::NonFlex)
                {
                    state.phase = Phase::NonFlex;
                    child_id
                } else {
                    state.phase = Phase::Flex;
                    first_child_id
                }
            } else {
                return LayoutResult::Size(box_constraints.min);
            }
        };

        let (min_major, max_major) = if state.phase == Phase::NonFlex {
            (0.0, ::std::f32::INFINITY)
        } else {
            let total_major = self.direction.major(&box_constraints.max);
            let remaining = total_major - state.total_non_flex;
            let major = remaining * state.get_params(&tree[next_child_id]).flex / state.flex_sum;
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
                },
            },
            Axis::Vertical => BoxConstraints {
                min: Size {
                    width: box_constraints.min.width,
                    height: min_major,
                },
                max: Size {
                    width: box_constraints.max.width,
                    height: max_major,
                },
            },
        };

        LayoutResult::RequestChild(next_child_id, child_box_constraints)
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
            params: Params { flex },
        }
    }
}

impl<Handle> Widget<Handle> for FlexItem {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }
}

impl WidgetMeta for FlexItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
