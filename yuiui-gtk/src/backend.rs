use futures::stream::StreamExt as _;
use glib::{MainContext, Sender, SourceId};
use gtk::Application;
use std::any::Any;
use yuiui::{
    Command, DestinedEffect, EffectContext, EventDestination, RawToken, RawTokenVTable,
    RenderLoopContext,
};

#[derive(Debug)]
pub struct Backend<M> {
    application: Application,
    main_context: MainContext,
    proxy: BackendProxy<M>,
}

impl<M: Send + 'static> Backend<M> {
    pub(super) fn new(
        application: Application,
        main_context: MainContext,
        proxy: BackendProxy<M>,
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

    pub fn proxy(&self) -> &BackendProxy<M> {
        &self.proxy
    }

    pub(super) fn request_render(&self) {
        let message_sender = self.proxy.sender.clone();
        glib::idle_add_once(move || {
            message_sender.send(Action::RequestRender).unwrap();
        });
    }
}

impl<M: Send + 'static> RenderLoopContext<M> for Backend<M> {
    fn invoke_command(&self, command: Command<M>, context: EffectContext) -> RawToken {
        let message_sender = self.proxy.sender.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let effect = future.await.destine(&context);
                message_sender.send(Action::PushEffect(effect)).unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(effect) = stream.next().await {
                    let effect = effect.destine(&context);
                    message_sender.send(Action::PushEffect(effect)).unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let effect = callback().destine(&context);
                message_sender.send(Action::PushEffect(effect)).unwrap();
            }),
            Command::Interval(period, callback) => glib::timeout_add(period, move || {
                let effect = callback().destine(&context);
                message_sender.send(Action::PushEffect(effect)).unwrap();
                glib::Continue(true)
            }),
        };
        create_token(source_id)
    }
}

#[derive(Debug, Clone)]
pub struct BackendProxy<M> {
    sender: Sender<Action<M>>,
}

impl<M> BackendProxy<M> {
    pub(super) fn new(sender: Sender<Action<M>>) -> Self {
        Self { sender }
    }

    pub fn push_effect(&self, effect: DestinedEffect<M>) {
        self.sender.send(Action::PushEffect(effect)).unwrap();
    }

    pub fn dispatch_event(&self, event: Box<dyn Any + Send>, destination: EventDestination) {
        self.sender
            .send(Action::DispatchEvent(event, destination))
            .unwrap();
    }
}

pub(super) enum Action<M> {
    RequestRender,
    DispatchEvent(Box<dyn Any + Send>, EventDestination),
    PushEffect(DestinedEffect<M>),
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
