use std::any::Any;

use geometrics::{Point, Size};
use layout::{BoxConstraints, Layout, LayoutResult, Layouter};
use tree::NodeId;

use super::{Widget, WidgetInstance, WidgetMeta, WidgetTree};

#[derive(Clone)]
pub struct Padding {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl Padding {
    pub fn uniform(padding: f32) -> Padding {
        Padding {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
        }
    }
}

impl<Handle> Widget<Handle> for Padding {
    type State = ();

    fn initial_state(&self) -> Self::State {
        Default::default()
    }

    fn layout(&self) -> Box<dyn Layout<WidgetInstance<Handle>>> {
        Box::new(self.clone())
    }
}

impl WidgetMeta for Padding {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Handle> Layout<WidgetInstance<Handle>> for Padding {
    fn measure(
        &mut self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        response: Option<(NodeId, Size)>,
        tree: &WidgetTree<Handle>,
        layouter: &mut dyn Layouter
    ) -> LayoutResult {
        if let Some((child_id, size)) = response {
            layouter.arrange(child_id, Point { x: self.left, y: self.top });
            LayoutResult::Size(Size {
                width: size.width + self.left + self.right,
                height: size.height + self.top + self.bottom
            })
        } else {
            let child_id = tree[node_id].first_child()
                    .filter(|&child| tree[child].next_sibling().is_none())
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
