mod backend;
mod window;

pub use backend::{Backend, BackendProxy};
pub use window::ApplicationWindow;

use glib::MainContext;
use gtk::Application;
use std::time::{Duration, Instant};
use yuiui::{Element, Forever, RenderFlow, RenderLoop, State};

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
            Action::PushEffect(path, effect) => {
                render_loop.push_effect(path, effect);
            }
        }

        if render_loop.run(&deadline, &mut state, &backend) == RenderFlow::Suspended {
            backend.schedule_render();
        }

        glib::Continue(true)
    });
}
