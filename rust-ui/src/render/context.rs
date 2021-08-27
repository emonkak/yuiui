use std::sync::mpsc::Sender;

use crate::widget::WidgetId;

#[derive(Debug, Clone)]
pub struct RenderContext {
    widget_id: WidgetId,
    update_sender: Sender<WidgetId>,
}

impl RenderContext {
    pub fn new(widget_id: WidgetId, update_sender: Sender<WidgetId>) -> Self {
        Self {
            widget_id,
            update_sender,
        }
    }

    pub fn request_update(&self) {
        self.update_sender.send(self.widget_id).unwrap();
    }
}

unsafe impl Sync for RenderContext {}
