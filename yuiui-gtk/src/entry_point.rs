use gtk::glib;
use gtk::prelude::*;
use std::time::{Duration, Instant};
use yuiui::{Element, RenderFlow, RenderLoop, State, Store, View};

use crate::backend::Backend;
use crate::command_runtime::{CommandRuntime, RenderAction};

pub trait EntryPoint<M>: Sized + 'static {
    fn attach_widget(&self, widget: &gtk::Widget);

    fn boot<E, S>(self, element: E, state: S)
    where
        E: Element<S, M, Backend<Self>> + 'static,
        <E::View as View<S, M, Backend<Self>>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        const DEALINE_PERIOD: Duration = Duration::from_millis(50);

        let (event_tx, event_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (action_tx, action_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut command_runtime =
            CommandRuntime::new(glib::MainContext::default(), action_tx.clone());
        let mut store = Store::new(state);
        let mut backend = Backend::new(self, event_tx);
        let mut render_loop = RenderLoop::create(element, &mut store);

        render_loop.run_forever(&mut command_runtime, &mut store, &mut backend);

        let widget = render_loop.node().state().unwrap().as_ref();

        backend.entry_point().attach_widget(widget);

        event_rx.attach(None, move |event| {
            action_tx.send(RenderAction::Event(event)).unwrap();
            glib::Continue(true)
        });

        action_rx.attach(None, move |action| {
            let deadline = Instant::now() + DEALINE_PERIOD;

            match action {
                RenderAction::RequestRerender => {}
                RenderAction::Message(message) => {
                    render_loop.push_message(message);
                }
                RenderAction::Event(event) => {
                    render_loop.push_event(event);
                }
            }

            if render_loop.run(&deadline, &mut command_runtime, &mut store, &mut backend)
                == RenderFlow::Suspend
            {
                command_runtime.request_rerender();
            }

            glib::Continue(true)
        });
    }
}

impl<M> EntryPoint<M> for gtk::Window {
    fn attach_widget(&self, widget: &gtk::Widget) {
        self.set_child(Some(widget));
        self.show();
    }
}

impl<M> EntryPoint<M> for gtk::ApplicationWindow {
    fn attach_widget(&self, widget: &gtk::Widget) {
        self.set_child(Some(widget));
        self.show();
    }
}
