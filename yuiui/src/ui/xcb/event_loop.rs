use std::error::Error;
use std::future::Future;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::runtime;
use tokio::sync::mpsc;
use tokio::task;
use tokio::task::{JoinHandle, LocalSet};
use x11rb::connection::Connection;
use x11rb::errors::ConnectionError;
use x11rb::protocol;
use x11rb::protocol::xproto;

use crate::event::{MouseEvent, WindowEvent};
use crate::geometrics::{PhysicalRectangle, PhysicalSize};
use crate::ui::{ControlFlow, Event};

pub struct EventLoop<Connection> {
    connection: Rc<Connection>,
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<Connection> EventLoop<Connection>
where
    Connection: self::Connection + AsRawFd,
{
    pub fn new(connection: Rc<Connection>) -> Self {
        Self { connection }
    }

    async fn run_async<Message, F>(&self, mut callback: F) -> Result<(), ConnectionError>
    where
        F: FnMut(Event<Message, xproto::Window>, &EventLoopContext<Message>) -> ControlFlow,
    {
        let stream_fd = AsyncFd::with_interest(self.connection.as_raw_fd(), Interest::READABLE)?;
        let local_set = LocalSet::new();
        let (message_sender, mut message_receiver) = mpsc::unbounded_channel();

        let context = EventLoopContext {
            local_set,
            message_sender,
        };

        callback(Event::LoopInitialized, &context);

        'outer: loop {
            let result = context
                .local_set
                .run_until(async {
                    tokio::select! {
                        guard = stream_fd.readable() => Either::Left(guard),
                        message = message_receiver.recv() => Either::Right(message),
                    }
                })
                .await;

            match result {
                Either::Left(guard) => {
                    let mut guard = guard.map_err(ConnectionError::IoError)?;

                    while let Some(event) = self.connection.poll_for_event()? {
                        let control_flow = self.process_event(event, &mut callback, &context);
                        if control_flow == ControlFlow::Break {
                            break 'outer;
                        }
                    }

                    guard.clear_ready();
                }
                Either::Right(Some(message)) => {
                    let control_flow = callback(Event::Message(message), &context);
                    if control_flow == ControlFlow::Break {
                        break 'outer;
                    }
                }
                Either::Right(_) => {}
            }
        }

        Ok(())
    }

    fn process_event<Message, F>(
        &self,
        event: protocol::Event,
        callback: &mut F,
        context: &EventLoopContext<Message>,
    ) -> ControlFlow
    where
        F: FnMut(Event<Message, xproto::Window>, &EventLoopContext<Message>) -> ControlFlow,
    {
        match event {
            // Handles only the last expose event.
            protocol::Event::Expose(event) if event.count == 0 => {
                let bounds = PhysicalRectangle {
                    x: event.x as _,
                    y: event.y as _,
                    width: event.width as _,
                    height: event.height as _,
                };
                callback(
                    Event::WindowEvent(event.window, WindowEvent::RedrawRequested(bounds)),
                    context,
                )
            }
            protocol::Event::ButtonPress(event) => callback(
                Event::WindowEvent(
                    event.child,
                    WindowEvent::PointerPressed(MouseEvent::from(event)),
                ),
                context,
            ),
            protocol::Event::ButtonRelease(event) => callback(
                Event::WindowEvent(
                    event.child,
                    WindowEvent::PointerReleased(MouseEvent::from(event)),
                ),
                context,
            ),
            protocol::Event::DestroyNotify(event) => callback(
                Event::WindowEvent(event.window, WindowEvent::Closed),
                context,
            ),
            protocol::Event::ConfigureNotify(event) => {
                let size = PhysicalSize {
                    width: event.width as _,
                    height: event.height as _,
                };
                callback(
                    Event::WindowEvent(event.window, WindowEvent::SizeChanged(size)),
                    context,
                )
            }
            _ => ControlFlow::Continue,
        }
    }
}

impl<Connection, Message> crate::ui::EventLoop<Message> for EventLoop<Connection>
where
    Connection: self::Connection + AsRawFd,
    Message: 'static,
{
    type WindowId = xproto::Window;

    type Context = EventLoopContext<Message>;

    fn run<F>(&mut self, callback: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(Event<Message, xproto::Window>, &Self::Context) -> ControlFlow,
    {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime
            .block_on(self.run_async(callback))
            .map_err(|error| Box::new(error) as Box<dyn Error>)
    }
}

pub struct EventLoopContext<Message> {
    local_set: LocalSet,
    message_sender: mpsc::UnboundedSender<Message>,
}

impl<Message> crate::ui::EventLoopContext<Message> for EventLoopContext<Message>
where
    Message: 'static,
{
    fn send(&self, message: Message) {
        self.message_sender
            .send(message)
            .ok()
            .expect("Failed to send message");
    }

    fn perform<F>(&self, future: F) -> JoinHandle<()>
    where
        F: 'static + Future<Output = Message>,
    {
        let message_sender = self.message_sender.clone();
        self.local_set.spawn_local(async move {
            message_sender
                .send(future.await)
                .ok()
                .expect("Failed to send message");
        })
    }

    fn request_idle<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce(Instant) -> Message,
    {
        let message_sender = self.message_sender.clone();
        self.local_set.spawn_local(async move {
            task::yield_now().await;
            // FIXME: Appropriate deadline
            let deadline = Instant::now() + Duration::from_millis(50);
            message_sender
                .send(callback(deadline))
                .ok()
                .expect("Failed to send message");
        })
    }
}
