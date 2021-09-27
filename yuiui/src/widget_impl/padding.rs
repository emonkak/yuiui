use yuiui_support::slot_tree::NodeId;

use crate::geometrics::{BoxConstraints, Point, Size, Thickness};
use crate::widget::{ElementNode, LayoutContext, Widget};

#[derive(Debug, PartialEq)]
pub struct Padding {
    pub thickness: Thickness,
}

impl<Message> Widget<Message> for Padding {
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
        assert_eq!(children.len(), 1, "Must to receive a single element child.");
        let child = children[0];
        let child_box_constraints = BoxConstraints {
            min: Size {
                width: box_constraints.min.width - (self.thickness.left + self.thickness.right),
                height: box_constraints.min.height - (self.thickness.top + self.thickness.bottom),
            },
            max: Size {
                width: box_constraints.max.width - (self.thickness.left + self.thickness.right),
                height: box_constraints.max.height - (self.thickness.top + self.thickness.bottom),
            },
        };
        let child_size = context.layout_child(child, child_box_constraints);
        context.set_position(
            child,
            Point {
                x: self.thickness.left,
                y: self.thickness.top,
            },
        );
        Size {
            width: child_size.width + self.thickness.left + self.thickness.right,
            height: child_size.height + self.thickness.top + self.thickness.bottom,
        }
    }
}

impl<Message: 'static> From<Padding> for ElementNode<Message> {
    fn from(widget: Padding) -> Self {
        widget.into_rc().into()
    }
}
