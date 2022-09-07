use futures::stream::StreamExt as _;
use glib::{MainContext, Sender, SourceId};
use gtk::Application;
use yuiui::{
    CancellationToken, Command, ComponentIndex, Effect, IdPathBuf, RawToken, RawTokenVTable,
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

    pub(super) fn schedule_render(&self) {
        let message_sender = self.proxy.sender.clone();
        glib::idle_add_once(move || {
            message_sender.send(Action::RequestRender).unwrap();
        });
    }
}

impl<S: State> RenderLoopContext<S> for Backend<S> {
    fn invoke_command(
        &self,
        id_path: IdPathBuf,
        component_index: ComponentIndex,
        command: Command<S>,
        cancellation_token: Option<CancellationToken>,
    ) {
        let message_sender = self.proxy.sender.clone();
        let source_id = match command {
            Command::Future(future) => self.main_context.spawn_local(async move {
                let effect = future.await;
                message_sender
                    .send(Action::PushEffect(id_path, component_index, effect))
                    .unwrap();
            }),
            Command::Stream(mut stream) => self.main_context.spawn_local(async move {
                while let Some(effect) = stream.next().await {
                    message_sender
                        .send(Action::PushEffect(id_path.clone(), component_index, effect))
                        .unwrap();
                }
            }),
            Command::Timeout(duration, callback) => glib::timeout_add_once(duration, move || {
                let effect = callback();
                message_sender
                    .send(Action::PushEffect(id_path, component_index, effect))
                    .unwrap();
            }),
            Command::Interval(period, callback) => glib::timeout_add(period, move || {
                let effect = callback();
                message_sender
                    .send(Action::PushEffect(id_path.clone(), component_index, effect))
                    .unwrap();
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

    pub fn push_effect(
        &self,
        id_path: IdPathBuf,
        component_index: ComponentIndex,
        effect: Effect<S>,
    ) {
        self.sender
            .send(Action::PushEffect(id_path, component_index, effect))
            .unwrap();
    }
}

pub(super) enum Action<S: State> {
    RequestRender,
    PushEffect(IdPathBuf, ComponentIndex, Effect<S>),
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
