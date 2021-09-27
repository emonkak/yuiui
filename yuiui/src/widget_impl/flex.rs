use yuiui_support::slot_tree::NodeId;

use crate::geometrics::{BoxConstraints, Point, Size};
use crate::widget::{ElementNode, LayoutContext, Widget};

#[derive(Debug, PartialEq)]
pub struct Flex {
    direction: Axis,
}

impl Flex {
    pub fn row() -> Self {
        Self {
            direction: Axis::Vertical,
        }
    }

    pub fn column() -> Self {
        Self {
            direction: Axis::Horizontal,
        }
    }
}

impl<Message> Widget<Message> for Flex {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Self::State::default()
    }

    fn should_update(&self, new_widget: &Self, _state: &Self::State) -> bool {
        self != new_widget
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<Message>,
        _state: &mut Self::State,
    ) -> Size {
        let mut flex_sum = 0.0;
        let mut total_non_flex = 0.0;
        let mut minor = self.direction.minor(box_constraints.min);

        let flex_params = children
            .iter()
            .map(|child| context.get_attributes(*child).get::<FlexParam>())
            .collect::<Vec<_>>();

        for (i, child) in children.iter().enumerate() {
            match flex_params[i] {
                Some(FlexParam(flex)) => {
                    flex_sum += flex;
                }
                None => {
                    let child_size = context.layout_child(*child, box_constraints);
                    minor = self.direction.minor(child_size).max(minor);
                    total_non_flex += self.direction.major(child_size);
                }
            }
        }

        for (i, child) in children.iter().enumerate() {
            match flex_params[i] {
                Some(FlexParam(flex)) => {
                    let total_major = self.direction.major(box_constraints.max);
                    let remaining = (total_major - total_non_flex).max(0.0);
                    let major = remaining * flex / flex_sum;

                    let child_box_constraints =
                        self.direction
                            .adjust_box_constraints(box_constraints, major, major);
                    let child_size = context.layout_child(*child, child_box_constraints);

                    minor = self.direction.minor(child_size).max(minor);
                }
                None => {}
            }
        }

        let total_major = self.direction.major(box_constraints.max);
        let mut major = 0.0;

        for child in children {
            let position = self.direction.pack_point(major, 0.0);
            context.set_position(*child, position);

            let size = context.get_size(*child);
            major += self.direction.major(size);
        }

        self.direction.pack_size(total_major, minor)
    }
}

impl<Message: 'static> From<Flex> for ElementNode<Message> {
    fn from(widget: Flex) -> Self {
        widget.into_rc().into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexParam(pub f32);

#[derive(Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn major<T>(&self, size: Size<T>) -> T {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    fn minor<T>(&self, size: Size<T>) -> T {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32) -> Point {
        match self {
            Self::Horizontal => Point { x: major, y: minor },
            Self::Vertical => Point { x: minor, y: major },
        }
    }

    fn pack_size(&self, major: f32, minor: f32) -> Size {
        match self {
            Self::Horizontal => Size {
                width: major,
                height: minor,
            },
            Self::Vertical => Size {
                width: minor,
                height: major,
            },
        }
    }

    fn adjust_box_constraints(
        &self,
        box_constraints: BoxConstraints,
        min_major: f32,
        max_major: f32,
    ) -> BoxConstraints {
        match self {
            Self::Horizontal => BoxConstraints {
                min: Size {
                    width: min_major,
                    height: box_constraints.min.height,
                },
                max: Size {
                    width: max_major,
                    height: box_constraints.max.height,
                },
            },
            Self::Vertical => BoxConstraints {
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
