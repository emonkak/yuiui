use std::any::Any;

use super::state::StateContainer;
use crate::geometrics::{Point, Size};
use crate::paint::{BoxConstraints, LayoutRequest};
use crate::support::generator::{Coroutine, Generator};

use super::element::{Children, Element, ElementId, IntoElement};
use super::message::MessageEmitter;
use super::widget::{Widget, WidgetSeal};

pub struct Padding<Renderer> {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    child: Element<Renderer>,
}

impl<Renderer> Padding<Renderer> {
    pub fn uniform(padding: f32, child: impl IntoElement<Renderer>) -> Self {
        Self {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
            child: child.into_element(),
        }
    }
}

impl<Renderer: 'static> Widget<Renderer> for Padding<Renderer> {
    type State = ();
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_pure_state(())
    }

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        vec![self.child.clone()]
    }

    fn layout<'a>(
        &'a self,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        child_ids: Vec<ElementId>,
        _renderer: &mut Renderer,
        _context: &mut MessageEmitter,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        assert_eq!(child_ids.len(), 1);
        Generator::new(move |co: Coroutine<LayoutRequest, Size>| async move {
            let child_id = child_ids[0];
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

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Renderer> WidgetSeal for Padding<Renderer> {}
