use gtk::glib;
use gtk::prelude::*;
use std::time::{Duration, Instant};
use yuiui::{Element, RenderFlow, RenderLoop, State, Store, View};

use crate::backend::GtkBackend;
use crate::execution_context::{ExecutionContext, RenderAction};

const DEALINE_PERIOD: Duration = Duration::from_millis(50);

pub struct Application {
    application: gtk::Application,
    window: gtk::ApplicationWindow,
}

impl Application {
    pub fn new(application: gtk::Application, window: gtk::ApplicationWindow) -> Self {
        Self {
            application,
            window,
        }
    }

    pub fn start<E, S, M>(self, element: E, mut store: Store<S>)
    where
        E: Element<S, M, GtkBackend> + 'static,
        <E::View as View<S, M, GtkBackend>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        let (event_tx, event_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (action_tx, action_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let context = ExecutionContext::new(glib::MainContext::default(), action_tx.clone());

        let mut backend = GtkBackend::new(self.application, self.window, event_tx);
        let mut render_loop = RenderLoop::create(element, &mut store);

        render_loop.run_forever(&context, &mut store, &mut backend);

        {
            let widget = render_loop.node().state().as_view_state().unwrap().as_ref();
            backend.window().set_child(Some(widget));
        }

        event_rx.attach(None, move |(event, destination)| {
            action_tx
                .send(RenderAction::Event(event, destination))
                .unwrap();
            glib::Continue(true)
        });

        action_rx.attach(None, move |action| {
            let deadline = Instant::now() + DEALINE_PERIOD;

            match action {
                RenderAction::RequestRender => {}
                RenderAction::Message(message) => {
                    render_loop.push_message(message);
                }
                RenderAction::Event(event, destination) => {
                    render_loop.push_event(event, destination);
                }
            }

            if render_loop.run(&deadline, &context, &mut store, &mut backend) == RenderFlow::Suspend
            {
                context.request_render();
            }

            glib::Continue(true)
        });
    }
}
