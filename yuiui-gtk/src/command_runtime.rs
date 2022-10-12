use futures::stream::StreamExt as _;
use gtk::glib;
use yuiui::{CancellationToken, Command, RawToken, RawTokenVTable, TransferableEvent};

#[derive(Debug)]
pub struct CommandRuntime<T> {
    main_context: glib::MainContext,
    action_sender: glib::Sender<RenderAction<T>>,
}

impl<T: Send + 'static> CommandRuntime<T> {
    pub(super) fn new(
        main_context: glib::MainContext,
        action_sender: glib::Sender<RenderAction<T>>,
    ) -> Self {
        Self {
            main_context,
            action_sender,
        }
    }
}

impl<T: Send + 'static> CommandRuntime<T> {
    pub fn request_rerender(&self) {
        let action_sender = self.action_sender.clone();
        glib::idle_add_once(move || {
            action_sender.send(RenderAction::RequestRerender).unwrap();
        });
    }
}

impl<T: Send + 'static> yuiui::CommandRuntime<T> for CommandRuntime<T> {
    fn spawn_command(
        &mut self,
        command: Command<T>,
        cancellation_token: Option<CancellationToken>,
    ) {
        let action_sender = self.action_sender.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let message = future.await;
                action_sender.send(RenderAction::Message(message)).unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(message) = stream.next().await {
                    action_sender.send(RenderAction::Message(message)).unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let message = callback();
                action_sender.send(RenderAction::Message(message)).unwrap();
            }),
            Command::Interval(period, mut callback) => glib::timeout_add(period, move || {
                let message = callback();
                action_sender.send(RenderAction::Message(message)).unwrap();
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
        let _ = Box::from_raw(data as *mut glib::SourceId);
    }

    let data = Box::into_raw(Box::new(source_id)) as *const ();

    RawToken::new(data, &VTABLE)
}

#[derive(Debug)]
pub(super) enum RenderAction<T> {
    Message(T),
    Event(TransferableEvent),
    RequestRerender,
}
