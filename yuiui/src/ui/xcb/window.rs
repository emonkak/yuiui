use std::rc::Rc;

use raw_window_handle::unix::XcbHandle;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use x11rb::connection::Connection;
use x11rb::errors::ConnectionError;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

use crate::geometrics::PhysicalRectangle;
use crate::ui::WindowContainer;

#[derive(Debug)]
pub struct Window<Connection> {
    connection: Rc<Connection>,
    screen_num: usize,
    window_id: xproto::Window,
}

impl Window<XCBConnection> {
    pub fn create_container(
        connection: Rc<XCBConnection>,
        screen_num: usize,
        bounds: PhysicalRectangle,
        scale_factor: f32,
    ) -> Result<WindowContainer<Self>, ConnectionError> {
        let window_id = connection.generate_id().unwrap();
        let screen = &connection.setup().roots[screen_num];

        let event_mask = xproto::EventMask::EXPOSURE
            | xproto::EventMask::STRUCTURE_NOTIFY
            | xproto::EventMask::KEY_PRESS
            | xproto::EventMask::KEY_RELEASE
            | xproto::EventMask::BUTTON_PRESS
            | xproto::EventMask::BUTTON_RELEASE
            | xproto::EventMask::POINTER_MOTION;

        let window_aux = xproto::CreateWindowAux::new()
            .event_mask(event_mask)
            .background_pixel(screen.white_pixel);

        connection.create_window(
            screen.root_depth,
            window_id,
            screen.root,
            bounds.x as _,
            bounds.y as _,
            bounds.width as _,
            bounds.height as _,
            0,
            xproto::WindowClass::INPUT_OUTPUT,
            0,
            &window_aux,
        )?;

        let window = Self {
            connection,
            screen_num,
            window_id,
        };
        let window_container = WindowContainer::new(window, bounds.size(), scale_factor);
        Ok(window_container)
    }
}

impl Clone for Window<XCBConnection> {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            screen_num: self.screen_num,
            window_id: self.window_id,
        }
    }
}

impl crate::ui::Window for Window<XCBConnection> {
    type Id = xproto::Window;

    #[inline]
    fn id(&self) -> Self::Id {
        self.window_id
    }

    #[inline]
    fn show(&self) {
        self.connection.map_window(self.window_id).unwrap();
        self.connection.flush().unwrap();
    }

    fn request_redraw(&self, bounds: PhysicalRectangle) {
        let event = xproto::ExposeEvent {
            response_type: xproto::EXPOSE_EVENT,
            sequence: 0,
            window: self.window_id,
            x: bounds.x as _,
            y: bounds.y as _,
            width: bounds.width as _,
            height: bounds.height as _,
            count: 0,
        };

        xproto::send_event(
            &*self.connection,
            false,
            self.window_id,
            xproto::EventMask::NO_EVENT,
            event,
        )
        .unwrap();

        self.connection.flush().unwrap();
    }
}

unsafe impl HasRawWindowHandle for Window<XCBConnection> {
    #[inline]
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Xcb(XcbHandle {
            window: self.window_id,
            connection: self.connection.get_raw_xcb_connection(),
            ..XcbHandle::empty()
        })
    }
}
