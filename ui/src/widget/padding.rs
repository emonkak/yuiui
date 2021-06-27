use std::any::Any;

use geometrics::{Point, Size};
use layout::{BoxConstraints, LayoutResult, LayoutContext};
use tree::NodeId;
use widget::widget::{FiberTree, Widget, WidgetBase};

/// A padding widget. Is expected to have exactly one child.
pub struct Padding {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl Padding {
    /// Create widget with uniform padding.
    pub fn uniform(padding: f32) -> Padding {
        Padding {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
        }
    }
}

impl<Window> Widget<Window> for Padding {
    fn layout(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        fiber_tree: &FiberTree<Window>,
        layout_context: &mut LayoutContext
    ) -> LayoutResult {
        if let Some((child_id, size)) = response {
            layout_context.arrange(child_id, Point { x: self.left, y: self.top });
            LayoutResult::Size(Size {
                width: size.width + self.left + self.right,
                height: size.height + self.top + self.bottom
            })
        } else {
            let child_id = fiber_tree[node_id].first_child()
                    .filter(|&child| fiber_tree[child].next_sibling().is_none())
                    .expect("Padding expected to receive a single element child.");
            let child_box_constraints = BoxConstraints {
                min: Size {
                    width: box_constraints.min.width - (self.left + self.right),
                    height: box_constraints.min.height - (self.top + self.bottom),
                },
                max: Size {
                    width: box_constraints.max.width - (self.left + self.right),
                    height: box_constraints.max.height - (self.top + self.bottom),
                }
            };
            LayoutResult::RequestChild(child_id, child_box_constraints)
        }
    }
}

impl WidgetBase for Padding {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
