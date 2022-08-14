use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
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
use tokio::time;
use x11rb::errors::ConnectionError;
use x11rb::protocol;
use x11rb::protocol::xproto;

use super::utils::refresh_rate;
use crate::event::{MouseEvent, WindowEvent};
use crate::geometrics::PhysicalSize;
use crate::ui::{ControlFlow, Event};

pub struct EventLoop<Connection> {
    connection: Rc<Connection>,
    screen_num: usize,
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

const DEFAULT_REFRESH_RATE: f64 = 60.0;

const MAX_DEALINE_PERIOD: Duration = Duration::from_millis(50);

impl<Connection> EventLoop<Connection>
where
    Connection: x11rb::connection::Connection + AsRawFd,
{
    pub fn new(connection: Rc<Connection>, screen_num: usize) -> Self {
        Self {
            connection,
            screen_num,
        }
    }

    async fn run_async<Message, F>(&self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Event<Message, xproto::Window>, &EventLoopContext<Message>) -> ControlFlow,
    {
        let stream_fd = AsyncFd::with_interest(self.connection.as_raw_fd(), Interest::READABLE)?;
        let local_set = LocalSet::new();
        let (message_sender, mut message_receiver) = mpsc::unbounded_channel();

        let refresh_rate = {
            let screen = &self.connection.setup().roots[self.screen_num];
            refresh_rate(&*self.connection, screen.root)
        }?
        .unwrap_or(DEFAULT_REFRESH_RATE);

        let mut context = EventLoopContext {
            local_set,
            message_sender,
            refresh_rate,
            available_timers: Rc::new(RefCell::new(BinaryHeap::new())),
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
                        let control_flow =
                            self.process_event(event, &mut callback, &mut context)?;
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
        context: &mut EventLoopContext<Message>,
    ) -> anyhow::Result<ControlFlow>
    where
        F: FnMut(Event<Message, xproto::Window>, &EventLoopContext<Message>) -> ControlFlow,
    {
        match event {
            // Handles only the last expose event because do not optimize by distinguishing between
            // subareas.
            protocol::Event::Expose(event) if event.count == 0 => Ok(callback(
                Event::WindowEvent(event.window, WindowEvent::RedrawRequested),
                context,
            )),
            protocol::Event::ButtonPress(event) => Ok(callback(
                Event::WindowEvent(
                    event.child,
                    WindowEvent::PointerPressed(MouseEvent::from(event)),
                ),
                context,
            )),
            protocol::Event::ButtonRelease(event) => Ok(callback(
                Event::WindowEvent(
                    event.child,
                    WindowEvent::PointerReleased(MouseEvent::from(event)),
                ),
                context,
            )),
            protocol::Event::DestroyNotify(event) => Ok(callback(
                Event::WindowEvent(event.window, WindowEvent::Closed),
                context,
            )),
            protocol::Event::ConfigureNotify(event) => {
                let size = PhysicalSize {
                    width: event.width as _,
                    height: event.height as _,
                };
                Ok(callback(
                    Event::WindowEvent(event.window, WindowEvent::Resized(size)),
                    context,
                ))
            }
            protocol::Event::RandrScreenChangeNotify(event) => {
                context.refresh_rate =
                    refresh_rate(&*self.connection, event.root)?.unwrap_or(DEFAULT_REFRESH_RATE);
                Ok(ControlFlow::Continue)
            }
            _ => Ok(ControlFlow::Continue),
        }
    }
}

impl<Connection, Message> crate::ui::EventLoop<Message> for EventLoop<Connection>
where
    Connection: x11rb::connection::Connection + AsRawFd,
    Message: 'static,
{
    type WindowId = xproto::Window;

    type Context = EventLoopContext<Message>;

    fn run<F>(&mut self, callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Event<Message, xproto::Window>, &Self::Context) -> ControlFlow,
    {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(self.run_async(callback))
    }
}

pub struct EventLoopContext<Message> {
    local_set: LocalSet,
    message_sender: mpsc::UnboundedSender<Message>,
    refresh_rate: f64,
    available_timers: Rc<RefCell<BinaryHeap<Reverse<Instant>>>>,
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

    fn delay<F>(&self, duration: Duration, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce() -> Message,
    {
        let message_sender = self.message_sender.clone();
        let available_timers = self.available_timers.clone();
        let sleep_until = Instant::now() + duration;

        available_timers.borrow_mut().push(Reverse(sleep_until));

        self.local_set.spawn_local(async move {
            time::sleep_until(sleep_until.into()).await;
            message_sender
                .send(callback())
                .ok()
                .expect("Failed to send message");
            available_timers.borrow_mut().pop();
        })
    }

    fn request_animation_frame<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce() -> Message,
    {
        let message_sender = self.message_sender.clone();
        let available_timers = self.available_timers.clone();
        let next_animation_frame =
            Instant::now() + Duration::from_secs_f64(1.0 / self.refresh_rate);

        available_timers
            .borrow_mut()
            .push(Reverse(next_animation_frame));

        self.local_set.spawn_local(async move {
            time::sleep_until(next_animation_frame.into()).await;
            message_sender
                .send(callback())
                .ok()
                .expect("Failed to send message");
            available_timers.borrow_mut().pop();
        })
    }

    fn request_idle<F>(&self, callback: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce(Instant) -> Message,
    {
        let message_sender = self.message_sender.clone();
        let available_timers = self.available_timers.clone();

        self.local_set.spawn_local(async move {
            task::yield_now().await;
            let max_deadline = Instant::now() + MAX_DEALINE_PERIOD;
            let deadline = available_timers
                .borrow()
                .peek()
                .map_or(max_deadline, |Reverse(next_wake)| {
                    max_deadline.min(*next_wake)
                });
            message_sender
                .send(callback(deadline))
                .ok()
                .expect("Failed to send message");
        })
    }
}
