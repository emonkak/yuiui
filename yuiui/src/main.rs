use either_macro::either;
use hlist::hlist;
use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use yuiui::*;

#[derive(Debug)]
struct AppState {
    count: Data<i64>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum AppMessage {
    Increment,
    Decrement,
}

impl State for AppState {
    type Message = AppMessage;

    fn reduce(&mut self, message: AppMessage) -> bool {
        match message {
            AppMessage::Increment => self.count.value += 1,
            AppMessage::Decrement => self.count.value -= 1,
        }
        true
    }
}

fn app() -> impl DebuggableElement<AppState, ()> {
    Block::new().el_with(hlist![
        Block::new().el_with(vec![Text::new("hello").el(), Text::new("world").el()]),
        Block::new().el_with(Text::new("hello world!").el()),
        Block::new().el_with(Some(Text::new("hello world!").el())),
        Block::new().el_with(either! {
            match 0 {
                0 => Text::new("foo").el(),
                1 => Some(Text::new("foo").el()),
                _ => vec![Text::new("foo").el()],
            }
        }),
        Text::new("!").el(),
        button(ButtonProps {
            label: "click me!".into(),
        }),
        counter().scope(|state: &AppState| &state.count),
    ])
}

fn main() {
    let state = AppState {
        count: Data::from(0),
    };
    let element = app();
    let mut context = RenderContext::new();
    let node = element.render(&state, &(), &mut context);
    println!("{:#?}", node);
}

#[derive(Debug)]
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

impl<S, B> View<S, B> for Text
where
    S: State,
{
    type Widget = TextWidget;

    type Children = hlist::HNil;

    fn build(
        &self,
        _children: &<Self::Children as ElementSeq<S, B>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _backend: &B,
    ) -> Self::Widget {
        TextWidget
    }
}

impl<'event> HasEvent<'event> for Text {
    type Event = ();
}

#[derive(Debug)]
pub struct TextWidget;

#[derive(Debug)]
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

impl<C, S, B> View<S, B> for Block<C>
where
    C: ElementSeq<S, B>,
    S: State,
{
    type Widget = BlockWidget;

    type Children = C;

    fn build(
        &self,
        _children: &<Self::Children as ElementSeq<S, B>>::Storage,
        _id_path: &IdPath,
        _state: &S,
        _backend: &B,
    ) -> Self::Widget {
        BlockWidget
    }
}

impl<'event, C> HasEvent<'event> for Block<C> {
    type Event = ();
}

#[derive(Debug)]
pub struct BlockWidget;

#[derive(Debug)]
pub struct ButtonProps {
    pub label: Cow<'static, str>,
}

pub fn button<S: State, B>(
    props: ButtonProps,
) -> ComponentElement<FunctionComponent<ButtonProps, impl DebuggableElement<S, B>, S, B>> {
    FunctionComponent::new(props, |props, _state, _backend| {
        Block::new().el_with(Text::new(props.label.clone()).el())
    })
    .el()
}

pub fn counter<E>(
) -> ComponentElement<FunctionComponent<(), impl DebuggableElement<Data<i64>, E>, Data<i64>, E>> {
    FunctionComponent::new((), |_props, state: &Data<i64>, _backend| {
        Block::new().el_with(Text::new(format!("{}", state.value)).el())
    })
    .el()
}
