use std::any::Any;

use crate::generator::{Coroutine, Generator};
use crate::geometrics::{Point, Size};
use crate::layout::{BoxConstraints, LayoutRequest};
use crate::tree::NodeId;

use super::{Widget, WidgetMeta, WidgetTree};

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

    fn layout<'a>(
        &'a self,
        box_constraints: BoxConstraints,
        node_id: NodeId,
        tree: &'a WidgetTree<Handle>,
        _state: &'a Self::State,
    ) -> Generator<LayoutRequest, Size, Size> {
        Generator::new(move |co: Coroutine<LayoutRequest, Size>| async move {
            let child_id = tree[node_id]
                .first_child()
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
                },
            };
            let child_size = co
                .suspend(LayoutRequest::LayoutChild(child_id, child_box_constraints))
                .await;
            co.suspend(LayoutRequest::ArrangeChild(
                child_id,
                Point {
                    x: self.left,
                    y: self.top,
                },
            ))
            .await;
            Size {
                width: child_size.width + self.left + self.right,
                height: child_size.height + self.top + self.bottom,
            }
        })
    }
}

impl WidgetMeta for Padding {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
