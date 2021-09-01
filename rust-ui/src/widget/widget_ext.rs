use super::element::{Key, WithKey};
use super::widget::Widget;

pub trait WidgetExt<Renderer>: Widget<Renderer> {
    #[inline]
    fn with_key(self, key: Key) -> WithKey<Self>
    where
        Self: Sized,
    {
        WithKey { widget: self, key }
    }
}
