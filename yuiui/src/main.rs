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

    fn update(&mut self, message: AppMessage) -> (bool, Effect<AppMessage>) {
        match message {
            AppMessage::CounterMessage(message) => {
                let (_, effect) = self.counter_store.update(message);
                (false, effect.map(AppMessage::CounterMessage))
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
        button.el_with(ButtonProps {
            label: "click me!".into(),
        }),
        counter.el().connect(
            |state: &AppState| &state.counter_store,
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
    let node = element.render(&mut context, &mut store);
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

impl<S, M, R> View<S, M, R> for Text {
    type State = TextState;

    type Children = ();

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::State {
        TextState
    }
}

impl<'event> EventListener<'event> for Text {
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

impl<C, S, M, R> View<S, M, R> for Block<C>
where
    C: ElementSeq<S, M, R>,
{
    type State = BlockState;

    type Children = C;

    fn build(
        &self,
        _children: &mut <Self::Children as ElementSeq<S, M, R>>::Storage,
        _store: &Store<S>,
        _renderer: &mut R,
    ) -> Self::State {
        BlockState
    }
}

impl<'event, C> EventListener<'event> for Block<C> {
    type Event = ();
}

#[derive(Debug)]
pub struct BlockState;

#[derive(Debug)]
pub struct ButtonProps {
    pub label: Cow<'static, str>,
}

pub fn button<S, M, R>(props: &ButtonProps, _store: &Store<S>) -> impl DebuggableElement<S, M, R> {
    Block::new().el_with(Text::new(props.label.clone()).el())
}

impl<S, M, R> ComponentProps<S, M, R> for ButtonProps {}

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

    fn update(&mut self, message: Self::Message) -> (bool, Effect<Self::Message>) {
        match message {
            CounterMessage::Increment => {
                self.count += 1;
            }
            CounterMessage::Decrement => {
                self.count -= 1;
            }
        }
        (true, Effect::none())
    }
}

fn counter<R>(
    _props: &(),
    store: &Store<CounterState>,
) -> impl DebuggableElement<CounterState, CounterMessage, R> {
    Block::new().el_with(Text::new(format!("{}", store.count)).el())
}
