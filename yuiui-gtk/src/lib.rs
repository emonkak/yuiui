mod backend;
mod execution_context;
mod window;

pub use window::ApplicationWindow;

use glib::MainContext;
use gtk::prelude::*;
use gtk::Application;
use std::time::{Duration, Instant};
use yuiui::{Element, Forever, RenderFlow, RenderLoop, State, Store};

use backend::Backend;
use execution_context::{ExecutionContext, RenderAction};

const DEALINE_PERIOD: Duration = Duration::from_millis(50);

pub fn run<El, S, M>(element: El, mut store: Store<S>)
where
    El: Element<S, M, Backend> + 'static,
    S: State<Message = M>,
    M: Send + 'static,
{
    let (event_tx, event_rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
    let (action_tx, action_rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

    let application = Application::new(None, Default::default());
    let context = ExecutionContext::new(MainContext::default(), action_tx.clone());

    let mut backend = Backend::new(application.clone(), event_tx);
    let mut render_loop = RenderLoop::create(element, &store, &mut backend);

    render_loop.run(&Forever, &context, &mut store, &mut backend);

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
            RenderAction::Message(message, state_scope) => {
                render_loop.push_message(message, state_scope);
            }
            RenderAction::Event(event, destination) => {
                render_loop.push_event(event, destination);
            }
        }

        if render_loop.run(&deadline, &context, &mut store, &mut backend) == RenderFlow::Suspended {
            context.request_render();
        }

        glib::Continue(true)
    });

    application.run();
}
