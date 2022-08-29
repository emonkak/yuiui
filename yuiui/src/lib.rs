mod adapt;
mod command;
mod component;
mod component_node;
mod context;
mod effect;
mod element;
mod event;
mod id;
mod sequence;
mod state;
mod view;
mod widget;
mod widget_node;
mod widget_tree;

pub use command::{Command, CommandHandler};
pub use component::{Component, FunctionComponent};
pub use component_node::{ComponentNode, ComponentStack};
pub use effect::Effect;
pub use element::{ComponentElement, DebuggableElement, Element, ElementSeq, ViewElement};
pub use event::Event;
pub use id::{Id, IdPath, NodeId, NodePath};
pub use state::{Data, State};
pub use view::View;
pub use widget::{Widget, WidgetEvent, WidgetLifeCycle};
pub use widget_node::{WidgetNode, WidgetNodeSeq};
pub use widget_tree::WidgetTree;

use std::borrow::Cow;
use std::fmt;
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

impl<S, E> View<S, E> for Text
where
    S: State,
{
    type Widget = Text;

    type Children = hlist::HNil;

    fn build(
        &self,
        _children: &<Self::Widget as Widget<S, E>>::Children,
        _state: &S,
        _env: &E,
    ) -> Self::Widget {
        self.clone()
    }
}

impl<S, E> Widget<S, E> for Text
where
    S: State,
{
    type Children = hlist::HNil;
}

impl<'event> WidgetEvent<'event> for Text {
    type Event = ();
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

impl<C, S, E> View<S, E> for Block<C>
where
    C: ElementSeq<S, E>,
    S: State,
{
    type Widget = BlockWidget<<C as ElementSeq<S, E>>::Store>;

    type Children = C;

    fn build(
        &self,
        _children: &<Self::Widget as Widget<S, E>>::Children,
        _state: &S,
        _env: &E,
    ) -> Self::Widget {
        BlockWidget {
            children: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct BlockWidget<C> {
    children: PhantomData<C>,
}

impl<C, S, E> Widget<S, E> for BlockWidget<C>
where
    C: WidgetNodeSeq<S, E>,
    S: State,
{
    type Children = C;
}

impl<'event, C> WidgetEvent<'event> for BlockWidget<C> {
    type Event = ();
}

impl<C> fmt::Debug for BlockWidget<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BlockWidget")
    }
}

#[derive(Debug)]
pub struct ButtonProps {
    pub label: Cow<'static, str>,
}

#[allow(non_snake_case)]
pub fn Button<S: State, E>(
    props: ButtonProps,
) -> FunctionComponent<ButtonProps, (), impl DebuggableElement<S, E>, S, E> {
    fn render<S: State, E>(
        props: &ButtonProps,
        _local_state: &(),
        _state: &S,
        _env: &E,
    ) -> impl DebuggableElement<S, E> {
        Block::new().el_with(Text::new(props.label.clone()).el())
    }

    FunctionComponent {
        props,
        render,
        should_update: None,
        lifecycle: None,
    }
}

#[allow(non_snake_case)]
pub fn Counter<E>() -> FunctionComponent<(), (), impl DebuggableElement<Data<i64>, E>, Data<i64>, E>
{
    fn render<E>(
        _props: &(),
        _local_state: &(),
        state: &Data<i64>,
        _env: &E,
    ) -> impl DebuggableElement<Data<i64>, E> {
        Block::new().el_with(Text::new(format!("{}", state.value)).el())
    }

    FunctionComponent {
        props: (),
        render,
        should_update: None,
        lifecycle: None,
    }
}
