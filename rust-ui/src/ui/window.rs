use raw_window_handle::HasRawWindowHandle;

use crate::geometrics::PhysicalRectangle;
use crate::graphics::Viewport;

pub trait Window: HasRawWindowHandle {
    type WindowId: Copy + Send;

    fn window_id(&self) -> Self::WindowId;

    fn get_bounds(&self) -> PhysicalRectangle;

    fn get_scale_factor(&self) -> f32;

    fn get_viewport(&self) -> Viewport {
        Viewport::from_physical(self.get_bounds().size(), self.get_scale_factor())
    }

    fn invalidate(&self, bounds: PhysicalRectangle);

    fn show(&self);
}
