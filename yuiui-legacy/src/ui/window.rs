use raw_window_handle::HasRawWindowHandle;

use crate::geometrics::{PhysicalRect, PhysicalSize, Viewport};

pub struct WindowContainer<Window> {
    window: Window,
    viewport: Viewport,
}

pub trait Window: HasRawWindowHandle {
    type Id: Copy + Eq + Send;

    fn id(&self) -> Self::Id;

    fn show(&self);

    fn request_redraw(&self, bounds: PhysicalRect);
}

impl<Window> WindowContainer<Window> {
    pub fn new(window: Window, size: PhysicalSize, scale_factor: f32) -> Self {
        let viewport = Viewport::from_physical(size, scale_factor);
        Self { window, viewport }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn resize_viewport(&mut self, size: PhysicalSize) -> bool {
        if self.viewport.physical_size() != size {
            self.viewport = Viewport::from_physical(size, self.viewport.scale_factor());
            true
        } else {
            false
        }
    }
}
