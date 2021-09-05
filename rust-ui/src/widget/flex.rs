use std::any::Any;

use crate::geometrics::{Point, Size};
use crate::paint::{BoxConstraints, LayoutRequest};
use crate::support::generator::{Coroutine, Generator};

use super::element::{Children, Element, ElementId, IntoElement};
use super::message::MessageSink;
use super::paint_object::PaintObject;
use super::state::StateContainer;
use super::widget::{Widget, WidgetSeal};

pub struct Flex<Renderer> {
    direction: Axis,
    children: Vec<Element<Renderer>>,
    flex_params: Vec<FlexParam>,
}

pub struct FlexPaint;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    NonFlex,
    Flex,
}

#[derive(Clone, Copy, Default)]
struct FlexParam {
    flex: f32,
}

impl<Renderer> Flex<Renderer> {
    pub fn row() -> Self {
        Self {
            direction: Axis::Horizontal,
            flex_params: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn column() -> Self {
        Self {
            direction: Axis::Vertical,
            flex_params: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn add(mut self, child: impl IntoElement<Renderer>, flex: f32) -> Self {
        self.children.push(child.into_element());
        self.flex_params.push(FlexParam { flex });
        self
    }
}

impl<Renderer: 'static> Widget<Renderer> for Flex<Renderer> {
    type State = FlexPaint;
    type Message = ();

    fn initial_state(&self) -> StateContainer<Renderer, Self, Self::State, Self::Message> {
        StateContainer::from_paint_object(FlexPaint)
    }

    fn render(&self, _state: &Self::State, _element_id: ElementId) -> Children<Renderer> {
        self.children.clone()
    }

    fn layout<'a>(
        &'a self,
        _state: &mut Self::State,
        box_constraints: BoxConstraints,
        children: Vec<ElementId>,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co: Coroutine<LayoutRequest, Size>| async move {
            let mut flex_sum = 0.0;
            let mut total_non_flex = 0.0;
            let mut minor = self.direction.minor(&box_constraints.min);

            for (child_id, flex_param) in children.iter().zip(&self.flex_params) {
                if flex_param.flex_phase() == Phase::NonFlex {
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, box_constraints))
                        .await;

                    minor = self.direction.minor(&child_size).max(minor);
                    total_non_flex += self.direction.major(&child_size);
                }
                flex_sum += flex_param.flex;
            }

            for (child_id, flex_param) in children.iter().zip(&self.flex_params) {
                if flex_param.flex_phase() == Phase::Flex {
                    let total_major = self.direction.major(&box_constraints.max);
                    let remaining = (total_major - total_non_flex).max(0.0);
                    let major = remaining * flex_param.flex / flex_sum;

                    let child_box_constraints =
                        self.direction
                            .adjust_box_constraints(&box_constraints, major, major);
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, child_box_constraints))
                        .await;

                    minor = self.direction.minor(&child_size).max(minor);
                }
            }

            let total_major = self.direction.major(&box_constraints.max);
            let mut major = 0.0;

            for child_id in &children {
                let point = self.direction.pack_point(major, 0.0);
                let child_size = co
                    .suspend(LayoutRequest::ArrangeChild(*child_id, point))
                    .await;
                major += self.direction.major(&child_size);
            }

            self.direction.pack_size(total_major, minor)
        })
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Renderer> WidgetSeal for Flex<Renderer> {}

impl<Renderer> PaintObject<Renderer> for FlexPaint {
    type Widget = Flex<Renderer>;

    type Message = ();

    fn layout<'a>(
        &'a mut self,
        widget: &'a Self::Widget,
        box_constraints: BoxConstraints,
        children: Vec<ElementId>,
        _renderer: &mut Renderer,
        _messages: &mut MessageSink,
    ) -> Generator<'a, LayoutRequest, Size, Size> {
        Generator::new(move |co: Coroutine<LayoutRequest, Size>| async move {
            let mut flex_sum = 0.0;
            let mut total_non_flex = 0.0;
            let mut minor = widget.direction.minor(&box_constraints.min);

            for (child_id, flex_param) in children.iter().zip(&widget.flex_params) {
                if flex_param.flex_phase() == Phase::NonFlex {
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, box_constraints))
                        .await;

                    minor = widget.direction.minor(&child_size).max(minor);
                    total_non_flex += widget.direction.major(&child_size);
                }
                flex_sum += flex_param.flex;
            }

            for (child_id, flex_param) in children.iter().zip(&widget.flex_params) {
                if flex_param.flex_phase() == Phase::Flex {
                    let total_major = widget.direction.major(&box_constraints.max);
                    let remaining = (total_major - total_non_flex).max(0.0);
                    let major = remaining * flex_param.flex / flex_sum;

                    let child_box_constraints =
                        widget
                            .direction
                            .adjust_box_constraints(&box_constraints, major, major);
                    let child_size = co
                        .suspend(LayoutRequest::LayoutChild(*child_id, child_box_constraints))
                        .await;

                    minor = widget.direction.minor(&child_size).max(minor);
                }
            }

            let total_major = widget.direction.major(&box_constraints.max);
            let mut major = 0.0;

            for child_id in &children {
                let point = widget.direction.pack_point(major, 0.0);
                let child_size = co
                    .suspend(LayoutRequest::ArrangeChild(*child_id, point))
                    .await;
                major += widget.direction.major(&child_size);
            }

            widget.direction.pack_size(total_major, minor)
        })
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Axis {
    fn major(&self, size: &Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn minor(&self, size: &Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    fn pack_point(&self, major: f32, minor: f32) -> Point {
        match self {
            Axis::Horizontal => Point { x: major, y: minor },
            Axis::Vertical => Point { x: minor, y: major },
        }
    }

    fn pack_size(&self, major: f32, minor: f32) -> Size {
        match self {
            Axis::Horizontal => Size {
                width: major,
                height: minor,
            },
            Axis::Vertical => Size {
                width: minor,
                height: major,
            },
        }
    }

    fn adjust_box_constraints(
        &self,
        box_constraints: &BoxConstraints,
        min_major: f32,
        max_major: f32,
    ) -> BoxConstraints {
        match self {
            Axis::Horizontal => BoxConstraints {
                min: Size {
                    width: min_major,
                    height: box_constraints.min.height,
                },
                max: Size {
                    width: max_major,
                    height: box_constraints.max.height,
                },
            },
            Axis::Vertical => BoxConstraints {
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

impl FlexParam {
    fn flex_phase(&self) -> Phase {
        if self.flex == 0.0 {
            Phase::NonFlex
        } else {
            Phase::Flex
        }
    }
}
