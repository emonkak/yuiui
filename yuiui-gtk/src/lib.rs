mod backend;
mod window;

pub use backend::{Backend, BackendProxy};
pub use window::ApplicationWindow;

use glib::MainContext;
use gtk::Application;
use std::time::{Duration, Instant};
use yuiui::{Deadline, Element, Forever, RenderFlow, RenderLoop, State};

use backend::Action;

const DEALINE_PERIOD: Duration = Duration::from_millis(50);

pub fn run<El, S>(application: Application, element: El, mut state: S)
where
    El: Element<S, Backend<S>> + 'static,
    S: State,
{
    let (sender, receiver) = MainContext::channel(glib::PRIORITY_DEFAULT);
    let backend = Backend::new(
        application,
        MainContext::default(),
        BackendProxy::new(sender),
    );
    let mut render_loop = RenderLoop::build(element, &state, &backend);

    render_loop.run(&Forever, &mut state, &backend);

    receiver.attach(None, move |action| {
        let deadline = Instant::now() + DEALINE_PERIOD;

        match action {
            Action::RequestRender => {}
            Action::DispatchEvent(event, destination) => {
                render_loop.dispatch_event(event, destination, &state, &backend);

                if deadline.did_timeout() {
                    backend.request_render();
                    return glib::Continue(true);
                }
            }
            Action::PushEffect(effect) => {
                render_loop.push_effect(effect);
            }
        }

        if render_loop.run(&deadline, &mut state, &backend) == RenderFlow::Suspended {
            backend.request_render();
        }

        glib::Continue(true)
    });
}
