use gtk::glib;
use gtk::prelude::*;
use std::time::{Duration, Instant};
use yuiui::{Element, RenderFlow, RenderLoop, State, Store, View};

use crate::execution_context::{ExecutionContext, RenderAction};
use crate::renderer::{EventPort, Renderer};

pub trait EntryPoint<M>: Sized + 'static {
    fn window(&self) -> &gtk::Window;

    fn message(&self, _message: &M) {}

    fn attach(&self, widget: &gtk::Widget, _event_port: &EventPort) {
        let window = self.window();
        window.set_child(Some(widget));
        window.show();
    }

    fn boot<E, S>(self, element: E, state: S)
    where
        E: Element<S, M, Renderer> + 'static,
        <E::View as View<S, M, Renderer>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        const DEALINE_PERIOD: Duration = Duration::from_millis(50);

        let (event_tx, event_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (action_tx, action_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let context = ExecutionContext::new(glib::MainContext::default(), action_tx.clone());

        let mut store = Store::new(state);
        let mut renderer = Renderer::new(self.window().clone(), event_tx);
        let mut render_loop = RenderLoop::create(element, &mut store);

        render_loop.run_forever(&context, &mut store, &mut renderer);

        let widget = render_loop.node().state().as_view_state().unwrap().as_ref();

        self.attach(widget, renderer.event_port());

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
                    self.message(&message);
                    render_loop.push_message(message);
                }
                RenderAction::Event(event, destination) => {
                    render_loop.push_event(event, destination);
                }
            }

            if render_loop.run(&deadline, &context, &mut store, &mut renderer)
                == RenderFlow::Suspend
            {
                context.request_render();
            }

            glib::Continue(true)
        });
    }
}

#[derive(Debug)]
pub struct DefaultEntryPoint<W> {
    window: W,
}

impl<W> From<W> for DefaultEntryPoint<W> {
    #[inline]
    fn from(window: W) -> Self {
        Self { window }
    }
}

impl<W: AsRef<gtk::Window> + 'static, M> EntryPoint<M> for DefaultEntryPoint<W> {
    #[inline]
    fn window(&self) -> &gtk::Window {
        self.window.as_ref()
    }
}
