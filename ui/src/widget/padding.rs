use std::any::Any;

use geometrics::{Point, Size};
use layout::{BoxConstraints, LayoutResult};
use tree::NodeId;
use widget::{RenderingTree, Widget};

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

impl<WindowHandle, PaintContext> Widget<WindowHandle, PaintContext> for Padding {
    fn layout(
        &mut self,
        node_id: NodeId,
        response: Option<(NodeId, Size)>,
        box_constraints: &BoxConstraints,
        rendering_tree: &mut RenderingTree<WindowHandle, PaintContext>,
    ) -> LayoutResult {
        if let Some((child_id, size)) = response {
            rendering_tree[child_id].arrange(Point { x: self.left, y: self.top });
            LayoutResult::Size(Size {
                width: size.width + self.left + self.right,
                height: size.height + self.top + self.bottom
            })
        } else {
            let child_id = rendering_tree[node_id].first_child()
                    .filter(|&child| rendering_tree[child].next_sibling().is_none())
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}
