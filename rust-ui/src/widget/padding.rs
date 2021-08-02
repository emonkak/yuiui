use rust_ui_derive::WidgetMeta;

use crate::generator::{Coroutine, Generator};
use crate::geometrics::{Point, Size};
use crate::paint::layout::{BoxConstraints, LayoutRequest};
use crate::tree::NodeId;

use super::{Widget, WidgetMeta, WidgetTree};

#[derive(WidgetMeta)]
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

impl<Painter> Widget<Painter> for Padding {
    type State = ();

    fn layout<'a>(
        &'a self,
        node_id: NodeId,
        box_constraints: BoxConstraints,
        tree: &'a WidgetTree<Painter>,
        _state: &mut Self::State,
        _painter: &mut Painter,
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
