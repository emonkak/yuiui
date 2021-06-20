use geometrics::{BoxConstraints, Point, Rectangle, Size};
use graph::NodeId;
use ui::{LayoutResult, LayoutContext, UIState};
use widget::Widget;

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

    pub fn ui<WidgetState: Clone, PaintContext>(self, child: NodeId, context: &mut UIState<WidgetState, PaintContext>) -> NodeId {
        context.add(self, &[child])
    }
}

impl<WindowHandle: Clone, PaintContext> Widget<WindowHandle, PaintContext> for Padding {
    fn layout(
        &mut self,
        box_constraints: &BoxConstraints,
        children: &[NodeId],
        size: Option<Size>,
        layout_context: &mut LayoutContext
    ) -> LayoutResult {
        if let Some(size) = size {
            layout_context.position_child(children[0], Point { x: self.left, y: self.top });
            LayoutResult::Size(Size {
                width: size.width + self.left + self.right,
                height: size.height + self.top + self.bottom
            })
        } else {
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
            LayoutResult::RequestChild(children[0], child_box_constraints)
        }
    }

    fn connect(&mut self, parent_handle: &WindowHandle, _rectangle: &Rectangle, _paint_context: &mut PaintContext) -> WindowHandle {
        parent_handle.clone()
    }
}
