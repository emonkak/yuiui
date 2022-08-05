use crate::element::Element;

pub trait Component: 'static {
    type Element: Element;

    fn render(&self) -> Self::Element;

    fn should_update(&self, _other: &Self) -> bool {
        true
    }
}
