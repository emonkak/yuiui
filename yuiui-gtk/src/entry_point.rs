use gtk::glib;
use gtk::prelude::*;
use std::time::{Duration, Instant};
use yuiui::{Element, RenderFlow, RenderLoop, State, Store, View};

use crate::backend::GtkBackend;
use crate::execution_context::{ExecutionContext, RenderAction};

const DEALINE_PERIOD: Duration = Duration::from_millis(50);

pub trait EntryPoint: AsRef<gtk::Window> {
    fn attach_widget(&self, widget: &gtk::Widget) {
        let window = self.as_ref();
        window.set_child(Some(widget));
        window.show();
    }

    fn boot<E, S, M>(&self, element: E, state: S)
    where
        E: Element<S, M, GtkBackend> + 'static,
        <E::View as View<S, M, GtkBackend>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        let (event_tx, event_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (action_tx, action_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let context = ExecutionContext::new(glib::MainContext::default(), action_tx.clone());

        let mut store = Store::new(state);
        let mut backend = GtkBackend::new(self.as_ref().clone(), event_tx);
        let mut render_loop = RenderLoop::create(element, &mut store);

        render_loop.run_forever(&context, &mut store, &mut backend);

        let widget = render_loop.node().state().as_view_state().unwrap().as_ref();
        self.attach_widget(widget);

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

impl EntryPoint for gtk::ApplicationWindow {
}

impl EntryPoint for gtk::Window {
}
