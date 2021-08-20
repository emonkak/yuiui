use raw_window_handle::HasRawWindowHandle;

use crate::base::PhysicalRectangle;

pub trait Window: HasRawWindowHandle {
    type WindowId: Copy + Send;

    fn window_id(&self) -> Self::WindowId;

    fn get_bounds(&self) -> PhysicalRectangle;

    fn invalidate(&self, bounds: PhysicalRectangle);
}
