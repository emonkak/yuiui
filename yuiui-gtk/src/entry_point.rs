use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use yuiui_core::{Element, IdPathBuf, RenderFlow, RenderLoop, State, TransferableEvent, View};

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

    pub fn run<S, M, E>(self, element: E, mut state: S)
    where
        E: Element<S, M, Self> + 'static,
        <E::View as View<S, M, Self>>::State: AsRef<gtk::Widget>,
        S: State<Message = M> + 'static,
        M: Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::channel();
        let command_runtime = CommandRuntime::new(glib::MainContext::default(), message_tx);
        let mut render_loop = RenderLoop::create(element, &mut state);

        render_loop.run_forever(&mut state, &self, &command_runtime);

        let widget = render_loop.node().view_state().unwrap().as_ref();

        self.attach_widget(widget);

        while gtk::Window::toplevels().n_items() > 0 {
            let main_context = command_runtime.main_context();
            let mut needs_render = false;

            while main_context.iteration(true) {
                while let Ok(message) = message_rx.try_recv() {
                    render_loop.push_message(message);
                    needs_render = true;
                }

                for event in self.inner.pending_events.borrow_mut().drain(..) {
                    render_loop.push_event(event);
                    needs_render = true;
                }

                if !main_context.pending() {
                    break;
                }
            }

            if needs_render {
                let deadline = Instant::now() + DEALINE_PERIOD;

                if matches!(
                    render_loop.run(&mut state, &self, &command_runtime, &deadline),
                    RenderFlow::Suspend
                ) {
                    command_runtime.request_rerender();
                    break;
                }
            }
        }

        if let Some(application) = self.inner.window.application() {
            application.quit();
        }
    }

    pub fn dispatch_event<T: Send + 'static>(&self, destination: IdPathBuf, payload: T) {
        let event = TransferableEvent::Forward(destination, Box::new(payload));
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
