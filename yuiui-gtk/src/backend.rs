use futures::stream::StreamExt as _;
use glib::{MainContext, Sender, SourceId};
use gtk::Application;
use std::any::Any;
use yuiui::{
    CancellationToken, Command, Effect, EventDestination, RawToken, RawTokenVTable,
    RenderLoopContext, State,
};

#[derive(Debug)]
pub struct Backend<S: State> {
    application: Application,
    main_context: MainContext,
    proxy: BackendProxy<S>,
}

impl<S: State> Backend<S> {
    pub(super) fn new(
        application: Application,
        main_context: MainContext,
        proxy: BackendProxy<S>,
    ) -> Self {
        Self {
            application,
            main_context,
            proxy,
        }
    }

    pub fn application(&self) -> &Application {
        &self.application
    }

    pub fn proxy(&self) -> &BackendProxy<S> {
        &self.proxy
    }

    pub(super) fn request_render(&self) {
        let message_sender = self.proxy.sender.clone();
        glib::idle_add_once(move || {
            message_sender.send(Action::RequestRender).unwrap();
        });
    }
}

impl<S: State> RenderLoopContext<S> for Backend<S> {
    fn invoke_command(&self, command: Command<S>, cancellation_token: Option<CancellationToken>) {
        let message_sender = self.proxy.sender.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let effect = future.await;
                message_sender.send(Action::PushEffect(effect)).unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(effect) = stream.next().await {
                    message_sender.send(Action::PushEffect(effect)).unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let effect = callback();
                message_sender.send(Action::PushEffect(effect)).unwrap();
            }),
            Command::Interval(period, callback) => glib::timeout_add(period, move || {
                let effect = callback();
                message_sender.send(Action::PushEffect(effect)).unwrap();
                glib::Continue(true)
            }),
        };
        if let Some(cancellation_token) = cancellation_token {
            let token = create_token(source_id);
            cancellation_token.register(token);
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendProxy<S: State> {
    sender: Sender<Action<S>>,
}

impl<S: State> BackendProxy<S> {
    pub(super) fn new(sender: Sender<Action<S>>) -> Self {
        Self { sender }
    }

    pub fn push_effect(&self, effect: Effect<S>) {
        self.sender.send(Action::PushEffect(effect)).unwrap();
    }

    pub fn dispatch_event(&self, event: Box<dyn Any + Send>, destination: EventDestination) {
        self.sender
            .send(Action::DispatchEvent(event, destination))
            .unwrap();
    }
}

pub(super) enum Action<S: State> {
    RequestRender,
    DispatchEvent(Box<dyn Any + Send>, EventDestination),
    PushEffect(Effect<S>),
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
