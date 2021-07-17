use std::any::Any;

use crate::generator::{Coroutine, Generator};
use crate::geometrics::{Point, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::tree::NodeId;

use super::{BoxedWidget, Widget, WidgetMeta, WidgetTree};

pub struct Flex {
    direction: Axis,
}

pub struct FlexItem {
    params: Params,
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

impl<Handle> Widget<Handle> for Flex {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn layout<'a>(
        &'a self,
        box_constraints: BoxConstraints,
        node_id: NodeId,
        tree: &'a WidgetTree<Handle>,
        _state: &'a Self::State,
    ) -> Generator<LayoutRequest, Size, Size> {
        Generator::new(move |co: Coroutine<LayoutRequest, Size>| async move {
            let mut flex_sum = 0.0;
            let mut total_non_flex = 0.0;
            let mut minor = self.direction.minor(&box_constraints.min);

            let children = tree
                .children(node_id)
                .map(|(child_id, child)| (child_id, get_params(child)))
                .collect::<Vec<_>>();

            for (child_id, params) in children.iter() {
                if params.flex_phase() == Phase::NonFlex {
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, box_constraints))
                        .await;

                    minor = self.direction.minor(&child_size).max(minor);
                    total_non_flex += self.direction.major(&child_size);
                }
                flex_sum += params.flex;
            }

            for (child_id, params) in children.iter() {
                if params.flex_phase() == Phase::Flex {
                    let total_major = self.direction.major(&box_constraints.max);
                    let remaining = (total_major - total_non_flex).max(0.0);
                    let major = remaining * params.flex / flex_sum;

                    let child_box_constraints =
                        self.direction
                            .apply_to_box_constraints(&box_constraints, major, major);
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, child_box_constraints))
                        .await;

                    minor = self.direction.minor(&child_size).max(minor);
                }
            }

            let total_major = self.direction.major(&box_constraints.max);
            let mut major = 0.0;

            for (child_id, _) in children.iter() {
                let point = self.direction.pack_point(major, 0.0);
                let child_size = co
                    .suspend(LayoutRequest::ArrangeChild(*child_id, point))
                    .await;
                major += self.direction.major(&child_size);
            }

            self.direction.pack_size(total_major, minor)
        })
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

    fn apply_to_box_constraints(
        &self,
        box_constraints: &BoxConstraints,
        min_major: f32,
        max_major: f32,
    ) -> BoxConstraints {
        match self {
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
        }
    }
}

fn get_params<Handle>(widget: &BoxedWidget<Handle>) -> Params {
    widget
        .as_any()
        .downcast_ref::<FlexItem>()
        .map(|flex_item| flex_item.params)
        .unwrap_or_default()
}
