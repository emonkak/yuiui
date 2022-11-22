use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use yuiui::{Element, IdPathBuf, RenderFlow, RenderLoop, State, Store, TransferableEvent, View};

use crate::command_runtime::CommandRuntime;

const DEALINE_PERIOD: Duration = Duration::from_millis(50);

#[derive(Debug, Clone)]
pub struct EntryPoint {
    inner: Rc<Inner>,
}

impl EntryPoint {
    pub fn new(window: gtk::ApplicationWindow) -> Self {
        Self {
            inner: Rc::new(Inner {
                window,
                pending_events: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn run<S, M, E>(self, element: E, state: S)
    where
        E: Element<S, M, Self> + 'static,
        <E::View as View<S, M, Self>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::channel();
        let mut command_runtime = CommandRuntime::new(glib::MainContext::default(), message_tx);
        let mut store = Store::new(state);
        let mut render_loop = RenderLoop::create(element, &mut store);

        render_loop.run_forever(&mut command_runtime, &mut store, &self);

        let widget = render_loop.node().state().unwrap().as_ref();

        self.attach_widget(widget);

        while gtk::Window::toplevels().n_items() > 0 {
            let mut needs_render = false;

            while command_runtime.main_context().iteration(true) {
                while let Ok(message) = message_rx.try_recv() {
                    render_loop.push_message(message);
                    needs_render = true;
                }

                for event in self.inner.pending_events.borrow_mut().drain(..) {
                    render_loop.push_event(event);
                    needs_render = true;
                }

                if !command_runtime.main_context().pending() {
                    break;
                }
            }

            if needs_render {
                let deadline = Instant::now() + DEALINE_PERIOD;

                if render_loop.run(&deadline, &mut command_runtime, &mut store, &self)
                    == RenderFlow::Suspend
                {
                    command_runtime.request_rerender();
                    break;
                }
            }
        }
    }

    pub fn forward_event<T: Send + 'static>(&self, destination: IdPathBuf, payload: T) {
        let event = TransferableEvent::Forward(destination, Box::new(payload));
        self.inner.pending_events.borrow_mut().push(event)
    }

    pub fn broadcast_event<T: Send + 'static>(&self, destinations: Vec<IdPathBuf>, payload: T) {
        let event = TransferableEvent::Broadcast(destinations, Box::new(payload));
        self.inner.pending_events.borrow_mut().push(event)
    }

    pub fn window(&self) -> &gtk::ApplicationWindow {
        &self.inner.window
    }

    fn attach_widget(&self, widget: &gtk::Widget) {
        self.inner.window.set_child(Some(widget));
        self.inner.window.show();
    }
}

#[derive(Debug)]
struct Inner {
    window: gtk::ApplicationWindow,
    pending_events: RefCell<Vec<TransferableEvent>>,
}
