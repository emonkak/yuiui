mod component;
mod context;
mod element;
mod element_seq;
mod node;
mod stage;
mod view;
mod widget;

use std::borrow::Cow;
use std::marker::PhantomData;

pub use component::Component;
pub use element::{component, view, ComponentElement, Element, ViewElement};
pub use element_seq::{Either, ElementSeq};
pub use stage::Stage;
pub use view::View;
pub use widget::Widget;

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

    type Children = ();

    fn build(&self, _children: &Self::Children) -> Self::Widget {
        self.clone()
    }
}

impl Widget for Text {
    type Children = ();
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

impl<Children: ElementSeq> View for Block<Children> {
    type Widget = BlockWidget<<Children as ElementSeq>::UINodes>;

    type Children = Children;

    fn build(&self, _children: &<Self::Children as ElementSeq>::VNodes) -> Self::Widget {
        BlockWidget {
            children: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockWidget<Children> {
    children: PhantomData<Children>,
}

impl<Children: 'static> Widget for BlockWidget<Children> {
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
    type Element = ViewElement<Block<(ViewElement<Text>,)>>;

    fn render(&self) -> Self::Element {
        view(Block::new(), (view(Text::new(self.label.clone()), ()),))
    }
}
