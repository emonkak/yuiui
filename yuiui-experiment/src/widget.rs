use crate::context::Id;

pub trait Widget: 'static {
    type Children;
}

impl Widget for () {
    type Children = ();
}

#[derive(Debug)]
pub struct WidgetPod<W: Widget> {
    pub(crate) id: Id,
    pub(crate) widget: W,
    pub(crate) children: W::Children,
}
