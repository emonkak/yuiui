use std::ops::Add;
use std::collections::VecDeque;

use crate::base::Rectangle;
use crate::graphics::background::Background;
use crate::graphics::color::Color;

#[derive(Debug, Clone)]
pub enum DrawOp {
    None,
    Group(VecDeque<DrawOp>),
    Quad {
        bounds: Rectangle,
        background: Background,
        border_radius: f32,
        border_width: f32,
        border_color: Color,
    },
}

impl Default for DrawOp {
    fn default() -> DrawOp {
        DrawOp::None
    }
}

impl Add for DrawOp {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::None, y) => y,
            (x, Self::None) => x,
            (Self::Group(mut xs), Self::Group(ys)) => {
                xs.extend(ys);
                Self::Group(xs)
            }
            (Self::Group(mut xs), y) => {
                xs.push_back(y);
                Self::Group(xs)
            }
            (x, Self::Group(mut ys)) => {
                ys.push_front(x);
                Self::Group(ys)
            }
            (x, y) => {
                let mut xs = VecDeque::with_capacity(2);
                xs.push_back(x);
                xs.push_back(y);
                Self::Group(xs)
            }
        }
    }
}
