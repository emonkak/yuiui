use std::rc::Rc;
use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;

use crate::geometrics::PhysicalRectangle;
use crate::graphics::{Background, Color, Primitive};

#[derive(Debug)]
pub struct Pipeline<Connection: self::Connection> {
    connection: Rc<Connection>,
    colormap: xproto::Colormap,
    draw_ops: Vec<DrawOp>,
}

#[derive(Debug)]
pub enum DrawOp {
    FillRectangle(xproto::AllocColorReply, PhysicalRectangle),
}

impl<Connection: self::Connection> Pipeline<Connection> {
    pub fn create(connection: Rc<Connection>, screen_num: usize) -> Result<Self, ReplyOrIdError> {
        let colormap = connection.generate_id()?;
        let screen = &connection.setup().roots[screen_num];

        connection.create_colormap(
            xproto::ColormapAlloc::NONE,
            colormap,
            screen.root,
            screen.root_visual,
        )?;

        Ok(Self {
            connection,
            colormap,
            draw_ops: Vec::new(),
        })
    }

    pub fn draw_ops(&self) -> &[DrawOp] {
        &self.draw_ops
    }

    pub fn alloc_color(&self, color: Color) -> Result<xproto::AllocColorReply, ReplyOrIdError> {
        let [red, green, blue, _] = color.into_u16_components();

        let color = self
            .connection
            .alloc_color(self.colormap, red, green, blue)?
            .reply()?;

        Ok(color)
    }

    pub fn push(&mut self, primitive: Primitive, depth: usize) {
        match primitive {
            Primitive::None => {}
            Primitive::Batch(primitives) => {
                for primitive in primitives {
                    self.push(primitive, depth)
                }
            }
            Primitive::Transform(_transform, _primitive) => {
                // TODO:
            }
            Primitive::Clip(_bounds, _primitive) => {
                // TODO:
            }
            Primitive::Quad {
                bounds, background, ..
            } => {
                let background_color = match background {
                    Background::Color(color) => self.alloc_color(color).unwrap(),
                };
                self.draw_ops
                    .push(DrawOp::FillRectangle(background_color, bounds.snap()));
            }
            Primitive::Text { .. } => {
                // TODO:
            }
            Primitive::Cache(primitive) => {
                self.push((&*primitive).clone(), depth);
            }
        }
    }
}

impl<Connection: self::Connection> Drop for Pipeline<Connection> {
    fn drop(&mut self) {
        self.connection.free_colormap(self.colormap).unwrap();
    }
}
