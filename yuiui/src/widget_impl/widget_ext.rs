use crate::widget::Widget;

pub trait WidgetExt<Own: ?Sized>: Widget<Own> {}

impl<W: ?Sized + Widget<O>, O: ?Sized> WidgetExt<O> for W {}
