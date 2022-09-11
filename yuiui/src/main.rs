use either_macro::either;
use hlist::hlist;
use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use yuiui::*;

#[derive(Debug)]
struct AppState {
    counter_store: Store<CounterState>,
}

#[derive(Debug)]
enum AppMessage {
    CounterMessage(CounterMessage),
}

impl State for AppState {
    type Message = AppMessage;

    fn update(&mut self, message: AppMessage) -> (bool, CommandBatch<AppMessage>) {
        match message {
            AppMessage::CounterMessage(message) => {
                let (dirty, commands) = self.counter_store.update(message);
                (dirty, commands.map(AppMessage::CounterMessage))
            }
        }
    }
}

fn app() -> impl DebuggableElement<AppState, AppMessage, ()> {
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
        counter().connect(
            |state: &Store<AppState>| &state.counter_store,
            AppMessage::CounterMessage,
        ),
    ])
}

fn main() {
    let mut store = Store::new(AppState {
        counter_store: Store::new(CounterState { count: 0 }),
    });
    let element = app();
    let mut context = RenderContext::new();
    let node = element.render(&mut context, &mut store, &mut ());
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

impl<S, M, B> View<S, M, B> for Text {
    type State = TextState;

    type Children = hlist::HNil;

    fn build(
        &self,
        _children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::State {
        TextState
    }
}

impl<'event> HasEvent<'event> for Text {
    type Event = ();
}

#[derive(Debug)]
pub struct TextState;

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

impl<C, S, M, B> View<S, M, B> for Block<C>
where
    C: ElementSeq<S, M, B>,
{
    type State = BlockState;

    type Children = C;

    fn build(
        &self,
        _children: &<Self::Children as ElementSeq<S, M, B>>::Storage,
        _store: &Store<S>,
        _backend: &mut B,
    ) -> Self::State {
        BlockState
    }
}

impl<'event, C> HasEvent<'event> for Block<C> {
    type Event = ();
}

#[derive(Debug)]
pub struct BlockState;

#[derive(Debug)]
pub struct ButtonProps {
    pub label: Cow<'static, str>,
}

pub fn button<S, M, B>(
    props: ButtonProps,
) -> ComponentElement<FunctionComponent<ButtonProps, (), impl DebuggableElement<S, M, B>, S, M, B>>
{
    FunctionComponent::new(props, |props, _local_state, _state, _backend| {
        Block::new().el_with(Text::new(props.label.clone()).el())
    })
    .el()
}

#[derive(Debug)]
struct CounterState {
    count: i64,
}

#[derive(Debug)]
#[allow(dead_code)]
enum CounterMessage {
    Increment,
    Decrement,
}

impl State for CounterState {
    type Message = CounterMessage;

    fn update(&mut self, message: Self::Message) -> (bool, CommandBatch<Self::Message>) {
        match message {
            CounterMessage::Increment => {
                self.count += 1;
            }
            CounterMessage::Decrement => {
                self.count -= 1;
            }
        }
        (true, CommandBatch::none())
    }
}

fn counter<B>() -> ComponentElement<
    FunctionComponent<
        (),
        (),
        impl DebuggableElement<CounterState, CounterMessage, B>,
        CounterState,
        CounterMessage,
        B,
    >,
> {
    FunctionComponent::new(
        (),
        |_props, _local_state, state: &CounterState, _backend| {
            Block::new().el_with(Text::new(format!("{}", state.count)).el())
        },
    )
    .el()
}
