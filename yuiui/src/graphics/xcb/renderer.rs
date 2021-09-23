use std::rc::Rc;
use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyOrIdError};
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;

use super::pipeline::{DrawOp, Pipeline};
use crate::geometrics::{PhysicalRectangle, PhysicalSize, Viewport};
use crate::graphics::{Color, Primitive};

#[derive(Debug)]
pub struct Renderer<Connection> {
    connection: Rc<Connection>,
    screen_num: usize,
    window: xproto::Window,
}

#[derive(Debug)]
pub struct Surface<Connection: self::Connection> {
    connection: Rc<Connection>,
    pixmap: xproto::Pixmap,
    gc: xproto::Gcontext,
}

impl<Connection: self::Connection> Renderer<Connection> {
    pub fn new(connection: Rc<Connection>, screen_num: usize, window: xproto::Window) -> Self {
        Self {
            connection,
            screen_num,
            window,
        }
    }

    fn fill_rectangle(
        &self,
        surface: &Surface<Connection>,
        color: &xproto::AllocColorReply,
        bounds: PhysicalRectangle,
    ) -> Result<(), ConnectionError> {
        self.connection.change_gc(
            surface.gc,
            &xproto::ChangeGCAux::new().foreground(color.pixel),
        )?;
        self.connection.poly_fill_rectangle(
            surface.pixmap,
            surface.gc,
            &[xproto::Rectangle {
                x: bounds.x as _,
                y: bounds.y as _,
                width: bounds.width as _,
                height: bounds.height as _,
            }],
        )?;
        Ok(())
    }

    fn commit(
        &self,
        surface: &Surface<Connection>,
        size: PhysicalSize,
    ) -> Result<(), ConnectionError> {
        self.connection.copy_area(
            surface.pixmap,
            self.window,
            surface.gc,
            0,
            0,
            size.width as _,
            size.height as _,
            0,
            0,
        )?;
        self.connection.flush()?;
        Ok(())
    }

    fn process_draw_op(
        &self,
        draw_op: &DrawOp,
        surface: &Surface<Connection>,
    ) -> Result<(), ConnectionError> {
        match draw_op {
            DrawOp::FillRectangle(color, bounds) => {
                self.fill_rectangle(surface, color, *bounds)?;
            }
        }
        Ok(())
    }
}

impl<Connection: self::Connection> crate::graphics::Renderer for Renderer<Connection> {
    type Surface = self::Surface<Connection>;
    type Pipeline = self::Pipeline<Connection>;

    fn create_surface(&mut self, viewport: &Viewport) -> Self::Surface {
        Surface::create(
            self.connection.clone(),
            self.screen_num,
            viewport.physical_size(),
        )
        .unwrap()
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, viewport: &Viewport) {
        *surface = Surface::create(
            self.connection.clone(),
            self.screen_num,
            viewport.physical_size(),
        )
        .unwrap()
    }

    fn create_pipeline(&mut self, _viewport: &Viewport) -> Self::Pipeline {
        Pipeline::create(self.connection.clone(), self.screen_num).unwrap()
    }

    fn perform_pipeline(
        &mut self,
        surface: &mut Self::Surface,
        pipeline: &mut Self::Pipeline,
        viewport: &Viewport,
        background_color: Color,
    ) {
        let alloc_background_color = pipeline.alloc_color(background_color).unwrap();

        self.fill_rectangle(
            surface,
            &alloc_background_color,
            PhysicalRectangle::from_size(viewport.physical_size()),
        )
        .unwrap();

        for draw_op in pipeline.draw_ops() {
            self.process_draw_op(draw_op, surface).unwrap();
        }

        self.commit(surface, viewport.physical_size()).unwrap();
    }

    fn update_pipeline(
        &mut self,
        pipeline: &mut Self::Pipeline,
        primitive: Primitive,
        depth: usize,
    ) {
        pipeline.push(primitive, depth);
    }
}

impl<Connection: self::Connection> Surface<Connection> {
    pub fn create(
        connection: Rc<Connection>,
        screen_num: usize,
        size: PhysicalSize,
    ) -> Result<Self, ReplyOrIdError> {
        let pixmap = connection.generate_id()?;
        let screen = &connection.setup().roots[screen_num];
        connection.create_pixmap(0, pixmap, screen.root, size.width as _, size.height as _)?;

        let gc = connection.generate_id()?;
        connection.create_gc(
            gc,
            screen.root,
            &xproto::CreateGCAux::default().foreground(screen.white_pixel),
        )?;

        Ok(Self {
            connection,
            pixmap,
            gc,
        })
    }
}

impl<Connection: self::Connection> Drop for Surface<Connection> {
    fn drop(&mut self) {
        self.connection.free_gc(self.gc).unwrap();
        self.connection.free_pixmap(self.pixmap).unwrap();
    }
}
