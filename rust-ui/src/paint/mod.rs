pub mod tree;

use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};
use crate::geometrics::Rectangle;

pub struct PaintContext<'a, Handle> {
    event_manager: &'a mut EventManager<Handle>,
    painter: &'a mut dyn Painter<Handle>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PaintHint {
    Always,
    Once,
}

pub trait Painter<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

impl<'a, Handle> PaintContext<'a, Handle> {
    pub fn add_handler(
        &mut self,
        handler: Arc<dyn EventHandler<Handle> + Send + Sync>,
    ) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(
        &mut self,
        handler_id: HandlerId,
    ) -> Arc<dyn EventHandler<Handle> + Send + Sync> {
        self.event_manager.remove(handler_id)
    }
}

impl<'a, Handle> Painter<Handle> for PaintContext<'a, Handle> {
    #[inline]
    fn handle(&self) -> &Handle {
        self.painter.handle()
    }

    #[inline]
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        self.painter.fill_rectangle(color, rectangle)
    }

    #[inline]
    fn commit(&mut self, rectangle: &Rectangle) {
        self.painter.commit(rectangle)
    }
}
