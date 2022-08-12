pub mod hlist;

mod adapt;
mod component;
mod context;
mod element;
mod sequence;
mod stage;
mod state;
mod view;
mod widget;

pub use component::Component;
pub use element::{ComponentElement, Element, ViewElement};
pub use sequence::{ElementSeq, WidgetNodeSeq};
pub use stage::Stage;
pub use state::{Data, Effect, State};
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

impl<S: State> View<S> for Text {
    type Widget = Text;

    type Children = hlist::HNil;

    fn build(self, _children: &<Self::Widget as Widget<S>>::Children, _state: &S) -> Self::Widget {
        self
    }
}

impl<S: State> Widget<S> for Text {
    type Children = hlist::HNil;
}

#[derive(Debug, Clone)]
pub struct Block<C> {
    children: PhantomData<C>,
}

impl<C> Block<C> {
    pub fn new() -> Self {
        Self {
            children: PhantomData,
        }
    }
}

impl<C, S> View<S> for Block<C>
where
    C: ElementSeq<S>,
    S: State,
{
    type Widget = BlockWidget<<C as ElementSeq<S>>::Store>;

    type Children = C;

    fn build(self, _children: &<Self::Widget as Widget<S>>::Children, _state: &S) -> Self::Widget {
        BlockWidget {
            children: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockWidget<C> {
    children: PhantomData<C>,
}

impl<C, S> Widget<S> for BlockWidget<C>
where
    C: WidgetNodeSeq<S>,
    S: State,
{
    type Children = C;
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

impl<S: State> Component<S> for Button {
    type Element = ViewElement<Block<HList![ViewElement<Text, S>]>, S>;

    fn render(&self, _state: &S) -> Self::Element {
        Block::new().el_with(hlist![Text::new(self.label.clone()).el()])
    }
}

#[derive(Debug, Clone)]
pub struct Counter;

impl Component<Data<i64>> for Counter {
    type Element = ViewElement<Block<HList![ViewElement<Text, Data<i64>>]>, Data<i64>>;

    fn render(&self, state: &Data<i64>) -> Self::Element {
        Block::new().el_with(hlist![Text::new(format!("{}", state.value)).el()])
    }
}
