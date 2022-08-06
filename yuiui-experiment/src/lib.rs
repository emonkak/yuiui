pub mod hlist;

mod component;
mod context;
mod element;
mod sequence;
mod stage;
mod view;
mod widget;

pub use component::Component;
pub use element::{component, view, ComponentElement, Element, ViewElement};
pub use sequence::{Either, ElementSeq, WidgetNodeSeq};
pub use stage::Stage;
pub use view::View;
pub use widget::Widget;

use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Text {
    content: Cow<'static, str>,
}

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl View for Text {
    type Widget = Text;

    type Children = hlist::HNil;

    fn build(self, _children: &<Self::Widget as Widget>::Children) -> Self::Widget {
        self
    }
}

impl Widget for Text {
    type Children = hlist::HNil;
}

#[derive(Debug, Clone)]
pub struct Block<Children> {
    children: PhantomData<Children>,
}

impl<Children> Block<Children> {
    pub fn new() -> Self {
        Self {
            children: PhantomData,
        }
    }
}

impl<Children: 'static + ElementSeq> View for Block<Children> {
    type Widget = BlockWidget<<Children as ElementSeq>::Nodes>;

    type Children = Children;

    fn build(self, _children: &<Self::Widget as Widget>::Children) -> Self::Widget {
        BlockWidget {
            children: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockWidget<Children> {
    children: PhantomData<Children>,
}

impl<Children: 'static + WidgetNodeSeq> Widget for BlockWidget<Children> {
    type Children = Children;
}

#[derive(Debug, Clone)]
pub struct Button {
    label: Cow<'static, str>,
}

impl Button {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl Component for Button {
    type Element = ViewElement<Block<hlist_type![ViewElement<Text>]>>;

    type State = ();

    fn render(&self, _state: &Self::State) -> Self::Element {
        view(
            Block::new(),
            hlist![view(Text::new(self.label.clone()), hlist![])],
        )
    }
}
