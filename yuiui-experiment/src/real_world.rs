use crate::element::Element;
use crate::view::View;
use crate::widget::WidgetPod;

pub struct RealWorld<E: Element> {
    widget_pod: WidgetPod<<E::View as View>::Widget>,
}

impl<E: Element> RealWorld<E> {
    pub fn new(widget_pod: WidgetPod<<E::View as View>::Widget>) -> Self {
        Self {
            widget_pod,
        }
    }

    pub fn widget_pod(&self) -> &WidgetPod<<E::View as View>::Widget> {
        &self.widget_pod
    }
}

