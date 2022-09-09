use futures::stream::StreamExt as _;
use glib::{MainContext, Sender, SourceId};
use std::any::Any;
use yuiui::{Command, CommandRunner, EventDestination, RawToken, RawTokenVTable, StateScope};

#[derive(Debug)]
pub struct ExecutionContext<T> {
    main_context: MainContext,
    port: Sender<RenderAction<T>>,
}

impl<T: Send + 'static> ExecutionContext<T> {
    pub(super) fn new(main_context: MainContext, port: Sender<RenderAction<T>>) -> Self {
        Self { main_context, port }
    }
}

impl<T: Send + 'static> ExecutionContext<T> {
    pub fn request_render(&self) {
        let port = self.port.clone();
        glib::idle_add_once(move || {
            port.send(RenderAction::RequestRender).unwrap();
        });
    }
}

impl<T: Send + 'static> CommandRunner<T> for ExecutionContext<T> {
    fn spawn_command(&self, command: Command<T>, state_scope: StateScope) -> RawToken {
        let port = self.port.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let message = future.await;
                port.send(RenderAction::Message(message, state_scope))
                    .unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(message) = stream.next().await {
                    port.send(RenderAction::Message(message, state_scope.clone()))
                        .unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let message = callback();
                port.send(RenderAction::Message(message, state_scope))
                    .unwrap();
            }),
            Command::Interval(period, callback) => glib::timeout_add(period, move || {
                let message = callback();
                port.send(RenderAction::Message(message, state_scope.clone()))
                    .unwrap();
                glib::Continue(true)
            }),
        };
        create_token(source_id)
    }
}

fn create_token(source_id: SourceId) -> RawToken {
    static VTABLE: RawTokenVTable = RawTokenVTable::new(cancel, drop);

    unsafe fn cancel(data: *const ()) {
        Box::from_raw(data as *mut SourceId).remove();
    }

    unsafe fn drop(data: *const ()) {
        Box::from_raw(data as *mut SourceId);
    }

    let data = Box::into_raw(Box::new(source_id)) as *const ();

    RawToken::new(data, &VTABLE)
}

pub(super) enum RenderAction<T> {
    Message(T, StateScope),
    Event(Box<dyn Any + Send>, EventDestination),
    RequestRender,
}
