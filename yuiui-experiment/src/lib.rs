mod children;
mod component;
mod view;
mod world;
mod element;

use std::borrow::Cow;
use std::convert::Infallible;
use std::marker::PhantomData;

pub use children::{Children, Either};
pub use component::Component;
pub use element::{Element, view, component};
pub use view::View;
pub use world::{Id, World};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Text {
    content: Cow<'static, str>,
}

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content: content.into()
        }
    }
}

impl View for Text {
    type Children = ();
}

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

impl<Children: self::Children> View for Block<Children> {
    type Children = Children;
}

#[derive(Debug)]
pub struct Button {
    label: Cow<'static, str>,
}

impl Button {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into()
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
