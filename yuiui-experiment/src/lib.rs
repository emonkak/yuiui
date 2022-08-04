mod component;
mod element;
mod element_seq;
mod view;
mod widget;
mod world;

use std::borrow::Cow;
use std::convert::Infallible;
use std::marker::PhantomData;

pub use component::Component;
pub use element::{component, view, Element};
pub use element_seq::{ElementSeq, Either};
pub use view::View;
pub use widget::Widget;
pub use world::{Id, World};

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

impl Widget for Text {}

#[derive(Debug)]
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
    type Widget = BlockWidget;

    type Children = Children;

    fn build(&self, _children: &Self::Children) -> Self::Widget {
        BlockWidget {}
    }
}

pub struct BlockWidget {}

impl Widget for BlockWidget {}

#[derive(Debug)]
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
    type View = Block<(Element<Text, Infallible>,)>;

    type Component = Infallible;

    fn render(&self) -> Element<Self::View, Self::Component> {
        view(Block::new(), (view(Text::new(self.label.clone()), ()),))
    }
}
