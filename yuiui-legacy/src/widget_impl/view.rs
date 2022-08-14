use yuiui_support::slot_tree::NodeId;

use crate::geometrics::{BoxConstraints, Point, RectOutsets, Size};
use crate::style::LayoutStyle;
use crate::widget::{ElementInstance, LayoutContext, Widget};

#[derive(Debug, PartialEq)]
pub struct View {
    direction: Axis,
    layout_style: LayoutStyle,
}

impl View {
    pub fn row(layout_style: LayoutStyle) -> Self {
        Self {
            direction: Axis::Vertical,
            layout_style,
        }
    }

    pub fn column(layout_style: LayoutStyle) -> Self {
        Self {
            direction: Axis::Horizontal,
            layout_style,
        }
    }
}

impl<State, Message> Widget<State, Message> for View {
    type LocalState = ();

    fn initial_state(&self) -> Self::LocalState {
        ()
    }

    fn should_update(&self, new_widget: &Self) -> bool {
        self != new_widget
    }

    fn layout(
        &self,
        box_constraints: BoxConstraints,
        children: &[NodeId],
        context: &mut LayoutContext<State, Message>,
        _state: &mut Self::LocalState,
    ) -> Size {
        let box_constraints = box_constraints.deflate(self.layout_style.padding);

        let mut flex_sum = 0.0;
        let mut total_non_flex = 0.0;
        let mut minor = self.direction.minor(box_constraints.min);

        let flex_params = children
            .iter()
            .map(|child| context.get_layout_style(*child).flex)
            .collect::<Vec<_>>();

        for (i, child) in children.iter().enumerate() {
            let flex_param = flex_params[i];
            if flex_param > 0.0 {
                flex_sum += flex_param;
            } else {
                let child_size = context.layout_child(*child, box_constraints);
                minor = self.direction.minor(child_size).max(minor);
                total_non_flex += self.direction.major(child_size);
            }
        }

        for (i, child) in children.iter().enumerate() {
            let flex_param = flex_params[i];
            if flex_param > 0.0 {
                let total_major = self.direction.major(box_constraints.max);
                let remaining = (total_major - total_non_flex).max(0.0);
                let major = remaining * flex_param / flex_sum;

                let child_box_constraints =
                    self.direction
                        .adjust_box_constraints(box_constraints, major, major);
                let child_size = context.layout_child(*child, child_box_constraints);

                minor = self.direction.minor(child_size).max(minor);
            }
        }

        let total_major = self.direction.major(box_constraints.max);
        let mut major = 0.0;

        for i in 0..children.len() {
            let child = children[i];
            let position = if i == 0 {
                self.direction
                    .pack_point_first(major, 0.0, self.layout_style.padding)
            } else {
                self.direction
                    .pack_point(major, 0.0, self.layout_style.padding)
            };

            context.set_position(child, position);

            let size = context.get_size(child);
            major += self.direction.major(size);
        }

        self.direction
            .pack_size(total_major, minor, self.layout_style.padding)
    }

    fn layout_style(&self) -> LayoutStyle {
        self.layout_style.clone()
    }
}

impl<State: 'static, Message: 'static> From<View> for ElementInstance<State, Message> {
    fn from(widget: View) -> Self {
        widget.into_rc().into()
    }
}

#[derive(Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn major(&self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    fn minor(&self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32, padding: RectOutsets) -> Point {
        match self {
            Self::Horizontal => Point {
                x: major,
                y: minor + padding.top,
            },
            Self::Vertical => Point {
                x: minor + padding.left,
                y: major,
            },
        }
    }

    fn pack_point_first(&self, major: f32, minor: f32, padding: RectOutsets) -> Point {
        match self {
            Self::Horizontal => Point {
                x: major + padding.left,
                y: minor + padding.top,
            },
            Self::Vertical => Point {
                x: minor + padding.left,
                y: major + padding.top,
            },
        }
    }

    fn pack_size(&self, major: f32, minor: f32, padding: RectOutsets) -> Size {
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
        .inflate(padding)
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
