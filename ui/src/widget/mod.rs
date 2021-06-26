pub mod fill;
pub mod flex;
pub mod null;
pub mod padding;
pub mod widget;

#[macro_export]
macro_rules! element {
    ($expr:expr => { $($content:tt)* }) => {
        $crate::widget::widget::Element::build($expr, __element_children!([] $($content)*))
    };
    ($expr:expr) => {
        $crate::widget::widget::Element::build($expr, [])
    };
}

#[macro_export]
macro_rules! __element_children {
    ([$($children:expr)*] $expr:expr => { $($content:tt)* } $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::widget::Child::Single($crate::widget::widget::Element::build($expr, __element_children!([] $($content)*)))] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr; $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::widget::Child::from($expr)] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::widget::Child::from($expr)])
    };
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
