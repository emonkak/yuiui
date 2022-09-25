use futures::stream::StreamExt as _;
use gtk::glib;
use std::any::Any;
use yuiui::{CancellationToken, Command, EventDestination, RawToken, RawTokenVTable};

#[derive(Debug)]
pub struct ExecutionContext<T> {
    main_context: glib::MainContext,
    action_port: glib::Sender<RenderAction<T>>,
}

impl<T: Send + 'static> ExecutionContext<T> {
    pub(super) fn new(
        main_context: glib::MainContext,
        action_port: glib::Sender<RenderAction<T>>,
    ) -> Self {
        Self {
            main_context,
            action_port,
        }
    }
}

impl<T: Send + 'static> ExecutionContext<T> {
    pub fn request_render(&self) {
        let action_port = self.action_port.clone();
        glib::idle_add_once(move || {
            action_port.send(RenderAction::RequestRender).unwrap();
        });
    }
}

impl<T: Send + 'static> yuiui::ExecutionContext<T> for ExecutionContext<T> {
    fn spawn_command(&self, command: Command<T>, cancellation_token: Option<CancellationToken>) {
        let action_port = self.action_port.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let message = future.await;
                action_port.send(RenderAction::Message(message)).unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(message) = stream.next().await {
                    action_port.send(RenderAction::Message(message)).unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let message = callback();
                action_port.send(RenderAction::Message(message)).unwrap();
            }),
            Command::Interval(period, callback) => glib::timeout_add(period, move || {
                let message = callback();
                action_port.send(RenderAction::Message(message)).unwrap();
                glib::Continue(true)
            }),
        };
        if let Some(cancellation_token) = cancellation_token {
            let token = create_token(source_id);
            cancellation_token.register(token);
        }
    }
}

fn create_token(source_id: glib::SourceId) -> RawToken {
    static VTABLE: RawTokenVTable = RawTokenVTable::new(cancel, drop);

    unsafe fn cancel(data: *const ()) {
        Box::from_raw(data as *mut glib::SourceId).remove();
    }

    unsafe fn drop(data: *const ()) {
        Box::from_raw(data as *mut glib::SourceId);
    }

    let data = Box::into_raw(Box::new(source_id)) as *const ();

    RawToken::new(data, &VTABLE)
}

#[derive(Debug)]
pub(super) enum RenderAction<T> {
    Message(T),
    Event(Box<dyn Any + Send>, EventDestination),
    RequestRender,
}
