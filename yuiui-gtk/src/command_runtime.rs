use futures::stream::StreamExt as _;
use gtk::glib;
use std::sync::mpsc::Sender;
use yuiui::{CancellationToken, Command, RawToken, RawTokenVTable};

#[derive(Debug)]
pub struct CommandRuntime<M> {
    main_context: glib::MainContext,
    message_sender: Sender<M>,
}

impl<M: Send + 'static> CommandRuntime<M> {
    pub(super) fn new(main_context: glib::MainContext, message_sender: Sender<M>) -> Self {
        Self {
            main_context,
            message_sender,
        }
    }

    pub fn main_context(&self) -> &glib::MainContext {
        &self.main_context
    }
}

impl<M: Send + 'static> CommandRuntime<M> {
    pub fn request_rerender(&self) {
        let main_context = self.main_context.clone();
        glib::idle_add_once(move || {
            main_context.wakeup();
        });
    }
}

impl<M: Send + 'static> yuiui::CommandRuntime<M> for CommandRuntime<M> {
    fn spawn_command(&self, command: Command<M>, cancellation_token: Option<CancellationToken>) {
        let message_sender = self.message_sender.clone();
        let main_context = self.main_context.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let message = future.await;
                message_sender.send(message).unwrap();
                main_context.wakeup();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(message) = stream.next().await {
                    message_sender.send(message).unwrap();
                    main_context.wakeup();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let message = callback();
                message_sender.send(message).unwrap();
                main_context.wakeup();
            }),
            Command::Interval(period, mut callback) => glib::timeout_add(period, move || {
                let message = callback();
                message_sender.send(message).unwrap();
                main_context.wakeup();
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
